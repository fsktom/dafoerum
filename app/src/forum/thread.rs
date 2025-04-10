// use crate::TimeUtils;
use crate::api;
use api::{ApiError, Post};

use leptos::either::Either;
use leptos::html::ol;
use leptos::{logging, prelude::*};
// use leptos_meta::Title;
use leptos_router::{hooks::use_params, params::Params};

/// Parameters for /thread/:id
#[derive(Params, PartialEq, Clone, Copy)]
struct ThreadParams {
    id: u32,
}

/// Renders the post list of a [`Thread`]
#[component]
pub fn ThreadOverview() -> impl IntoView {
    let params = use_params::<ThreadParams>();
    let Ok(ThreadParams { id }) = params.get_untracked() else {
        let view = view! {
          <h2 class="text-4xl font-bold">"Invalid id!"</h2>
          <a href="/" class="block rounded-sm hover:text-blue-700">
            <h3 class="text-3xl font-bold">"Go to the frontpage"</h3>
          </a>
        };
        return Either::Left(view);
    };

    let thread_res = Resource::new(move || (), move |()| api::get_thread(id));

    let thread_head_view = move || {
        Suspend::new(async move {
            let thread = match thread_res.await {
                Ok(thread) => thread,
                Err(err) => {
                    logging::log!("{err:?} - {err}");
                    let view = view! { <h2 class="text-4xl font-bold">"Error occured! " {format!("{err:?}")}</h2> };
                    return Either::Left(view);
                }
            };
            let forum_res = Resource::new(move || (), move |()| api::get_forum(thread.forum_id));
            let (forum, category_name) = match forum_res.await {
                Ok(n) => (n.0, n.1),
                Err(err) => {
                    // will only  occur if forum_id in thread doesn't exist as a forum
                    // => breaks invariant in get_thread
                    logging::log!("{err:?} - {err}");
                    let view = view! { <h2 class="text-4xl font-bold">"Error occured! " {format!("{err:?}")}</h2> };
                    return Either::Left(view);
                }
            };
            let view = view! {
              <p>
                <a href="/" class="font-medium text-blue-600 underline hover:no-underline">
                  "Forum"
                </a>
                " -> "
                {category_name.to_string()}
                " -> "
                <a
                  href=format!("/forum/{}", forum.id)
                  class="font-medium text-blue-600 underline hover:no-underline"
                >
                  {forum.name.to_string()}
                </a>
                " -> "
                <a
                  href=format!("/thread/{}", thread.id)
                  class="font-medium text-blue-600 underline hover:no-underline"
                >
                  {thread.subject.to_string()}
                </a>
              </p>
              <h2 class="text-4xl font-bold">{thread.subject}</h2>
              <p>"Thread id: "{thread.id}</p>
              <p>"Origin post id: "{thread.origin_post_id}</p>
            };
            Either::Right(view)
        })
    };

    let view = view! {
      <Suspense fallback=move || {
        view! { <p>"Loading thread..."</p> }
      }>{thread_head_view}<Posts thread_id=id /></Suspense>
    };
    Either::Right(view)
}

/// Renders a list of [`Posts`][Post] from the given [`Thread`]
#[component]
fn Posts(thread_id: u32) -> impl IntoView {
    // change to readsignal<u32> when implementing multiview (multiple threads at once)?

    let create_post = ServerAction::<api::CreatePost>::new();

    let posts_res = Resource::new(
        move || create_post.version().get(),
        move |_| api::get_posts_from_thread(thread_id),
    );

    let post_list_view = move || {
        Suspend::new(async move {
            let posts = match posts_res.await {
                Ok(posts) => posts,
                Err(err) => {
                    logging::log!("{err:?} - {err}");
                    return Either::Left(view! { <p>"Posts couldn't be loaded!"</p> });
                }
            };
            let view = posts
                .into_iter()
                .map(|post| PostItem(PostItemProps { post }))
                .collect_view();
            Either::Right(ol().class("flex flex-col gap-2").child(view))
        })
    };

    // server-side error handling
    let error = move || {
        // will be None before first dispatch
        let Some(val) = create_post.value().get() else {
            return Either::Left(().into_view());
        };
        // Will be Ok if no errors occured
        let Err(e) = val else {
            return Either::Left(().into_view());
        };

        let msg = match e {
            ApiError::EmptyContent => "Post content cannot be empty!".into(),
            _ => format!("Error from server: {e}"),
        };

        let view = view! { <p class="text-lg font-bold text-red-700">{msg}</p> };
        Either::Right(view)
    };

    let (client_error, set_client_error) = signal("none".to_string());

    view! {
      // server-side errors
      {error}

      // client-side errors / validation
      {move || {
        if client_error() == "none" {
          Either::Left(().into_view())
        } else {
          let view = view! { <p class="text-lg font-bold text-red-700">{client_error()}</p> };
          Either::Right(view)
        }
      }}

      // https://flowbite.com/docs/forms/textarea/#comment-box
      <ActionForm
        action=create_post
        on:submit:capture=move |ev| {
          let post = api::CreatePost::from_event(&ev);
          let Ok(post) = post else {
            return;
          };
          if post.content.is_empty() {
            set_client_error.set("Post content cannot be empty!".to_string());
            ev.prevent_default();
          }
        }
        attr:class="mb-4 w-full max-w-md bg-gray-50 rounded-lg border border-gray-200"
      >
        // I hope there's a better way to do this...
        <input class="hidden" name="thread_id" value=thread_id />
        <textarea
          name="content"
          rows="5"
          placeholder="Write a post..."
          required
          on:input:target=move |ev| {
            if !ev.target().value().is_empty() {
              set_client_error.set("none".to_string());
            }
          }
          prop:value=move || { create_post.version().with(move |_| String::new()) }
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
      {post_list_view}
    }
}

/// Renders a list item with a box containing a single [`Post`]
#[component]
pub fn PostItem(post: Post) -> impl IntoView {
    view! {
      <li>
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
          <p class="mb-3 font-normal text-gray-700 whitespace-pre-wrap break-words">
            {post.content}
          </p>
        </article>
      </li>
    }
}
