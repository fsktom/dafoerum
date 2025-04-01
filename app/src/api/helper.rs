//! Contains helper functions used in [`super`]
//!
//! Separating it because I want [`super`] to only contain functions that
//! are also API endpoints (`#[server]`)

use super::{
    ApiError, Category, Collection, Counter, Database, Forum, GetCollection, Post, Thread, bson,
};
use leptos::prelude::*;

/// Gives access to the [`Database`]
///
/// # Errors
///
/// * [`ApiError::DbNotInContext`] if the db hasn't been saved via [`provide_context`] before
///   (for future: maybe panic instead since it's irrecoverrible?
///   maybe at the top level in server somehow...)
pub fn get_db() -> Result<Database, ApiError> {
    use_context::<Database>().ok_or(ApiError::DbNotInContext)
}

/// Looks up the current sequence of a post/thread/..., increments it and returns the incremented value
///
/// Required when creating new such element
///
/// # Panics
///
/// Will panic if the given `category` doesn't exist in the [`Collection<Counter>`] of the database
///
/// # Errors
///
/// * [`ApiError::Db`] if the db connection fails in any way
pub async fn get_and_increment_id_of(
    category: &'static str,
    counter_col: Collection<Counter>,
) -> Result<u32, ApiError> {
    let current_id = counter_col
        .find_one_and_update(
            bson::doc! {"category": category},
            bson::doc! {"$inc": {"sequence": 1}},
        )
        .await?
        // maybe later: create counter if missing instead of panicking
        .expect("counter should exist in db")
        .sequence;
    Ok(current_id + 1)
}

/// Queries database to check if a [`Post`] with the given `post_id` exists
/// and returns it.
///
/// # Errors
///
/// * [`ApiError::NotFound`] if the `post_id` is not in the db
/// * [`ApiError::Db`] if the db connection fails in any way
pub async fn get_post(post_id: u32, db: Database) -> Result<Post, ApiError> {
    let post_col = Post::collection(&db);
    let post = post_col.find_one(bson::doc! {"id": post_id}).await?;

    // invariant: if post is saved in database, the thread it is in must also exist

    post.ok_or(ApiError::NotFound("post".into(), post_id))
}

/// Queries database to check if a [`Thread`] with the given `thread_id` exists
/// and returns it.
///
/// # Errors
///
/// * [`ApiError::NotFound`] if the `thread_id` is not in the db
/// * [`ApiError::Db`] if the db connection fails in any way
pub async fn get_thread(thread_id: u32, db: Database) -> Result<Thread, ApiError> {
    let thread_col = Thread::collection(&db);
    let thread = thread_col.find_one(bson::doc! {"id": thread_id}).await?;

    // invariant: if thread is saved in database, the forum it is in must also exist

    thread.ok_or(ApiError::NotFound("thread".into(), thread_id))
}

/// Queries database to check if a [`Forum`] with the given `forum_id` exists
/// and returns it with the name of its [`Category`]
///
/// # Errors
///
/// * [`ApiError::NotFound`] if the `forum_id` is not in the db
/// * [`ApiError::Db`] if the db connection fails in any way
#[allow(
    clippy::missing_panics_doc,
    reason = "made sure otherwise it's ok to unwrap"
)]
pub async fn get_forum(forum_id: u32, db: Database) -> Result<(Forum, String), ApiError> {
    let category_col = Category::collection(&db);
    let category = category_col
        .find_one(bson::doc! {"forums.id": forum_id})
        .await?;

    // easier than dealing with projections in mongodb and Rust (maybe someday I'm skilled enough)
    let Some(category) = category else {
        return Err(ApiError::NotFound("forum".into(), forum_id));
    };

    // or maybe in future make sure this array is sorted -> binary search for forum_id
    // but even then, with projections I could make less data be sent over the network
    let forum = category
        .forums
        .into_iter()
        .find(|f| f.id == forum_id)
        .expect("already checked before for existence");

    Ok((forum, category.name))
}

/// Queries the databse for the amount of [`Thread`]s for the given `forum_id`
///
/// Doesn't check for [`Forum`] existence, will probably return `0` for such
///
/// # Errors
///
/// * [`ApiError::Db`] if the db connection fails in any way
pub async fn count_threads_of(forum_id: u32, db: Database) -> Result<u64, ApiError> {
    let thread_col = Thread::collection(&db);
    let count = thread_col
        .count_documents(bson::doc! {"forum_id": forum_id})
        .await?;
    Ok(count)
}

/// Queries the databse for the amount of [`Post`]s for the given `thread_id`
///
/// Doesn't check for [`Thread`] existence, will probably return `0` for such
///
/// # Errors
///
/// * [`ApiError::Db`] if the db connection fails in any way
pub async fn count_posts_of(thread_id: u32, db: Database) -> Result<u64, ApiError> {
    let post_col = Post::collection(&db);
    let count = post_col
        .count_documents(bson::doc! {"thread_id": thread_id})
        .await?;
    Ok(count)
}
