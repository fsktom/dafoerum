use crate::TimeUtils;
use crate::api;
use api::{ApiError, Category, Forum, Post, Thread};

use leptos::either::{Either, EitherOf3};
use leptos::html::ol;
use leptos::{logging, prelude::*};
use leptos_meta::Title;
use leptos_router::{
    components::A,
    hooks::{use_navigate, use_params},
    params::Params,
};

/// Renders a list of all [`Forums`][Forum]
#[component]
pub fn Forums() -> impl IntoView {
    let categories_res: Resource<Result<Vec<Category>, ApiError>> =
        Resource::new(move || (), move |()| api::get_categories());

    let category_list_view = move || {
        Suspend::new(async move {
            let categories = match categories_res.await {
                Ok(categories) => categories,
                Err(err) => {
                    logging::log!("{err:?} - {err}");
                    return Either::Left(view! { <p>"Forums couldn't be loaded!"</p> });
                }
            };

            let view = categories
                .into_iter()
                .map(|category| CategoryItem(CategoryItemProps { category }))
                .collect_view();
            Either::Right(view)
        })
    };

    view! {
      <Title text="Forums | Dafoerum" />
      <Suspense fallback=move || {
        view! { <p>"Loading forums..."</p> }
      }>
        // NotFoundError: Failed to execute 'insertBefore' on 'Node': The node before which the new node is to be inserted is not a child of this node.
        // when using <For /> and navigating to this page from any other
        // see https://github.com/leptos-rs/leptos/issues/3385
        {category_list_view}
      </Suspense>
    }
}

/// Renders a single forum category with its forums as a table
#[component]
fn CategoryItem(category: Category) -> impl IntoView {
    view! {
      <section class="p-4 mb-2 bg-purple-200 shadow-[0_3px_0_theme(colors.purple.300)] rounded-xs w-9/10 sm:8/10">
        <h2 class="text-2xl font-bold font-display text-purple-950">{category.name.clone()}</h2>
        <table class="w-full table-fixed">
          <thead>
            <tr>
              <th scope="col" class="w-20">
                "Forum"
              </th>
              <th scope="col" class="w-40">
                "Last activity"
              </th>
              <th scope="col" class="w-15">
                "#"
              </th>
            </tr>
          </thead>
          <tbody>
            {category
              .forums
              .into_iter()
              .map(|forum: Forum| ForumRow(ForumRowProps { forum }))
              .collect_view()}
          </tbody>
        </table>
      </section>
    }
}

/// Renders a table row containing info on a [`Forum`]
#[component]
fn ForumRow(forum: Forum) -> impl IntoView {
    let latest_thread_res = Resource::new(
        move || (),
        move |()| api::get_latest_post_and_thread(forum.latest_thread_id),
    );
    let thread_n_post_count_res = Resource::new(
        move || (),
        move |()| api::count_threads_and_posts_of_forum(forum.id),
    );

    let latest_thread_summary_view = move || {
        Suspend::new(async move {
            let result = latest_thread_res;
            let (post, thread) = match result.await {
                Ok(counts) => counts,
                Err(err) => {
                    logging::log!("{err:?} - {err}");
                    return Either::Left(view! { <p>"Thread couldn't be loaded!"</p> });
                }
            };

            let view = view! {
              <A
                href=format!("/thread/{}", thread.id)
                {..}
                class="block overflow-hidden w-full underline whitespace-nowrap hover:no-underline overflow-ellipsis"
              >
                {thread.subject}
              </A>
              <p>
                "Last post "
                <time datetime=post
                  .created_at
                  .to_string()>{post.created_at.ago()}" minutes ago"</time>
              </p>
            };
            Either::Right(view)
        })
    };

    let counts_view = move || {
        Suspend::new(async move {
            let (thread_count, post_count) = match thread_n_post_count_res.await {
                Ok(counts) => counts,
                Err(err) => {
                    logging::log!("{err:?} - {err}");
                    return Either::Left(view! { <p>{err.to_string()}</p> });
                }
            };

            let view = view! {
              <p>"Threads: "<span class="font-medium">{thread_count}</span></p>
              <p>"Posts: "<span class="font-medium">{post_count}</span></p>
            };
            Either::Right(view)
        })
    };

    view! {
      <tr class="text-purple-900 not-last:border-dotted not-last:border-purple-300 not-last:border-b-4">
        <th scope="row" class="text-lg">
          <A href=forum.id.to_string() {..} class="font-bold underline hover:no-underline">
            {forum.name}
          </A>
        </th>

        <td class="py-2 leading-5">
          <Suspense>{latest_thread_summary_view}</Suspense>
        </td>
        <td class="py-2 leading-5">
          <Suspense fallback=move || "\u{2026}".into_view()>{counts_view}</Suspense>
        </td>
      </tr>
    }
}

/// Parameters for /forum/:id
#[derive(Params, PartialEq, Clone, Copy)]
struct ForumParams {
    id: u32,
}

