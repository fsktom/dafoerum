mod api;
mod forum;

use jiff::ToSpan;
use leptos::{logging, prelude::*, task::spawn_local};
use leptos_meta::{MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    StaticSegment,
    components::{Route, Router, Routes},
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
        <body class="flex flex-col justify-evenly items-center p-6 w-full text-center bg-slate-100">
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
      <Title text="Welcome to Leptos" />

      // content for this welcome page
      <Router>
        <main class="flex flex-col gap-10 items-center">
          <Routes fallback=|| "Page not found.".into_view()>
            <Route path=StaticSegment("") view=HomePage />
            <Route path=path!("/posts") view=Posts />
          </Routes>
        </main>
      </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
      <h1 class="text-2xl font-bold text-red-400">"Welcome to Leptos!"</h1>
      <button
        class="p-2 text-red-100 bg-blue-500 border-2 border-blue-600 shadow cursor-pointer"
        on:click=on_click
      >
        "Click Me: "
        {count}
      </button>
      <a href="/posts">"See Posts"</a>
      <ArticleDate id=1 />
    }
}

#[component]
fn Posts() -> impl IntoView {
    let create_post = ServerAction::<api::CreatePost>::new();

    let psts = Resource::new(move || create_post.version().get(), |_| api::get_db_posts());

    let (comment, set_comment) = signal(String::new());

    let postss = move || {
        Suspend::new(async move {
            let posts = psts.await.unwrap();
            posts
                .into_iter()
                .map(|p| {
                    view! {
                      <li>
                        <forum::PostItem post=p />
                      </li>
                    }
                })
                .collect_view()
        })
    };

    view! {
      <Title text="Them Posts" />
      <ol class="flex flex-col gap-2">{postss}</ol>
      // https://flowbite.com/docs/forms/textarea/#comment-box
      <ActionForm
        action=create_post
        attr:class="w-full mb-4 border border-gray-200 rounded-lg bg-gray-50"
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
            class="inline-flex items-center py-2.5 px-4 text-xs font-medium text-center text-white bg-blue-700 rounded-lg hover:bg-blue-800 focus:ring-4 focus:ring-blue-200 dark:focus:ring-blue-900"
          />
        </div>
      </ActionForm>
    }
}

#[component]
fn ArticleDate(id: usize) -> impl IntoView {
    let now = jiff::Timestamp::now();
    let date = jiff::tz::db()
        .get("Europe/Berlin")
        .unwrap()
        .to_datetime(now)
        .with()
        .subsec_nanosecond(0)
        .build()
        .unwrap();

    let date = RwSignal::new(date);
    let on_click = move |_| {
        *date.write() += 1.day();
        let w = date.get().to_string();
        spawn_local(async {
            let Ok(a) = gay(w).await else { unreachable!() };
            logging::log!("client only {a}")
        });
    };

    let display_date = move || format!("{} - {:#?}", date(), date().weekday());

    logging::log!("post with {id}");

    view! {
      <article class="flex flex-col p-4 w-1/3 bg-amber-200">
        <h3 class="self-start">{display_date}</h3>
        <button
          class="p-1 text-gray-400 bg-amber-300 cursor-pointer hover:bg-amber-400"
          on:click=on_click
        >
          "Increase date"
        </button>
        <p>"Post #"{id}</p>
        <p>
          "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Quisque a urna vel purus feugiat ultrices in in ipsum. Vestibulum sollicitudin pretium arcu, elementum ultrices erat sollicitudin ac. Morbi ornare lectus ut scelerisque porttitor. Curabitur faucibus nulla non ipsum ultricies interdum. Vestibulum dapibus enim ante, id ullamcorper ex placerat a. Vestibulum volutpat dui id dapibus aliquam. Sed facilisis ullamcorper mi eget fermentum."
        </p>
      </article>
    }
}

#[server(endpoint = "gay")]
pub async fn gay(word: String) -> Result<String, ServerFnError> {
    logging::log!("{word} things happening only on server");
    Ok("WTF".to_string())
}
