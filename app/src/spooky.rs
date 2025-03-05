#[cfg(feature = "ssr")]
use mongodb::Database;
#[cfg(feature = "ssr")]
use mongodb::bson;

use leptos::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Post {
    pub id: usize,
    pub content: String,
    // gay: f64,
}

#[server(endpoint = "whos")]
pub async fn get_db_posts() -> Result<Vec<Post>, ServerFnError> {
    let db = use_context::<Database>().unwrap();
    let a = db.collection::<Post>("rigid_posts");
    let mut psts = vec![];
    let mut c = a.find(bson::doc! {}).await.unwrap();
    while c.advance().await.unwrap() {
        psts.push(c.deserialize_current().unwrap());
    }
    Ok(psts)
}