/// Renders the thread list of a [`Forum`]
#[component]
pub fn ForumOverview() -> impl IntoView {
    let title_format = |text| format!("{text} - Forums | Dafoerum");
    let params = use_params::<ForumParams>();
    let Ok(ForumParams { id }) = params.get_untracked() else {
        return Either::Left(view! {
          <h2 class="text-4xl font-bold">"Invalid id!"</h2>
          <a href="/" class="block rounded-sm hover:text-blue-700">
            <h3 class="text-3xl font-bold">"Go to the frontpage"</h3>
          </a>
        });
    };

    let (error, set_error) = signal::<Option<ApiError>>(None);

    let forum_res = Resource::new(move || (), move |()| api::get_forum(id));
    let forum_head_view = move || {
        let Some(forum_res) = forum_res.get() else {
            // necessary check bc <Suspense/> will render children once before resource is loaded
            return EitherOf3::A(view! { <p>"initial"</p> });
        };
        let (forum, category_name) = match forum_res {
            Ok(forum) => {
                set_error(None);
                forum
            }
            Err(err) => {
                logging::log!("{err:?} - {err}");
                set_error(Some(err));
                return EitherOf3::B(().into_view());
            }
        };
        EitherOf3::C(view! {
          <Title text=forum.name.to_string() formatter=title_format />
          <nav class="self-start">
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
          </nav>
          <h2 class="text-4xl font-bold">{forum.name}</h2>
          <p>"Forum id: "{forum.id}</p>
        })
    };

    let errored_view = move || {
        let Some(api_error) = error() else {
            return Either::Left(view! { <p>"This shouldn't happen!"</p> });
        };

        let msg = match api_error {
            ApiError::NotFound(_, id) => {
                format!("There exists no forum with the id {id}")
            }
            _ => format!("Error from server: {api_error}"),
        };

        Either::Right(view! { <p class="text-lg font-bold text-red-700">{msg}</p> })
    };
    let waiting_view = move || {
        view! { <p>"Loading the forum..."</p> }
    };

    // for future reference: (still not sure though)
    // Suspense will wait for resources read synchronously, i.e. in a blocking way, and show fallback
    // if you use Suspend it will load it in the background, but still display the page
    // (unless a synchronous element blocks it)
    // so, synchronous for elements that MUST appear
    // asynchronous for elements that can lazily load in the background?
    // and.. for the above to work, Suspense only works on resources in its DIRECT children?
    // edit: Tbh IM NOT FUCKING SURE. Suspend::new() stuff also makes it fall back in other stuff
    // idfk im too stoopid (see) ThreadOverview suspense also waiting for <Posts /> to load
    Either::Right(view! {
      <Suspense fallback=waiting_view>
        {forum_head_view} <Show when=move || error().is_none() fallback=errored_view>
          <Threads forum_id=id />
        </Show>
      </Suspense>
    })
}

/// Renders a list of all [`Threads`][Thread] of a given [`Forum`]
#[component]
pub fn Threads(forum_id: u32) -> impl IntoView {
    let create_thread = ServerAction::<api::CreateThread>::new();
    let threads_res: Resource<Result<Vec<Thread>, ApiError>> =
        Resource::new(move || (), move |()| api::get_threads(forum_id));

    let (error, set_error) = signal::<Option<ApiError>>(None);

    // redirect to created thread on thread creation
    Effect::new(move |_| {
        let Some(result) = create_thread.value().get() else {
            return;
        };
        if let Ok(thread_id) = result {
            let navigate = use_navigate();
            let url = format!("/thread/{thread_id}");
            navigate(&url, leptos_router::NavigateOptions::default());
        }
    });

    let thread_list_view = move || {
        Suspend::new(async move {
            let threads = match threads_res.await {
                Ok(threads) => {
                    set_error(None);
                    threads
                }
                Err(err) => {
                    logging::log!("{err:?} - {err}");
                    set_error(Some(err));
                    return Either::Left(().into_view());
                }
            };
            Either::Right(
                threads
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
                    .collect_view(),
            )
        })
    };

    // server-side error handling
    let form_errored_view = move || {
        // will be None before first dispatch
        let Some(val) = create_thread.value().get() else {
            return Either::Left(().into_view());
        };
        // Will be Ok if no errors occured
        let Err(e) = val else {
            return Either::Left(().into_view());
        };

        logging::log!("{e:?} - {e}");

        let msg = match e {
            ApiError::EmptyContent => "Post content cannot be empty!".into(),
            ApiError::EmptySubject => "Subject cannot be empty!".into(),
            _ => format!("Error from server: {e}"),
        };

        Either::Right(view! { <p class="text-lg font-bold text-red-700">{msg}</p> })
    };
    let waiting_view = move || {
        view! { <p>"Loading the threads..."</p> }
    };

    view! {
      <Suspense fallback=waiting_view>
        {form_errored_view} <Show when=move || error().is_some()>
          <p>"big error"</p>
        </Show>
        <ActionForm
          action=create_thread
          attr:class="mb-4 w-full max-w-md bg-gray-50 rounded-lg border border-gray-200"
        >
          // I hope there's a better way to do this...
          <input class="hidden" name="forum_id" value=forum_id />
          <input
            name="subject"
            placeholder="Write a subject..."
            class="block p-2.5 w-full text-sm text-gray-900 bg-gray-50 rounded-lg border border-gray-300 focus:border-blue-500 focus:ring-blue-500"
          />
          <textarea
            name="post_content"
            rows="5"
            placeholder="Write a post..."
            class="py-2 px-4 w-full text-sm text-gray-900 bg-white rounded-t-lg border-0 focus:ring-0 placeholder:italic"
          ></textarea>
          <div class="flex justify-between items-center py-2 px-3 border-t border-gray-200">
            <input
              type="submit"
              value="Create Thread"
              class="inline-flex items-center py-2.5 px-4 text-xs font-medium text-center text-white bg-blue-700 rounded-lg hover:bg-blue-800 focus:ring-4 focus:ring-blue-200"
            />
          </div>
        </ActionForm> <ol class="mb-2 text-lg font-semibold text-gray-900">{thread_list_view}</ol>
      </Suspense>
    }
}

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
