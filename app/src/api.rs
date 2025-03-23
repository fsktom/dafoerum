//! Each function here should be `#[server]` function
//!
//! Helper functions are in the [`helper`] submodule

#[cfg(feature = "ssr")]
pub mod helper;

#[cfg(feature = "ssr")]
use mongodb::{Collection, Database, bson};

use leptos::{
    prelude::*,
    server_fn::error::{FromServerFnError, ServerFnErrorErr},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Error type used for backend-frontend interaction
#[derive(Debug, Clone, Error, Deserialize, Serialize)]
pub enum ApiError {
    /// A wrapper around [`ServerFnErrorErr`], basically whenever something
    /// happens between the request and response
    #[error("server fn error: {0}")]
    ServerFn(ServerFnErrorErr),

    /// Used when a generic mongodb error happens
    #[error("opaque mongodb error: {0}")]
    Db(String),
    /// Used for de/serializations errors during database communication
    #[error("(de)serialization error in db: {0}")]
    DbDeSer(String),
    /// Used when, for some reason (shouldn't ever happen...), the
    /// database is not in leptos context (see [`provide_context`])
    #[error("database not in leptos context")]
    DbNotInContext,

    /// Used when the given item doesn't exist in the databse
    #[error("{0} with id {1} doesn't exist in the database")]
    NotFound(String, u32),

    /// Used when the content of a post is empty
    #[error("content cannot be empty")]
    EmptyContent,
    /// Used when the subject of a thread is empty
    #[error("subject cannot be empty")]
    EmptySubject,
}
impl FromServerFnError for ApiError {
    fn from_server_fn_error(value: ServerFnErrorErr) -> Self {
        ApiError::ServerFn(value)
    }
}
#[cfg(feature = "ssr")]
impl From<mongodb::error::Error> for ApiError {
    fn from(value: mongodb::error::Error) -> Self {
        use mongodb::error::ErrorKind as E;
        let msg = value.to_string();
        match *value.kind {
            E::BsonSerialization(_) | E::BsonDeserialization(_) => ApiError::DbDeSer(msg),
            _ => ApiError::Db(msg),
        }
    }
}

/// Responsible for blanket implementation of [`GetCollection`]
#[allow(dead_code, reason = "bc only used by ssr-only")]
trait CollectionName: std::marker::Send + std::marker::Sync + std::marker::Sized {
    /// Name of the corresponding [`Collection`] in the [`Database`]
    fn collection_name() -> &'static str;
}

/// Trait for easy access to the type's [`Collection`] in the [`Database`]
///
/// You shouldn't have to implement this directly. Instead, implement [`CollectionName`]
/// with the name of the collection holding this type in the db,
/// and this trait will be implemented automatically.
#[cfg(feature = "ssr")]
trait GetCollection: CollectionName {
    /// Returns the type's [`Collection`] in the [`Database`]
    fn collection(db: &Database) -> Collection<Self>;
}
#[cfg(feature = "ssr")]
impl<T> GetCollection for T
where
    T: CollectionName,
{
    fn collection(db: &Database) -> Collection<Self> {
        db.collection::<Self>(Self::collection_name())
    }
}

/// Used for creating new things that require an incrementing id
///
/// I didn't want to use UUID for everything, because I like the idea
/// of an ever-increasing post, thread, forum, user id :)
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Counter {
    /// Name of the thing to sequence, e.g. "post", "thread", "user" etc.
    ///
    /// This *must* exist in the db collection, otherwise dependent functions
    /// like [`helper::get_and_increment_id_of`] will panic
    category: String,

    /// Current highest id
    sequence: u32,
}
impl CollectionName for Counter {
    fn collection_name() -> &'static str {
        "counters"
    }
}

/// Represents a category: contains multiple top-level [`Forums`][Forum]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Category {
    pub name: String,
    pub order: u32,
    pub forums: Vec<Forum>,
}
impl CollectionName for Category {
    fn collection_name() -> &'static str {
        "categories"
    }
}

