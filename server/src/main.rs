use axum::Router;
use axum::extract::FromRef;
use leptos::prelude::*;
use leptos_axum::{LeptosRoutes, generate_route_list};
use tracing_subscriber::filter::{EnvFilter, LevelFilter};

use mongodb::Client;

#[derive(FromRef, Debug, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    // pub db: Database,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let env = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .from_env_lossy();
    tracing_subscriber::fmt().with_env_filter(env).init();

    let conf = get_configuration(None)?;
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;

    let routes = generate_route_list(app::App);

    dotenvy::dotenv()?;

    let mongo_uri = std::env::var("MONGO_DB_URI")?;
    let mongo_client = Client::with_uri_str(mongo_uri).await?;
    let db = mongo_client.database("forum");

    let state = AppState { leptos_options };

    let app = Router::new()
        .leptos_routes_with_context(&state, routes, move || provide_context(db.clone()), {
            let opts = state.clone().leptos_options;
            move || app::shell(opts.clone())
        })
        .fallback(leptos_axum::file_and_error_handler::<AppState, _>(
            app::shell,
        ))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!("listening on http://localhost:{}", addr.port());
    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
