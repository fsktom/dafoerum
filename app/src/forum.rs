use crate::api;

use leptos::prelude::*;

#[component]
pub fn PostItem(post: api::Post) -> impl IntoView {
    view! {
      <article class="flex flex-col p-4 bg-amber-200 border border-amber-400">
        <div class="flex justify-between">
          <h3>"Posted at "{post.created_at}</h3>
          <h3>"Post #"{post.id}</h3>
        </div>
        <p>{post.content}</p>
      </article>
    }
}
