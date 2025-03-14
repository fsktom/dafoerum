//! Each function here should be `#[server]` function
//!
//! Helper functions are in the [`helper`]` submodule

#[cfg(feature = "ssr")]
mod helper;

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
    #[error("server fn error: {0}")]
    ServerFn(ServerFnErrorErr),
    #[error("opaque mongodb error: {0}")]
    Db(String),
    #[error("(de)serialization error in db: {0}")]
    DbDeSer(String),
    #[error("database not in leptos context")]
    DbNotInContext,
    #[error("{0} with id {1} doesn't exist in the database")]
    NotFound(String, u32),
    #[error("content cannot be empty")]
    EmptyContent,
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
        use mongodb::error::ErrorKind::*;
        let msg = value.to_string();
        match *value.kind {
            BsonSerialization(_) | BsonDeserialization(_) => ApiError::DbDeSer(msg),
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

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Counter {
    category: String,
    sequence: u32,
}
impl CollectionName for Counter {
    fn collection_name() -> &'static str {
        "counters"
    }
}

/// Represents a thread: it's part of a forum (tbd) and contains multiple posts
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Thread {
    pub id: u32,
    pub origin_post_id: u32,
    pub subject: String,
    pub forum_id: u32,
}
impl CollectionName for Thread {
    fn collection_name() -> &'static str {
        "threads"
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: u32,
    pub content: String,

    #[serde(with = "jiff_timestamp_as_bson_datetime")]
    pub created_at: jiff::Timestamp,
    pub thread_id: u32,
}
impl Post {
    /// 2025-03-07T02:12:38+01:00
    pub fn date_in_berlin(&self) -> String {
        self.created_at
            .in_tz("Europe/Berlin")
            .unwrap()
            .strftime("%FT%T%:z")
            .to_string()
    }
}
#[cfg(feature = "ssr")]
impl CollectionName for Post {
    fn collection_name() -> &'static str {
        "posts"
    }
}

/// Looks up if the given `thread_id` exists in the database and returns the [`Thread`] if so
#[server]
pub async fn get_thread(thread_id: u32) -> Result<Thread, ApiError> {
    let db = helper::get_db()?;
    helper::get_thread(thread_id, db).await
}

/// Fetches all [`Threads`][Thread] from the database in id-descending order
#[server]
pub async fn get_threads() -> Result<Vec<Thread>, ApiError> {
    let db = helper::get_db()?;
    let thread_col = Thread::collection(&db);
    let mut threads = vec![];
    let mut threads_cursor = thread_col
        .find(bson::doc! {})
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
    use bson::DateTime;
    // https://docs.rs/bson/latest/bson/serde_helpers/chrono_datetime_as_bson_datetime
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    /// Deserializes a [`jiff::Timestamp`] from a [`bson::DateTime`].
    pub fn deserialize<'de, D>(deserializer: D) -> Result<jiff::Timestamp, D::Error>
    where
        D: Deserializer<'de>,
    {
        let datetime = bson::DateTime::deserialize(deserializer)?;
        Ok(jiff::Timestamp::from_millisecond(datetime.timestamp_millis()).expect("bad mongo"))
    }

    /// Serializes a [`jiff::Timestamp`] as a [`bson::DateTime`].
    pub fn serialize<S: Serializer>(
        val: &jiff::Timestamp,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        let datetime = DateTime::from_millis(val.as_millisecond());
        datetime.serialize(serializer)
    }
}
