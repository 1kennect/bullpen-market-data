//! The no-auth tools: Wikipedia, tested on canned JSON bodies.

use crate::runtime::tools::wiki;

#[test]
fn wikipedia_returns_the_top_page_extract() {
    let body = r#"{"query":{"pages":{
        "736":{"index":1,"title":"Nasdaq","extract":"Nasdaq is an American stock exchange."},
        "999":{"index":2,"title":"Other","extract":"Unrelated."}}}}"#;
    let out = match wiki::from_json("Nasdaq", body, 1500) {
        Ok(out) => out,
        Err(e) => unreachable!("wiki parse: {e}"),
    };
    assert!(out.content.starts_with("Nasdaq"), "{}", out.content);
    assert!(
        out.content.contains("American stock exchange"),
        "{}",
        out.content
    );
}

#[test]
fn wikipedia_no_match_says_so() {
    let out = match wiki::from_json("zzzz", r#"{"batchcomplete":""}"#, 1500) {
        Ok(out) => out,
        Err(e) => unreachable!("wiki parse: {e}"),
    };
    assert!(
        out.content.contains("No Wikipedia article"),
        "{}",
        out.content
    );
}
