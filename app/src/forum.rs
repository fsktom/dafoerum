pub mod thread;

use crate::TimeUtils;
use crate::api;
use api::{ApiError, Category, Forum, Post, Thread};

use leptos::either::{Either, EitherOf3};
use leptos::html::Dialog;
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
      <section
        id=clean_name_for_id(&category.name)
        class="p-4 mb-2 bg-purple-200 shadow-[0_3px_0_theme(colors.purple.300)] rounded-xs w-19/20 sm:8/10"
      >
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
          <A
            href=forum.id.to_string()
            {..}
            class="block overflow-hidden w-full font-bold underline whitespace-nowrap hover:no-underline overflow-ellipsis"
          >
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

    let create_thread_modal_ref = NodeRef::<Dialog>::new();

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
                // displaying of error handled in errored_view
                return EitherOf3::B(().into_view());
            }
        };
        EitherOf3::C(view! {
          <Title text=forum.name.to_string() formatter=title_format />
          <nav class="mb-2 w-full text-purple-900">
            <a href="/forum" class="font-medium underline hover:no-underline">
              "Forums"
            </a>
            " -> "
            // adding IDs and linking to it sucks in SPA - it will only scroll to it on page refresh
            <a
              href=format!("/forum#{}", clean_name_for_id(&category_name))
              class="font-medium underline hover:no-underline"
            >
              {category_name.to_string()}
            </a>
            " -> "
            <a href=format!("/forum/{}", forum.id) class="font-medium hover:underline">
              {forum.name.to_string()}
            </a>
          </nav>
          <div class="flex flex-wrap justify-between mb-2">
            <h1 class="text-3xl font-extrabold md:text-4xl lg:text-5xl text-purple-950 font-display">
              {forum.name}
            </h1>
            // https://developer.mozilla.org/en-US/docs/Web/HTML/Element/button#browser_compatibility
            // https://developer.mozilla.org/en-US/docs/Web/API/HTMLButtonElement/commandForElement
            // requires VERY recent browser (Chrome Apr 2025 +) and experiment enabled on firefox/safari
            // but I'm gonna require this because KEKW
            // it's very new and discovered it by accident while perusing button mdn
            // and was wondering why it didn't work (and why I couldn't google it easily)
            // on macos i stil lhave chrome 134 (and it requries 135 to work xd)
            // and not much stuff online cuz new

            // better than using NodeRef .show_modal() with on:click
            // <button
            // commandfor="create-thread-modal"
            // command="show-modal"

            // do it when it's more supported and works properly or you updated chrome
            <button
              on:click=move |_| {
                create_thread_modal_ref.get().unwrap().show_modal().unwrap();
              }
              class="flex justify-center items-center py-1 px-2 text-sm font-bold text-purple-100 bg-purple-800 rounded-full sm:py-2 sm:px-4 md:py-3 md:px-6 md:text-lg lg:text-xl hover:bg-purple-900 hover:cursor-pointer sm:text-md"
            >
              "Create Thread"
            </button>
          </div>
          <p>"Here will come a short description of the forum some day"</p>
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
    let forum_id = id;
    let create_thread_modal_id = "create-thread-modal";
    Either::Right(view! {
      <Suspense fallback=waiting_view>
        <Show when=move || error().is_none() fallback=errored_view>
          <section class="p-4 bg-purple-200 w-19/20 rounded-xs sm:8/10">{forum_head_view}</section>
          <section class="p-4 bg-purple-200 w-19/20 rounded-xs sm:8/10">
            <CreateThreadModal id=create_thread_modal_id forum_id create_thread_modal_ref />
            <ThreadList forum_id />
          </section>
        </Show>
      </Suspense>
    })
}

/// Renders a modal of thread creation with the given `id`
/// (for `<button>` use with `commandfor` later)
///
/// Creates a [`Thread`] and its origin [`Post`]
///
/// Takes in a [`NodeRef`] to the dialog created in this component
/// but created earlier, so that it can be used by parent [`ForumOverview`]
#[component]
pub fn CreateThreadModal(
    id: &'static str,
    forum_id: u32,
    create_thread_modal_ref: NodeRef<Dialog>,
) -> impl IntoView {
    let create_thread = ServerAction::<api::CreateThread>::new();

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
            _ => e.to_string(),
        };

        let view = view! {
          <div class="flex flex-col justify-center items-center p-2 mb-4 text-lg font-bold text-red-700 bg-red-50 border-2 border-red-400">
            <p class="font-bold">"Error from server:"</p>
            <p>{msg}</p>
          </div>
        };
        Either::Right(view)
    };

    view! {
      <dialog
        node_ref=create_thread_modal_ref
        id=id
        // center, but more top on mobile bc of on-screen keyboard
        class="fixed left-1/2 top-1/3 p-4 text-purple-900 bg-purple-50 rounded-xl border-2 border-purple-200 -translate-x-1/2 -translate-y-1/3 sm:top-1/2 sm:p-8 sm:-translate-y-1/2 md:p-12 backdrop:backdrop-blur-[2px] w-sm md:w-md"
      >
        {form_errored_view}
        <ActionForm action=create_thread attr:class="w-full">
          <input class="hidden" name="forum_id" value=forum_id />
          <label class="font-medium">
            "Subject"
            <input
              name="subject"
              placeholder="Greatest thread ever"
              required
              class="p-2.5 mb-2 w-full text-sm font-normal bg-purple-100 rounded-lg border border-purple-400 placeholder:italic"
            />
          </label>
          <label class="font-medium">
            "Content"
            <textarea
              name="post_content"
              rows="5"
              placeholder="Type here using Markdown (soon\u{2122})..."
              required
              wrap="soft"
              class="py-2 px-4 mb-4 w-full text-sm font-normal bg-purple-100 rounded-lg border border-purple-400 placeholder:italic"
            ></textarea>
          </label>
          <input
            type="submit"
            value="Create Thread"
            class="flex justify-center items-center py-1 mb-2 w-full font-bold text-purple-100 bg-purple-800 rounded-lg sm:py-2 sm:text-lg md:text-xl hover:bg-purple-900 hover:cursor-pointer text-md"
          />
        </ActionForm>
        // <button commandfor="create-thread-modal" command="close" class="p-2 bg-purple-100">
        <form method="dialog">
          <input
            type="submit"
            value="Cancel"
            class="flex justify-center items-center py-1 mb-2 w-full font-bold text-red-50 bg-red-800 rounded-lg sm:py-2 sm:text-lg md:text-xl hover:bg-red-900 hover:cursor-pointer text-md"
          />
        </form>
      </dialog>
    }
}

