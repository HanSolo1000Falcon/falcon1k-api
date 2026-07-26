mod poll_manager;

use axum::http;
use http::status::StatusCode;
use std::collections::HashMap;
use worker::*;

const FSYSUTILS_INSTALL_SCRIPT: &str = include_str!("fsysutils.sh");

#[event(fetch)]
async fn main(req: Request, env: Env, ctx: Context) -> Result<Response> {
    let url: Url = req.url().expect("Missing request URL");
    let url_path: &str = url.path();
    if url_path == "/" || url_path == "" {
        get_response_json(406, Some("This API is not meant to be viewed in a browser."), HashMap::new())
    } else if url_path.starts_with("/poll") {
      poll_manager::handle_poll_request(req, env).await
    } else if url_path == "/fsysutils" || url_path == "/fsysutils/" {
        Response::ok(FSYSUTILS_INSTALL_SCRIPT)
    } else {
        not_found()
    }
}

pub fn get_response_json(
    status: u16,
    detailed: Option<&str>,
    extra_headers: HashMap<String, String>,
) -> Result<Response> {
    let headers: Headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;

    for (k, v) in extra_headers {
        headers.set(&k, &v)?;
    }

    let body: ResponseBody = match detailed {
        Some(msg) => ResponseBody::Body(serde_json::json!({
            "status": status,
            "message": StatusCode::from_u16(status).ok().and_then(|s| s.canonical_reason()).unwrap_or("Unknown"),
            "detailed": msg,
        }).to_string().into_bytes()),
        None => ResponseBody::Empty
    };

    Ok(Response::from_body(body)?
        .with_status(status)
        .with_headers(headers))
}

pub fn not_found() -> Result<Response> {
    get_response_json(404, Some("Requested resource not found"), HashMap::new())
}
