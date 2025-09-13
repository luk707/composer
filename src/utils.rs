use axum::http::Extensions;
use tower_http::request_id::RequestId;

pub fn get_request_id(req: &Extensions) -> String {
    req.get::<RequestId>()
        .map(request_id_to_string)
        .unwrap_or_default()
}

pub fn request_id_to_string(req_id: &RequestId) -> String {
    req_id
        .header_value()
        .to_str()
        .ok()
        .unwrap_or_default()
        .to_string()
}
