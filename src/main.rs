use axum::{routing::get, Router};
use time::UtcOffset;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, fmt, EnvFilter};

#[tokio::main]
async fn main() ->anyhow::Result<()>{
    // UTC timestamp
    let offset = UtcOffset::UTC;

    // initialise tracing
    tracing_subscriber::registry()
    .with(
        fmt::layer()
            .json()
            .flatten_event(true)
            .with_timer(fmt::time::OffsetTime::new(
                offset,
                time::format_description::well_known::Rfc3339,
            ))
            .with_level(true)
            .with_target(false)
            .with_thread_ids(false)
            .with_thread_names(false),
    )
    .with(EnvFilter::from_default_env())
    .try_init()?;

    let app = Router::new().route("/", get(|| async { 
        info!(
            msg = "This is a log message"
        );
        "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
