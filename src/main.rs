use std::{iter::once, time::Duration};

use axum::{body::Body, extract::Request, http::{HeaderName, Response}, routing::get, Router};
use time::UtcOffset;
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer}, sensitive_headers::SetSensitiveRequestHeadersLayer, trace::TraceLayer
};
use tracing::{Span, info};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

use crate::utils::get_request_id;

mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
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

    // APP
    let app = Router::new()
        .route(
            "/",
            get(|| async {
                info!(msg = "This is a log message");
                "Hello, World!"
            }),
        )
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http().on_request(|req: &Request<Body>, _span: &Span| {
                let headers = req
                    .headers()
                    .iter()
                    .filter(|(k, _)| k.as_str().to_ascii_lowercase() != "x-request-id")
                    .map(|(k, v)| {
                        let val = if v.is_sensitive() {
                            "******"
                        } else {
                            v.to_str().unwrap_or("<non-utf8>")
                        };
                        format!("{}: {}", k.as_str(), val)
                    })
                    .collect::<Vec<_>>()
                    .join("; ");

                info!(
                    msg = "Request initiated",
                    req_id = %get_request_id(req.extensions()),
                    method = %req.method(),
                    uri = %req.uri(),
                    headers = %headers
                )
            })// </on request>
            .on_response(|res: &Response<Body>, latency: Duration, _span: &Span| {
                info!(
                    msg = "Request processed",
                    req_id = %get_request_id(res.extensions()),
                    status = %res.status().as_u16(),
                    latency = ?latency
                )
            }),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid::default()))
        .layer(SetSensitiveRequestHeadersLayer::new(once(
            HeaderName::from_static("access-token"),
        )));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    Ok(())
}
