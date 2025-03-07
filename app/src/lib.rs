mod api;
mod forum;

use leptos::prelude::*;
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
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

      // sets the document title
      <Title text="Dafoerum" />

      // content for this welcome page
      <Router>
        <header class="flex justify-center">
          <NavBar />
        </header>
        <main class="flex flex-col gap-10 items-center">
          <Routes fallback=|| "Page not found.".into_view()>
            <Route path=StaticSegment("") view=HomePage />
          </Routes>
        </main>
      </Router>
    }
}

/// Renders the top navigation bar
#[component]
fn NavBar() -> impl IntoView {
    // https://flowbite.com/docs/components/navbar/
    view! {
      <nav class="bg-white border-gray-200">
        <div class="flex flex-wrap justify-between items-center p-4 mx-auto max-w-screen-xl">
          <div class="block w-auto">
            <ul class="flex flex-row space-x-8 font-medium bg-white rounded-lg border-0 border-gray-100">
              <li>
                <a href="/" class="block text-blue-700 rounded-sm">
                  "Home"
                </a>
              </li>
              <li>
                <a href="/profile" class="block text-gray-900 rounded-sm hover:text-blue-700">
                  "Profile"
                </a>
              </li>
            </ul>
          </div>
        </div>
      </nav>
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
