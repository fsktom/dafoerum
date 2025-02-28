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
        <header class="flex justify-around items-center p-6 w-full bg-slate-200">
          <h1>"gay"</h1>
        </header>
        <main>
          <Routes fallback=|| "Page not found.".into_view()>
            <Route path=StaticSegment("") view=HomePage />
          </Routes>
        </main>
        <footer></footer>
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
        class="p-2 text-red-100 bg-blue-500 border-2 border-blue-600 shadow"
        on:click=on_click
      >
        "Click Me: "
        {count}
      </button>
    }

    // (
    //     h1().class("text-xl")
    //         .class("text-red-500")
    //         .child("Welcome to Leptos!"),
    //     button()
    //         .on(ev::click, on_click)
    //         .child(("Click Me: ", count)),
    // )
}
