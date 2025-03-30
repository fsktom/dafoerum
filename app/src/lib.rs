//! Code shared across the frontend and backend of dafoerum
//!
//! Server-only code that has to be used by the front end e.g. using [`ServerActions`][ServerAction]
//! is gated under the `ssr` feature using `#[cfg(feature = "ssr")]`
//! or implicitly with the `#[server]` macro

#![allow(
    clippy::must_use_candidate,
    reason = "works badly with rust-analyzer and #[component]"
)]

pub mod api;
mod forum;

use leptos::either::Either;
use leptos::html::ol;
use leptos::logging;
use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
    hooks::use_location,
    path,
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
        <body class="overflow-y-scroll bg-purple-50 h-svh">
          <App />
        </body>
      </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    let title_format = |text| format!("{text} - Dafoerum");

    view! {
      <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css" />
      <Title text="Dafoerum" formatter=title_format />

      <Router>
        <header>
          <NavBar />
        </header>
        <main class="flex flex-col items-center py-8">
          <div class="flex flex-col gap-4 items-center sm:items-center sm:w-2/3 w-9/11">
            <Routes fallback=|| "Page not found.".into_view()>
              <Route path=StaticSegment("") view=HomePage />
              <Route path=StaticSegment("/latest") view=Latest />
              <Route path=path!("/forum/:id") view=forum::ForumOverview />
              <Route path=path!("/thread/:id") view=forum::ThreadOverview />
            </Routes>
          </div>
        </main>
      </Router>
    }
}

/// Renders the top navigation bar
#[component]
fn NavBar() -> impl IntoView {
    let path = use_location().pathname;

    view! {
      // hide on mobile - TBD: mobile navbar hamburger
      <nav class="hidden justify-center w-full h-20 bg-purple-700 sm:flex shadow-[0_3px_0_theme(colors.purple.300)]">
        // solid light purple "shadow" to seperate nav from main
        <div class="flex flex-wrap justify-between items-center p-4 max-w-screen-xl">
          <ul class="flex flex-row gap-5 font-medium rounded-lg border-0">
            <NavLink href="/" content="Home" pathname=path />
            <NavLink href="/latest" content="Latest Posts" pathname=path />
            <NavLink href="/profile" content="Profile" pathname=path />
          </ul>
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
          class="block py-2 px-4 text-lg rounded-sm hover:underline decoration-purple-50"
          class=(["text-purple-100"], move || pathname() != href)
          class=(["text-purple-50", "bg-purple-500"], move || pathname() == href)
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
      <h2 class="text-4xl font-bold">"Forums:"</h2>
      <forum::Forums />
    }
}

/// Renders a list of the most recently posted posts
#[component]
fn Latest() -> impl IntoView {
    const NUM_OF_POSTS_TO_FETCH: i64 = 10;
    let posts_res = Resource::new(
        move || (),
        |()| api::get_latest_posts(NUM_OF_POSTS_TO_FETCH),
    );

    let (is_loading, set_is_loading) = signal(true);

    let post_list_view = move || {
        Suspend::new(async move {
            let posts = match posts_res.await {
                Ok(posts) => posts,
                Err(err) => {
                    logging::log!("{err:?} - {err}");
                    return Either::Left(view! { <p>"Posts couldn't be loaded!"</p> });
                }
            };
            set_is_loading(false);
            let view = posts
                .into_iter()
                .map(|post| forum::PostItem(forum::PostItemProps { post }))
                .collect_view();
            Either::Right(ol().class("flex flex-col gap-2").child(view))
        })
    };

    // https://flowbite.com/icons/
    let refresh_icon_view = move || view! {
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
      <h1 class="text-4xl font-extrabold md:text-5xl">"Latest Posts"</h1>
      <button
        type="button"
        on:click=move |_| {
          set_is_loading(true);
          posts_res.refetch();
        }
        class="inline-flex justify-center items-center py-2.5 px-5 w-60 h-10 font-medium text-center text-purple-100 rounded-lg"
        // active:cursor-wait
        class=(
          ["bg-purple-800", "hover:bg-purple-900", "hover:cursor-pointer"],
          move || !is_loading(),
        )
        class=(["bg-purple-500", "cursor-not-allowed"], move || is_loading())
      >
        <Show when=move || !is_loading() fallback=move || "Loading...">
          {refresh_icon_view}
          "Check for new posts"
        </Show>
      </button>
      <ol class="flex flex-col gap-2">{post_list_view}</ol>
    }
}
