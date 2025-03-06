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
    pub created_at: String,
}

#[server(endpoint = "whos")]
pub async fn get_db_posts() -> Result<Vec<Post>, ServerFnError> {
    let db = use_context::<Database>().unwrap();
    let a = db.collection::<Post>("posts");
    let mut psts = vec![];
    let mut c = a.find(bson::doc! {}).await.unwrap();
    while c.advance().await.unwrap() {
        psts.push(c.deserialize_current().unwrap());
    }
    Ok(psts)
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

    let post_col = db.collection::<Post>("posts");

    let p = Post {
        id: id + 1,
        content,
        created_at: bson::DateTime::now().to_string(),
    };

    post_col.insert_one(&p).await.unwrap();

    Ok(())
}
