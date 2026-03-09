//! Every route against a real Postgres. Each test runs in a tenant of its own.

use std::time::Duration;

use axum::Router;
use axum::http::StatusCode;
use secrecy::SecretString;
use serde_json::{Value, json};
use uuid::Uuid;

use super::{app, send};
use crate::db::Db;

/// Names the Postgres the database tests use; unset skips them.
const URL_VAR: &str = "BULLPEN_STORE_TEST_DATABASE_URL";

/// A migrated router and the base path of a fresh tenant and user 111, or None without a database.
async fn live() -> Option<(Router, String)> {
    live_db().await.map(|(app, base, _, _)| (app, base))
}

/// What live returns, with the database and the tenant.
async fn live_db() -> Option<(Router, String, Db, String)> {
    let url = std::env::var(URL_VAR).ok()?;
    let db = match Db::connect(&SecretString::from(url), 4, Duration::from_secs(5)).await {
        Ok(db) => db,
        Err(e) => unreachable!("{URL_VAR} connects: {e}"),
    };
    if let Err(e) = db.migrate().await {
        unreachable!("migrations apply: {e}");
    }
    let tenant = Uuid::new_v4().to_string();
    Some((
        app(db.clone()),
        format!("/api/v1/tenants/{tenant}/users/111"),
        db,
        tenant,
    ))
}

/// A message as the engine would store it.
fn message(text: &str) -> Value {
    json!({"role": "user", "content": {"role": "user", "content": text}})
}

