use crate::api;

use leptos::{logging, prelude::*};
// use leptos_meta::Title;
use leptos_router::{hooks::use_params, params::Params};

/// Renders a list of all threads
#[component]
pub fn Threads() -> impl IntoView {
    let threads = Resource::new(move || (), move |_| api::get_threads());

    let thread_list_view = move || {
        Suspend::new(async move {
            threads
                .await
                .unwrap()
                .into_iter()
                .map(|thread| {
                    view! {
                      <li class="space-y-1 max-w-md list-disc list-inside text-gray-500">
                        <a
                          href=format!("/thread/{}", thread.id)
                          class="font-medium text-blue-600 underline hover:no-underline"
                        >
                          {thread.subject}
                        </a>
                      </li>
                    }
                })
                .collect_view()
        })
    };

    view! { <ol class="mb-2 text-lg font-semibold text-gray-900">{thread_list_view}</ol> }
}

/// Parameters for /thread/:id
#[derive(Params, PartialEq, Clone, Copy)]
struct ThreadParams {
    id: u32,
}

/// Renders the post list of a thread
#[component]
pub fn ThreadOverview() -> impl IntoView {
    let params = use_params::<ThreadParams>();
    let Ok(ThreadParams { id }) = params.get_untracked() else {
        return view! {
          <h2 class="text-4xl font-bold">"Invalid id!"</h2>
          <a href="/" class="block rounded-sm hover:text-blue-700">
            <h3 class="text-3xl font-bold">"Go to the frontpage"</h3>
          </a>
        }
        .into_any();
    };

    let n = Resource::new(move || (), move |_| api::get_thread(id));
    let a = Suspend::new(async move {
        let thread = match n.await {
            Ok(thread) => thread,
            Err(err) => {
                logging::log!("{err:?} - {err}");
                return view! { <h2 class="text-4xl font-bold">"Error occured! " {format!("{err:?}")}</h2> }
                    .into_any();
            }
        };
        view! {
          <h2 class="text-4xl font-bold">{thread.subject}</h2>
          <p>"Thread id: "{thread.id}</p>
          <p>"Origin post id: "{thread.origin_post_id}</p>
        }
        .into_any()
    });

    view! {
      {a}
      <Posts thread_id=id />
    }
    .into_any()
}

/// Renders a list of posts from the given thread
#[component]
fn Posts(thread_id: u32) -> impl IntoView {
    // change to readsignal<u32> when implementing multiview (multiple threads at once)?

    let create_post = ServerAction::<api::CreatePost>::new();

    let posts = Resource::new(
        move || create_post.version().get(),
        move |_| api::get_posts_from_thread(thread_id),
    );

    let (comment, set_comment) = signal(String::new());

    let post_list_view = move || {
        Suspend::new(async move {
            posts
                .await
                .unwrap()
                .into_iter()
                .map(|post| {
                    view! {
                      <li>
                        <PostItem post />
                      </li>
                    }
                })
                .collect_view()
        })
    };

    view! {
      // https://flowbite.com/docs/forms/textarea/#comment-box
      <ActionForm
        action=create_post
        attr:class="max-w-md w-full mb-4 border border-gray-200 rounded-lg bg-gray-50"
        // clear textarea after comment posted
        // TODO: don't clear if posting fails :))))
        // maybe easy but I'm stoopid -> only plausible option would be to react to Vec<Post> changes
        on:submit=move |_| { set_comment(String::new()) }
      >
        // I hope there's a better way to do this...
        <input class="hidden" name="thread_id" value=thread_id />
        <textarea
          name="content"
          rows="5"
          placeholder="Write a post..."
          prop:value=comment
          class="py-2 px-4 w-full text-sm text-gray-900 bg-white rounded-t-lg border-0 focus:ring-0 placeholder:italic"
        ></textarea>
        <div class="flex justify-between items-center py-2 px-3 border-t border-gray-200">
          <input
            type="submit"
            value="Create Post"
            class="inline-flex items-center py-2.5 px-4 text-xs font-medium text-center text-white bg-blue-700 rounded-lg hover:bg-blue-800 focus:ring-4 focus:ring-blue-200"
          />
        </div>
      </ActionForm>
      <ol class="flex flex-col gap-2">{post_list_view}</ol>
    }
}

#[component]
pub fn PostItem(post: api::Post) -> impl IntoView {
    view! {
      <article class="p-6 w-full max-w-md bg-white rounded-lg border border-gray-200 shadow-sm0">
        <div class="flex justify-between">
          <h6 class="mb-2 text-xs font-bold tracking-tight text-gray-900">
            "Posted at "<time datetime=post.date_in_berlin()>{post.date_in_berlin()}</time>
          </h6>
          <h6 class="mb-2 text-xs font-bold tracking-tight text-gray-900">
            "Post #"{post.id}" in "
            <a
              href=format!("/thread/{}", post.thread_id)
              class="font-medium text-blue-600 underline hover:no-underline"
            >
              "Thread #"
              {post.thread_id}
            </a>
          </h6>
        </div>
        // to render newlines
        <p class="mb-3 font-normal text-gray-700 whitespace-pre-wrap break-words">{post.content}</p>
      </article>
    }
}