/// Represents a top-level forum: contains multiple [`Threads`][Thread]
/// TBD: can contain [`Subforum`][Subforum]
///
/// [`Threads`][Thread] are saved in a separate db collection and refer to their parent forum
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Forum {
    pub id: u32,
    pub name: String,
}

/// Represents a thread: it's part of a [`Forum`] and contains multiple [`Posts`][Post]
///
/// [`Posts`][Post] are saved in a separate db collection and refer to their parent thread
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thread {
    pub id: u32,
    pub origin_post_id: u32,
    pub forum_id: u32,
    pub subject: String,
}
impl CollectionName for Thread {
    fn collection_name() -> &'static str {
        "threads"
    }
}

/// Represents a post: it's part of a thread and contains a message
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: u32,
    pub content: String,

    /// Will be de/serialized as [`bson::DateTime`] for communication with the db
    #[serde(with = "jiff_timestamp_as_bson_datetime")]
    pub created_at: jiff::Timestamp,
    pub thread_id: u32,
}
impl Post {
    /// 2025-03-07T02:12:38+01:00
    #[allow(
        clippy::missing_panics_doc,
        reason = "unwrapping after a static str in"
    )]
    #[must_use]
    pub fn date_in_berlin(&self) -> String {
        self.created_at
            .in_tz("Europe/Berlin")
            .unwrap()
            .strftime("%FT%T%:z")
            .to_string()
    }
}
impl CollectionName for Post {
    fn collection_name() -> &'static str {
        "posts"
    }
}

/// Queries all [`Categories`][Category] contining top-level [`Forums`][Forum] from the db
#[server]
pub async fn get_categories() -> Result<Vec<Category>, ApiError> {
    let db = helper::get_db()?;
    // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    let category_col = Category::collection(&db);
    let mut categories = vec![];
    let mut categories_cursor = category_col
        .find(bson::doc! {})
        // ascending
        .sort(bson::doc! {"order": 1})
        .await?;
    while categories_cursor.advance().await? {
        categories.push(categories_cursor.deserialize_current()?);
    }
    Ok(categories)
}

/// Looks up if the given `forum_id` exists in the database and returns the [`Forum`]
/// with the name of its [`Category`] if so
#[server]
pub async fn get_forum(forum_id: u32) -> Result<(Forum, String), ApiError> {
    let db = helper::get_db()?;
    // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    helper::get_forum(forum_id, db).await
}

/// Looks up if the given `thread_id` exists in the database and returns the [`Thread`] if so
#[server]
pub async fn get_thread(thread_id: u32) -> Result<Thread, ApiError> {
    let db = helper::get_db()?;
    // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    helper::get_thread(thread_id, db).await
}

/// Fetches all [`Threads`][Thread] of a given [`Forum`] from the database in id-descending order
#[server]
pub async fn get_threads(forum_id: u32) -> Result<Vec<Thread>, ApiError> {
    let db = helper::get_db()?;
    // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let thread_col = Thread::collection(&db);
    let mut threads = vec![];
    let mut threads_cursor = thread_col
        .find(bson::doc! {"forum_id": forum_id})
        // descending
        .sort(bson::doc! {"id": -1})
        .await?;
    while threads_cursor.advance().await? {
        threads.push(threads_cursor.deserialize_current()?);
    }
    Ok(threads)
}

