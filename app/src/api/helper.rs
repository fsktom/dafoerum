//! Contains helper functions used in [`super`]
//!
//! Separating it because I want [`super`] to only contain functions that
//! are also API endpoints (`#[server]`)

use super::{ApiError, Collection, Counter, Database, GetCollection, Thread, bson};
use leptos::prelude::*;

/// Gives access to the [`Database`]
pub fn get_db() -> Result<Database, ApiError> {
    use_context::<Database>().ok_or(ApiError::DbNotInContext)
}

/// Looks up the current sequence of a post/thread/..., increments it and returns the incremented value
///
/// Required when creating new such element
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

/// Like [`get_thread`] but not as API endpoint
pub async fn get_thread(thread_id: u32, db: Database) -> Result<Thread, ApiError> {
    let threads_col = Thread::collection(&db);
    let thread = threads_col.find_one(bson::doc! {"id": thread_id}).await?;

    thread.ok_or(ApiError::NotFound("thread".into(), thread_id))
}
