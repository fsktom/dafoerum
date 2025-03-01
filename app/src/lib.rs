use jiff::ToSpan;
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
      <Post />
    }
}

#[component]
fn Post() -> impl IntoView {
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
    let on_click = move |_| *date.write() += 1.day();

    let display_date = move || format!("{} - {:#?}", date(), date().weekday());

    view! {
      <article class="flex flex-col p-4 w-1/3 bg-amber-200">
        <h3 class="self-start">{display_date}</h3>
        <button
          class="p-1 text-gray-400 bg-amber-300 cursor-pointer hover:bg-amber-400"
          on:click=on_click
        >
          "Increase date"
        </button>
        <p>
          "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Quisque a urna vel purus feugiat ultrices in in ipsum. Vestibulum sollicitudin pretium arcu, elementum ultrices erat sollicitudin ac. Morbi ornare lectus ut scelerisque porttitor. Curabitur faucibus nulla non ipsum ultricies interdum. Vestibulum dapibus enim ante, id ullamcorper ex placerat a. Vestibulum volutpat dui id dapibus aliquam. Sed facilisis ullamcorper mi eget fermentum."
        </p>
      </article>
    }
}