#[tokio::test]
async fn health_answers_when_the_database_does() {
    let Some((app, _)) = live().await else { return };
    let (status, body) = send(&app, "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

#[tokio::test]
async fn a_conversation_belongs_to_one_user_at_one_visibility() {
    let Some((app, base)) = live().await else {
        return;
    };
    let id = Uuid::new_v4();
    let path = format!("{base}/conversations/{id}");
    let public = json!({"channel_id": "chan", "visibility": "public"});

    let (status, body) = send(&app, "PUT", &path, Some(public.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["id"], json!(id));
    assert_eq!(
        send(&app, "PUT", &path, Some(public.clone())).await.0,
        StatusCode::OK
    );

    let private = json!({"channel_id": "chan", "visibility": "private"});
    assert_eq!(
        send(&app, "PUT", &path, Some(private)).await.0,
        StatusCode::CONFLICT
    );
    let other = path.replace("/users/111/", "/users/222/");
    assert_eq!(
        send(&app, "PUT", &other, Some(public)).await.0,
        StatusCode::CONFLICT
    );

    let owned = |vis: &str| format!("{path}?visibility={vis}");
    assert_eq!(
        send(&app, "GET", &owned("public"), None).await.1["owned"],
        true
    );
    assert_eq!(
        send(&app, "GET", &owned("private"), None).await.1["owned"],
        false
    );
    assert_eq!(
        send(&app, "GET", &path, None).await.0,
        StatusCode::BAD_REQUEST
    );
    let other_owned = format!("{other}?visibility=public");
    assert_eq!(
        send(&app, "GET", &other_owned, None).await.1["owned"],
        false
    );
}

#[tokio::test]
async fn history_is_the_newest_summary_then_the_messages_after_it() {
    let Some((app, base)) = live().await else {
        return;
    };
    let id = Uuid::new_v4();
    let path = format!("{base}/conversations/{id}");
    let ensure = json!({"channel_id": "chan", "visibility": "public"});
    send(&app, "PUT", &path, Some(ensure)).await;

    let messages = format!("{path}/messages");
    let turns = json!({"messages": [message("a"), message("b"), message("c")]});
    let (status, body) = send(&app, "POST", &messages, Some(turns)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["positions"], json!([1, 2, 3]));

    let summary = json!({"content": {"role": "summary", "content": "a and b"}, "covers": 2});
    let (status, body) = send(&app, "POST", &format!("{path}/summary"), Some(summary)).await;
    assert_eq!(status, StatusCode::CREATED);
    assert_eq!(body["position"], 4);
    send(
        &app,
        "POST",
        &messages,
        Some(json!({"messages": [message("d")]})),
    )
    .await;

    let (_, body) = send(&app, "GET", &format!("{messages}?limit=10"), None).await;
    let positions: Vec<i64> = body["messages"]
        .as_array()
        .map(|rows| rows.iter().filter_map(|r| r["position"].as_i64()).collect())
        .unwrap_or_default();
    assert_eq!(positions, vec![4, 3, 5]);
    assert_eq!(body["messages"][0]["role"], "summary");
    assert_eq!(body["messages"][1]["content"]["content"], "c");

    let (_, body) = send(&app, "GET", &format!("{messages}?limit=1"), None).await;
    assert_eq!(body["messages"].as_array().map(Vec::len), Some(2));

    let bad = json!({"content": {}, "covers": 9});
    assert_eq!(
        send(&app, "POST", &format!("{path}/summary"), Some(bad))
            .await
            .0,
        StatusCode::BAD_REQUEST
    );
}

#[tokio::test]
async fn messages_reach_only_an_owned_conversation() {
    let Some((app, base)) = live().await else {
        return;
    };
    let missing = format!("{base}/conversations/{}/messages", Uuid::new_v4());
    let (status, body) = send(&app, "GET", &format!("{missing}?limit=5"), None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["messages"], json!([]));
    let turns = json!({"messages": [message("a")]});
    assert_eq!(
        send(&app, "POST", &missing, Some(turns.clone())).await.0,
        StatusCode::NOT_FOUND
    );

    let id = Uuid::new_v4();
    let path = format!("{base}/conversations/{id}");
    send(
        &app,
        "PUT",
        &path,
        Some(json!({"channel_id": "c", "visibility": "public"})),
    )
    .await;
    let theirs = format!("{}/messages", path.replace("/users/111/", "/users/222/"));
    assert_eq!(
        send(&app, "POST", &theirs, Some(turns)).await.0,
        StatusCode::CONFLICT
    );
    assert_eq!(
        send(&app, "GET", &format!("{theirs}?limit=5"), None)
            .await
            .0,
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn latest_finds_the_open_conversation_and_end_closes_every_one() {
    let Some((app, base)) = live().await else {
        return;
    };
    let (first, second) = (Uuid::new_v4(), Uuid::new_v4());
    for id in [first, second] {
        let body = json!({"channel_id": "chan", "visibility": "public"});
        send(
            &app,
            "PUT",
            &format!("{base}/conversations/{id}"),
            Some(body),
        )
        .await;
    }
    let private = json!({"channel_id": "chan", "visibility": "private"});
    send(
        &app,
        "PUT",
        &format!("{base}/conversations/{}", Uuid::new_v4()),
        Some(private),
    )
    .await;

    let latest = format!("{base}/channels/chan/latest?visibility=public");
    assert_eq!(
        send(&app, "GET", &latest, None).await.1["id"],
        json!(second)
    );

    let (_, body) = send(&app, "POST", &format!("{base}/channels/chan/end"), None).await;
    assert_eq!(body["ended"], 3);
    assert_eq!(send(&app, "GET", &latest, None).await.1["id"], Value::Null);
}

#[tokio::test]
async fn a_pending_action_is_claimed_once_by_its_owner() {
    let Some((app, base)) = live().await else {
        return;
    };
    let token = Uuid::new_v4();
    let hold = format!("{base}/pending/{token}");
    let action = json!({"tool": "market_quote", "args": {"symbol": "AAPL"}});
    let body = json!({"action": action, "payload_hash": "h", "ttl_seconds": 60});
    assert_eq!(
        send(&app, "PUT", &hold, Some(body.clone())).await.0,
        StatusCode::CREATED
    );
    assert_eq!(
        send(&app, "PUT", &hold, Some(body)).await.0,
        StatusCode::CONFLICT
    );

    let claim = format!("{hold}/claim");
    let theirs = claim.replace("/users/111/", "/users/222/");
    let approved = json!({"approved": true});
    assert_eq!(
        send(&app, "POST", &theirs, Some(approved.clone())).await.0,
        StatusCode::NOT_FOUND
    );
    let (status, body) = send(&app, "POST", &claim, Some(approved.clone())).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["action"], action);
    assert_eq!(
        send(&app, "POST", &claim, Some(approved)).await.0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn a_lapsed_hold_cannot_be_claimed() {
    let Some((app, base)) = live().await else {
        return;
    };
    let hold = format!("{base}/pending/{}", Uuid::new_v4());
    let body = json!({"action": {}, "payload_hash": "h", "ttl_seconds": 1});
    send(&app, "PUT", &hold, Some(body)).await;
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let claim = json!({"approved": true});
    assert_eq!(
        send(&app, "POST", &format!("{hold}/claim"), Some(claim))
            .await
            .0,
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn the_profile_graph_merges_labels_and_forgets_on_request() {
    let Some((app, base)) = live().await else {
        return;
    };
    let fact = |object: &str, confidence: f64| {
        json!({
            "subject": {"kind": "person", "label": "me"},
            "relation": "watches",
            "object": {"kind": "security", "label": object},
            "confidence": confidence,
        })
    };
    let facts = format!("{base}/profile/facts");
    let body = json!({"facts": [fact("AAPL", 0.6), fact("aapl", 0.9), fact("MSFT", 0.5)]});
    assert_eq!(
        send(&app, "POST", &facts, Some(body)).await.0,
        StatusCode::OK
    );

    let (_, body) = send(&app, "GET", &format!("{base}/profile?limit=10"), None).await;
    let nodes = body["nodes"].as_array().cloned().unwrap_or_default();
    assert_eq!(nodes.len(), 3, "{nodes:?}");
    assert_eq!(body["relations"].as_array().map(Vec::len), Some(2));
    assert_eq!(body["relations"][0]["object"]["label"], "AAPL");
    assert!(
        body["relations"][0]["confidence"]
            .as_f64()
            .is_some_and(|c| (c - 0.9).abs() < 1e-6)
    );

    let matching = format!("{base}/profile/matching?subject=ME&relation=watches");
    assert_eq!(
        send(&app, "GET", &matching, None).await.1["relations"]
            .as_array()
            .map(Vec::len),
        Some(2)
    );

    let key = json!({"subject": "me", "relation": "watches", "object": "msft"});
    let relations = format!("{base}/profile/relations");
    assert_eq!(
        send(&app, "DELETE", &relations, Some(key.clone())).await.1["deleted"],
        true
    );
    assert_eq!(
        send(&app, "DELETE", &relations, Some(key)).await.1["deleted"],
        false
    );

    let (_, body) = send(
        &app,
        "DELETE",
        &format!("{base}/profile/nodes?label=aapl"),
        None,
    )
    .await;
    assert_eq!(body["deleted"], 1);
    let (_, body) = send(&app, "GET", &format!("{base}/profile?limit=10"), None).await;
    assert_eq!(body["relations"], json!([]));

    let (_, body) = send(&app, "DELETE", &format!("{base}/data"), None).await;
    assert_eq!(body["deleted"], 2);
    let (_, body) = send(&app, "GET", &format!("{base}/profile?limit=10"), None).await;
    assert_eq!(body["nodes"], json!([]));
}

#[tokio::test]
async fn memories_are_unexpired_scoped_and_filtered_by_kind() {
    let Some((app, base, db, tenant)) = live_db().await else {
        return;
    };
    for (user, kind, content, expired) in [
        ("111", "episodic", "asked about AAPL", false),
        ("111", "task", "tracking MSFT earnings", false),
        ("111", "task", "lapsed", true),
        ("222", "task", "someone else", false),
    ] {
        let inserted = sqlx::query(
            "insert into memories (id, tenant_id, user_id, kind, content, confidence, expires_at)
             values ($1, $2, $3, $4, $5, 0.8,
                     case when $6 then now() - interval '1 minute' end)",
        )
        .bind(Uuid::new_v4())
        .bind(&tenant)
        .bind(user)
        .bind(kind)
        .bind(content)
        .bind(expired)
        .execute(db.pool())
        .await;
        assert!(inserted.is_ok(), "{inserted:?}");
    }
    let (_, body) = send(&app, "GET", &format!("{base}/memories?limit=5"), None).await;
    let contents: Vec<&str> = body["memories"]
        .as_array()
        .map(|rows| rows.iter().filter_map(|r| r["content"].as_str()).collect())
        .unwrap_or_default();
    assert_eq!(contents, vec!["tracking MSFT earnings", "asked about AAPL"]);
    let (_, body) = send(
        &app,
        "GET",
        &format!("{base}/memories?limit=5&kinds=task"),
        None,
    )
    .await;
    assert_eq!(body["memories"].as_array().map(Vec::len), Some(1));
    assert_eq!(body["memories"][0]["kind"], "task");
}