/// Renders a list of all [`Threads`][Thread] of a given [`Forum`]
#[component]
pub fn ThreadList(forum_id: u32) -> impl IntoView {
    let threads_res = Resource::new(move || (), move |()| api::get_threads(forum_id));

    let (error, set_error) = signal::<Option<ApiError>>(None);

    let thread_list_view = move || {
        Suspend::new(async move {
            let mut threads = match threads_res.await {
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

            // show threads with more recent activity first
            threads.sort_unstable_by_key(|(_, _, post)| std::cmp::Reverse(post.created_at));

            let view = threads
                .into_iter()
                .map(|(thread, post_count, latest_post)| {
                    ThreadRow(ThreadRowProps {
                        thread,
                        post_count,
                        latest_post,
                    })
                })
                .collect_view();
            Either::Right(view)
        })
    };

    view! {
      <Show
        when=move || error().is_none()
        fallback=move || {
          view! {
            <p class="text-lg font-bold text-center text-red-700">
              "Threads couldn't be loaded due to an error: "
              {error().unwrap_or_default().to_string()}
            </p>
          }
        }
      >

        <table class="w-full table-fixed">
          <thead>
            <tr>
              <th scope="col" class="w-40">
                "Thread"
              </th>
              <th scope="col" class="w-20">
                "Last activity"
              </th>
              <th scope="col" class="w-10">
                "#"
              </th>
            </tr>
          </thead>
          <tbody>
            <Suspense fallback=move || {
              view! {
                <tr class="text-purple-900">
                  <th scope="row" colspan="3" class="text-2xl text-center animate-bounce">
                    "\u{2026}"
                  </th>
                </tr>
              }
            }>{thread_list_view}</Suspense>
          </tbody>
        </table>
      </Show>
    }
}

/// A table row representing a [`Thread`]
#[component]
fn ThreadRow(thread: Thread, post_count: u64, latest_post: Post) -> impl IntoView {
    view! {
      <tr class="text-center text-purple-900 not-last:border-dotted not-last:border-purple-300 not-last:border-b-4">
        <th scope="row" class="text-lg">
          <a
            href=format!("/thread/{}", thread.id)
            class="block overflow-hidden w-full font-bold underline whitespace-nowrap hover:no-underline overflow-ellipsis"
          >
            {thread.subject}
          </a>
        </th>

        <td class="py-2 leading-5 text-center">
          <time datetime=latest_post.created_at.to_string()>{latest_post.created_at.ago()}</time>
          " minutes ago"
        </td>
        <td class="py-2 leading-5 text-center">{post_count}</td>
      </tr>
    }
}

/// Cleans up a [`Category`] name or w/e to make it usable as the id of an HTML element
///
/// See <https://developer.mozilla.org/en-US/docs/Web/HTML/Global_attributes/id>
fn clean_name_for_id(name: &str) -> String {
    let mut new = String::new();
    for word in name.split_ascii_whitespace() {
        new.push_str(word);
    }

    new
}
