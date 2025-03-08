mod api;
mod forum;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
    hooks::use_location,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
      <!DOCTYPE html>
      <html lang="en">
        <head>
          <meta charset="utf-8" />
          <meta name="viewport" content="width=device-width, initial-scale=1" />
          <AutoReload options=options.clone() />
          <HydrationScripts options />
          <MetaTags />
        </head>
        <body>
          <App />
        </body>
      </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
      <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css" />
      <Title text="Dafoerum" />

      <Router>
        <header class="flex justify-center">
          <NavBar />
        </header>
        <main class="flex flex-col gap-5 items-center">
          <Routes fallback=|| "Page not found.".into_view()>
            <Route path=StaticSegment("") view=HomePage />
            <Route path=StaticSegment("/latest") view=Latest />
          </Routes>
        </main>
      </Router>
    }
}

/// Renders the top navigation bar
#[component]
fn NavBar() -> impl IntoView {
    let path = use_location().pathname;

    // https://flowbite.com/docs/components/navbar/
    view! {
      <nav class="bg-white border-gray-200">
        <div class="flex flex-wrap justify-between items-center p-4 mx-auto max-w-screen-xl">
          <div class="block w-auto">
            <ul class="flex flex-row space-x-8 font-medium bg-white rounded-lg border-0 border-gray-100">
              <NavLink href="/" content="Home" pathname=path />
              <NavLink href="/latest" content="Latest Posts" pathname=path />
              <NavLink href="/profile" content="Profile" pathname=path />
            </ul>
          </div>
        </div>
      </nav>
    }
}

/// Renders a list element with link for the navigation bar that changes colors when you're on its page
#[component]
fn NavLink(
    /// Route to the page
    href: &'static str,
    /// The displayed link text
    content: &'static str,
    /// Gotten by [`use_location`]
    pathname: Memo<String>,
) -> impl IntoView {
    view! {
      <li>
        <a
          href=href
          class="block rounded-sm hover:text-blue-700"
          class=("text-gray-900", move || pathname() != href)
          class=("text-blue-700", move || pathname() == href)
        >
          {content}
        </a>
      </li>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    view! {
      <h1 class="mb-4 text-4xl font-extrabold tracking-tight leading-none text-gray-900 md:text-5xl lg:text-6xl">
        "Welcome to Dafoerum!"
      </h1>
      <Posts />
    }
}

/// Renders a list of the most recently posted posts
#[component]
fn Latest() -> impl IntoView {
    const NUM_OF_POSTS_TO_FETCH: i64 = 10;
    let posts = Resource::new(move || (), |_| api::get_latest_posts(NUM_OF_POSTS_TO_FETCH));
    let post_list_view = move || {
        Suspend::new(async move {
            posts
                .await
                .unwrap()
                .into_iter()
                .map(|post| {
                    view! {
                      <li>
                        <forum::PostItem post />
                      </li>
                    }
                })
                .collect_view()
        })
    };

    // https://flowbite.com/icons/
    let refresh_icon = view! {
      <svg
        class="w-6 h-6 text-gray-800 dark:text-white"
        aria-hidden="true"
        xmlns="http://www.w3.org/2000/svg"
        width="24"
        height="24"
        fill="none"
        viewBox="0 0 24 24"
      >
        <path
          stroke="currentColor"
          stroke-linecap="round"
          stroke-linejoin="round"
          stroke-width="2"
          d="M17.651 7.65a7.131 7.131 0 0 0-12.68 3.15M18.001 4v4h-4m-7.652 8.35a7.13 7.13 0 0 0 12.68-3.15M6 20v-4h4"
        />
      </svg>
    };

    view! {
      <h2 class="text-4xl font-extrabold">"Latest Posts"</h2>
      <button
        type="button"
        on:click=move |_| posts.refetch()
        class="inline-flex items-center py-2.5 px-5 text-sm font-medium text-center text-white bg-blue-700 rounded-lg hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 focus:outline-none me-2"
      >
        {refresh_icon}
        "Check for new posts"
      </button>
      <ol class="flex flex-col gap-2">{post_list_view}</ol>
    }
}

#[component]
fn Posts() -> impl IntoView {
    let create_post = ServerAction::<api::CreatePost>::new();

    let posts = Resource::new(move || create_post.version().get(), |_| api::get_db_posts());

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
                        <forum::PostItem post />
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