/// Tries to create a [`Thread`] within the given forum and with a [`Post`] of `post_content`
///
/// Will error if `subject` or `post_content` are empty
///
/// Returns the `thread_id` of the created [`Thread`]
#[server]
pub async fn create_thread(
    forum_id: u32,
    subject: String,
    post_content: String,
) -> Result<u32, ApiError> {
    if subject.is_empty() {
        return Err(ApiError::EmptySubject);
    }
    if post_content.is_empty() {
        return Err(ApiError::EmptyContent);
    }

    let db = helper::get_db()?;

    let _ = helper::get_forum(forum_id, db.clone()).await?;

    let counter_col = Counter::collection(&db);
    let thread_id = helper::get_and_increment_id_of("thread", counter_col.clone()).await?;

    let post_col = Post::collection(&db);
    let post_id = helper::get_and_increment_id_of("post", counter_col).await?;
    let new_post = Post {
        id: post_id,
        content: post_content,
        created_at: jiff::Timestamp::now(),
        thread_id,
    };
    post_col.insert_one(&new_post).await?;

    let thread_col = Thread::collection(&db);
    let new_thread = Thread {
        id: thread_id,
        origin_post_id: post_id,
        subject,
        forum_id,
    };
    thread_col.insert_one(&new_thread).await?;

    Ok(thread_id)
}

/// Fetches the latest `num` [`Posts`][Post] from the database in id-descending order
#[server]
pub async fn get_latest_posts(num: i64) -> Result<Vec<Post>, ApiError> {
    let db = helper::get_db()?;
    let post_col = Post::collection(&db);
    let mut posts = vec![];
    let mut post_cursor = post_col
        .find(bson::doc! {})
        // descending
        .sort(bson::doc! {"id":-1})
        .limit(num)
        .await?;
    while post_cursor.advance().await? {
        posts.push(post_cursor.deserialize_current()?);
    }
    Ok(posts)
}

/// Fetches a certain thread's [`Posts`][Post] from the databse in id-ascending order
#[server]
pub async fn get_posts_from_thread(thread_id: u32) -> Result<Vec<Post>, ApiError> {
    let db = helper::get_db()?;
    // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    let post_col = Post::collection(&db);
    let mut posts = vec![];
    let mut post_cursor = post_col
        .find(bson::doc! {"thread_id": thread_id})
        // ascending
        .sort(bson::doc! {"id": 1})
        .await?;
    while post_cursor.advance().await? {
        posts.push(post_cursor.deserialize_current()?);
    }
    Ok(posts)
}

/// Creates a post in the given [`Thread`]
///
/// # Errors
///
/// - [`ApiError::EmptyContent`] if `content` is empty
/// - [`ApiError::NotFound`] if `thread_id` isn't in use
#[server]
pub async fn create_post(thread_id: u32, content: String) -> Result<(), ApiError> {
    if content.is_empty() {
        return Err(ApiError::EmptyContent);
    }

    let db = helper::get_db()?;

    let _ = helper::get_thread(thread_id, db.clone()).await?;

    let counter_col = Counter::collection(&db);
    let id = helper::get_and_increment_id_of("post", counter_col).await?;

    let post_col = Post::collection(&db);

    let new_post = Post {
        id,
        content,
        created_at: jiff::Timestamp::now(),
        thread_id,
    };

    post_col.insert_one(&new_post).await?;

    Ok(())
}

pub mod jiff_timestamp_as_bson_datetime {
    // https://docs.rs/bson/latest/bson/serde_helpers/chrono_datetime_as_bson_datetime
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserializes a [`jiff::Timestamp`] from a [`bson::DateTime`].
    #[allow(clippy::missing_errors_doc)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<jiff::Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let datetime = bson::DateTime::deserialize(deserializer)?;
        let Ok(timestamp) = jiff::Timestamp::from_millisecond(datetime.timestamp_millis()) else {
            unreachable!(
                "a bson DateTime in ms shouldn't be out of range for jiff timestamp creation"
            )
        };
        Ok(timestamp)
    }

    /// Serializes a [`jiff::Timestamp`] as a [`bson::DateTime`].
    #[allow(clippy::missing_errors_doc)]
    pub fn serialize<S: Serializer>(
        val: &jiff::Timestamp,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let datetime = bson::DateTime::from_millis(val.as_millisecond());
        datetime.serialize(serializer)
    }
}
