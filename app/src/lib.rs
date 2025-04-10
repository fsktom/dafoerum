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
    components::{A, Outlet, ParentRoute, Route, Router, Routes},
    hooks::use_location,
    path,
};

pub trait TimeUtils {
    /// Pretty prints how long ago given date was from now
    fn ago(self) -> String;
}
impl TimeUtils for jiff::Timestamp {
    fn ago(self) -> String {
        let now = jiff::Timestamp::now();
        let diff = now - self;
        // oh damn i get it. different api from chrono but i like it
        // also it's natural i dont need to read the docs just follow RA/LSP
        // and read first sentence of method (e.g. tried get_minutes() but docs said i wanted total)
        let minutes = diff.total(jiff::Unit::Minute).expect("i dan faqd up");
        format!("{minutes:.1}")
    }
}

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
        <body class="overflow-y-scroll bg-purple-100 h-svh">
          <App />
        </body>
      </html>
    }
}

/// View for when you do trailing slash mofo
#[component]
pub fn Faq() -> impl IntoView {
    // because for some reason /latest/ etc otherwise cause db not found in context error
    // i could redirect to og page but difficutl for e.g. /forum/:id :))))
    view! { <p>"don't do this i hate you"</p> }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
      <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css" />
      <Title text="Dafoerum" />

      <Router>
        <header>
          <NavBar />
        </header>
        <main class="flex flex-col items-center py-8">
          <div class="flex flex-col gap-4 items-center max-w-4xl sm:items-center md:w-3/4 2xl:w-2/3 w-9/11 sm:w-8/10 lg:w-8/11 xl:w-7/10">
            <Routes fallback=|| "Page not found.".into_view()>
              <Route path=StaticSegment("") view=HomePage />

              <Route path=StaticSegment("/latest/") view=Faq />
              <Route path=StaticSegment("/latest") view=Latest />

              <ParentRoute path=StaticSegment("/forum") view=move || view! { <Outlet /> }>
                <Route path=StaticSegment("/") view=Faq />
                <Route path=path!(":id/") view=Faq />

                <Route path=StaticSegment("") view=forum::Forums />
                <Route path=path!(":id") view=forum::ForumOverview />
              </ParentRoute>

              <Route path=path!("/thread/:id/") view=Faq />
              <Route path=path!("/thread/:id") view=forum::thread::ThreadOverview />
            </Routes>
          </div>
        </main>
      </Router>
    }
}

/// Used for e.g. highlighting a link if you're on a specific page
pub enum MatchPath {
    /// `/{path}` and `{path}`
    Full(&'static str),
    /// `/{path}*` amd `{path}*`
    Start(&'static str),
}
impl MatchPath {
    /// Checks if [`self`] matches the given `path`
    ///
    /// # Example
    ///
    /// ```
    /// use app::MatchPath;
    ///
    /// assert!(MatchPath::Full("forum").matches("/forum"));
    /// assert!(!MatchPath::Full("forum").matches("/forum/3"));
    ///
    /// assert!(MatchPath::Start("forum").matches("/forum"));
    /// assert!(MatchPath::Start("forum").matches("/forum/3"));
    ///
    /// ```
    pub fn matches(&self, path: &str) -> bool {
        match self {
            Self::Full(p) => path == *p || format!("/{p}") == path,
            Self::Start(p) => path.starts_with(p) || path.starts_with(&format!("/{p}")),
        }
    }
}

/// Renders the top navigation bar
#[component]
fn NavBar() -> impl IntoView {
    let path = use_location().pathname;

    view! {
      // hide on mobile - TBD: mobile navbar hamburger
      <nav class="hidden justify-around w-full h-20 bg-purple-700 sm:flex shadow-[0_3px_0_theme(colors.purple.300)]">
        // solid light purple "shadow" to seperate nav from main
        <div class="hidden justify-center items-center w-20 text-lg font-bold text-purple-50 uppercase md:flex md:text-xl lg:w-40 lg:text-2xl font-display md:w-30">
          "Dafoerum"
        </div>
        <div class="flex flex-wrap justify-between items-center p-4 max-w-screen-xl">
          <menu class="flex flex-row gap-5 font-medium rounded-lg border-0">
            <NavLink href="/" matching=&[MatchPath::Full("")] content="Home" pathname=path />
            <NavLink
              href="/forum"
              matching=&[MatchPath::Start("forum"), MatchPath::Start("thread")]
              content="Forums"
              pathname=path
            />
            <NavLink
              href="/latest"
              matching=&[MatchPath::Full("latest")]
              content="Latest Posts"
              pathname=path
            />
            <NavLink
              href="/wiki"
              matching=&[MatchPath::Start("wiki")]
              content="Wiki"
              pathname=path
            />
            <NavLink
              href="/profile"
              matching=&[MatchPath::Start("profile")]
              content="Profile"
              pathname=path
            />
          </menu>
        </div>
        <div class="hidden invisible md:block lg:w-40 md:w-30"></div>
      </nav>
    }
}

/// Renders a list element with link for the navigation bar that changes colors when you're on its page
#[component]
fn NavLink(
    /// Route to the page
    href: &'static str,
    /// Routes on which this link should be highlighted
    matching: &'static [MatchPath],
    /// The displayed link text
    content: &'static str,
    /// Gotten by [`use_location`]
    pathname: Memo<String>,
) -> impl IntoView {
    let is_current = move || matching.iter().any(|p| p.matches(&pathname()));

    // doing it witha NodeRef and checking aria-working only works on full reload, and not reactive for some reason...
    // also <A> doesn't work well with these conditional classes...

    view! {
      <li>
        <a
          href=href
          class="block py-2 px-4 text-lg rounded-sm hover:underline decoration-purple-50"
          class=(["text-purple-100"], move || !is_current())
          class=(["text-purple-50", "bg-purple-500"], move || is_current())
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
      // https://book.leptos.dev/view/03_components.html#spreading-attributes-onto-components
      <A
        href="forum"
        {..}
        class="flex justify-center items-center p-5 h-20 text-2xl font-bold text-purple-100 uppercase bg-purple-800 rounded-2xl hover:bg-purple-900 hover:cursor-pointer w-md"
      >
        "Go to the forum"
      </A>
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
                .map(|post| forum::thread::PostItem(forum::thread::PostItemProps { post }))
                .collect_view();
            Either::Right(ol().class("flex flex-col gap-2").child(view))
        })
    };

    // https://flowbite.com/icons/
    let refresh_icon_view = move || {
        view! {
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
        }
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
