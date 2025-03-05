use app::*;
use axum::Router;
// use axum::extract::FromRef;
use leptos::logging::log;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};

// #[derive(FromRef, Debug, Clone)]
// pub struct AppState {
//     pub leptos_options: LeptosOptions,
// }

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(App);

    // let state = AppState { leptos_options };

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
