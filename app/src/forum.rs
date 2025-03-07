use crate::api;

use leptos::prelude::*;

#[component]
pub fn PostItem(post: api::Post) -> impl IntoView {
    view! {
      <article class="p-6 w-full max-w-md bg-white rounded-lg border border-gray-200 shadow-sm0">
        <div class="flex justify-between">
          <h6 class="mb-2 text-xs font-bold tracking-tight text-gray-900">
            "Posted at "<time datetime=post.date_in_berlin()>{post.date_in_berlin()}</time>
          </h6>
          <h6 class="mb-2 text-xs font-bold tracking-tight text-gray-900">"Post #"{post.id}</h6>
        </div>
        // to render newlines
        <p class="mb-3 font-normal text-gray-700 whitespace-pre-wrap break-words">{post.content}</p>
      </article>
    }
}
