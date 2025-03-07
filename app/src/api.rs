#[cfg(feature = "ssr")]
use mongodb::{Database, bson};

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Counter {
    category: String,
    sequence: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: usize,
    pub content: String,
    pub created_at: jiff::Timestamp,
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
impl From<PostDB> for Post {
    fn from(value: PostDB) -> Self {
        Post {
            id: value.id,
            content: value.content,
            created_at: jiff::Timestamp::from_millisecond(value.created_at.timestamp_millis())
                .expect("bad mongo"),
        }
    }
}

#[cfg(feature = "ssr")]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PostDB {
    pub id: usize,
    pub content: String,
    pub created_at: bson::DateTime,
}

#[server(endpoint = "whos")]
pub async fn get_db_posts() -> Result<Vec<Post>, ServerFnError> {
    let db = use_context::<Database>().unwrap();
    let post_col = db.collection::<PostDB>("posts");
    let mut posts = vec![];
    let mut post_cursor = post_col
        .find(bson::doc! {})
        .sort(bson::doc! {"id": -1})
        .await
        .unwrap();
    while post_cursor.advance().await.unwrap() {
        posts.push(Post::from(post_cursor.deserialize_current().unwrap()));
    }
    Ok(posts)
}

#[server]
pub async fn create_post(content: String) -> Result<(), ServerFnError> {
    let db = use_context::<Database>().unwrap();

    let counter_col = db.collection::<Counter>("counters");
    let id = counter_col
        .find_one_and_update(
            bson::doc! {"category": "post"},
            bson::doc! {"$inc": {"sequence": 1}},
        )
        .await
        .unwrap()
        .unwrap()
        .sequence;

    let post_col = db.collection::<PostDB>("posts");

    let new_post = PostDB {
        id: id + 1,
        content,
        created_at: bson::DateTime::now(),
    };

    post_col.insert_one(&new_post).await.unwrap();

    Ok(())
}
