use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use worker::*;

use crate::{get_response_json, not_found};

const POLL_JSON: &str = include_str!("poll.json");

#[derive(Deserialize)]
struct CurrentPoll {
    #[serde(rename = "pollName")]
    poll_name: String,
    options: Vec<String>,
}

#[derive(Deserialize)]
struct VoteBody {
    #[serde(rename = "votedFor")]
    voted_for: i64,
}

#[derive(serde::Deserialize)]
struct PollRow {
    #[serde(rename = "PollName")]
    poll_name: String,
    #[serde(rename = "JsonData")]
    json_data: String,
}

#[derive(serde::Deserialize)]
struct PollDataRow {
    #[serde(rename = "JsonData")]
    json_data: String,
}

pub async fn handle_poll_request(req: Request, env: Env) -> Result<Response> {
    let url: Url = req.url()?;
    let pathname: String = url.path().replace("/poll", "");

    let mut headers: Headers = Headers::new();
    headers.set("Content-Type", "application/json")?;
    headers.set("Access-Control-Allow-Origin", "*")?;
    headers.set("Access-Control-Allow-Methods", "GET, OPTIONS")?;
    headers.set("Access-Control-Allow-Headers", "Content-Type")?;

    match pathname.as_str() {
        "/current" => {
            let current: Value = serde_json::from_str(POLL_JSON)
                .map_err(|e| Error::RustError(format!("bad poll.json: {e}")))?;
            Ok(Response::from_json(&current)?.with_headers(headers))
        }

        "/upload" => handle_poll_upload(req, env).await,

        "/fetch" => {
            let mut out = serde_json::Map::new();
            let db = env.d1("DB")?;

            match db.prepare("SELECT * FROM Polls").all().await {
                Ok(result) => {
                    if let Ok(rows) = result.results::<PollRow>() {
                        for row in rows {
                            if let Ok(parsed) = serde_json::from_str::<Value>(&row.json_data) {
                                out.insert(row.poll_name, parsed);
                            }
                        }
                    }
                }
                Err(_) => {
                    return get_response_json(500, Some("Couldn't find polls."), HashMap::new())
                }
            }

            Ok(Response::from_json(&Value::Object(out))?.with_headers(headers))
        }

        _ => not_found(),
    }
}

async fn handle_poll_upload(mut req: Request, env: Env) -> Result<Response> {
    if req.method() == Method::Options {
        let mut headers = HashMap::new();
        headers.insert(
            "Access-Control-Allow-Methods".into(),
            "POST, OPTIONS".into(),
        );
        return get_response_json(204, None, headers);
    }

    if req.method() != Method::Post {
        return get_response_json(
            405,
            Some("You can only send POST requests to this URL"),
            HashMap::new(),
        );
    }

    let body: VoteBody = match req.json().await {
        Ok(b) => b,
        Err(e) => {
            return get_response_json(
                400,
                Some(&format!("Failed to read request body. Make sure you are sending a valid JSON body. Error details: {e}")),
                HashMap::new(),
            );
        }
    };

    let current: CurrentPoll = serde_json::from_str(POLL_JSON)
        .map_err(|e| Error::RustError(format!("bad poll.json: {e}")))?;

    if body.voted_for < 0 || body.voted_for as usize >= current.options.len() {
        return get_response_json(
            400,
            Some(
                "Correct request body but 'votedFor' was out of the bounds of the 'options' array.",
            ),
            HashMap::new(),
        );
    }

    let db: D1Database = env.d1("DB")?;
    let row: Option<PollDataRow> = db
        .prepare("SELECT JsonData FROM Polls WHERE PollName = ?")
        .bind(&[current.poll_name.clone().into()])?
        .first(None)
        .await?;

    let mut poll_data: Value = match row {
        Some(r) => serde_json::from_str(&r.json_data).unwrap_or_else(|_| json!({ "votes": [] })),
        None => json!({ "votes": [] }),
    };

    if !poll_data["votes"].is_array() {
        poll_data["votes"] = json!([]);
    }
    poll_data["votes"]
        .as_array_mut()
        .unwrap()
        .push(json!(body.voted_for));

    db.prepare(
        "INSERT INTO Polls(PollName, JsonData) VALUES (?, ?) ON CONFLICT(PollName) DO UPDATE SET JsonData = excluded.JsonData",
    )
        .bind(&[current.poll_name.clone().into(), poll_data.to_string().into()])?
        .run()
        .await?;

    get_response_json(200, Some("Successfully voted."), HashMap::new())
}
