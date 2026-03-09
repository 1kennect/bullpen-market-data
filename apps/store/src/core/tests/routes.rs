//! Routes that answer before the database: the token, scope keys, and bounds.

use axum::http::StatusCode;
use secrecy::SecretString;
use serde_json::json;

use super::{app, send, send_as};
use crate::db::Db;
use crate::routes::equal;

/// A router whose pool never connects; any request that reaches the database fails.
fn offline() -> axum::Router {
    let url = SecretString::from("postgres://nobody@127.0.0.1:1/none");
    match Db::lazy(&url, std::time::Duration::from_secs(1)) {
        Ok(db) => app(db),
        Err(e) => unreachable!("a lazy pool builds: {e}"),
    }
}

const BASE: &str = "/api/v1/tenants/guild/users/111";

#[tokio::test]
async fn every_api_route_needs_the_token() {
    let app = offline();
    let path = format!("{BASE}/memories?limit=5");
    let (status, body) = send_as(&app, "GET", &path, None, None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["error"], "missing or invalid bearer token");
    let (status, _) = send_as(&app, "GET", &path, None, Some("wrong-token-0123456789")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn health_needs_no_token_and_reports_the_database() {
    let (status, body) = send_as(&offline(), "GET", "/health", None, None).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["status"], "unavailable");
}

#[tokio::test]
async fn a_limit_outside_the_bounds_is_rejected() {
    let app = offline();
    for limit in ["0", "1001"] {
        let (status, body) =
            send(&app, "GET", &format!("{BASE}/memories?limit={limit}"), None).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{limit}");
        assert!(body["error"].as_str().is_some_and(|e| e.contains("limit")));
    }
}

#[tokio::test]
async fn an_unknown_memory_kind_is_rejected() {
    let path = format!("{BASE}/memories?limit=5&kinds=episodic,dream");
    let (status, body) = send(&offline(), "GET", &path, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["error"], "memory kind dream is unknown");
}

#[tokio::test]
async fn a_scope_key_past_the_bound_is_rejected() {
    let long = "u".repeat(201);
    let path = format!("/api/v1/tenants/guild/users/{long}/memories?limit=5");
    let (status, body) = send(&offline(), "GET", &path, None).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"].as_str().is_some_and(|e| e.contains("user")));
}

#[tokio::test]
async fn a_ttl_past_the_bound_is_rejected() {
    let path = format!("{BASE}/pending/{}", uuid::Uuid::nil());
    let body = json!({"action": {}, "payload_hash": "h", "ttl_seconds": 86_401});
    let (status, body) = send(&offline(), "PUT", &path, Some(body)).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(
        body["error"]
            .as_str()
            .is_some_and(|e| e.contains("ttl_seconds"))
    );
}

#[tokio::test]
async fn a_fact_with_confidence_past_one_is_rejected() {
    let fact = json!({
        "subject": {"kind": "person", "label": "me"},
        "relation": "holds",
        "object": {"kind": "security", "label": "AAPL"},
        "confidence": 1.5,
    });
    let path = format!("{BASE}/profile/facts");
    let (status, _) = send(&offline(), "POST", &path, Some(json!({"facts": [fact]}))).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn a_database_failure_hides_its_cause() {
    let (status, body) = send(&offline(), "GET", &format!("{BASE}/memories?limit=5"), None).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["error"], "store database failed");
}

#[test]
fn token_comparison_needs_equal_length_and_bytes() {
    assert!(equal(b"abc", b"abc"));
    assert!(!equal(b"abc", b"abd"));
    assert!(!equal(b"abc", b"abcd"));
    assert!(!equal(b"", b"a"));
}
