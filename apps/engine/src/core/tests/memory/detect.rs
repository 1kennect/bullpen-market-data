//! The gate on profile extraction. Rules, so it costs no model call on a greeting.

use crate::core::config;
use crate::core::traits::memory::detector::FactDetector;
use crate::runtime::harness::memory::detect::{RuleDetector, Rules};

fn detector() -> RuleDetector {
    RuleDetector::default()
}

#[test]
fn a_stated_preference_passes_the_gate() {
    let d = detector();
    for turn in [
        "I prefer dividend stocks over growth names",
        "I am a long term investor",
        "I hold a lot of NVDA in my retirement account",
        "My name is Alex and I live in Austin",
        "I bought some MSFT last week",
        "I usually trade on Fridays",
    ] {
        assert!(d.carries_fact(turn), "{turn:?} states something");
    }
}

#[test]
fn a_negated_preference_still_passes() {
    // A negated first person statement passes the gate.
    let d = detector();
    assert!(d.carries_fact("I do not like holding cash"));
    assert!(d.carries_fact("I never work on weekends"));
}

#[test]
fn small_talk_and_questions_cost_nothing() {
    let d = detector();
    for turn in [
        "hi",
        "hello there",
        "thanks!",
        "thank you so much",
        "how did NVDA close?",
        "what stocks are up today?",
        "Can you tell me about index funds?",
        "ok",
        "",
        "   ",
    ] {
        assert!(
            !d.carries_fact(turn),
            "{turn:?} states nothing about the asker"
        );
    }
}

#[test]
fn a_question_about_oneself_is_still_a_question() {
    // A sentence ending in a question mark is a question.
    let d = detector();
    assert!(!d.carries_fact("What stocks should I pick as a beginner investor?"));
}

#[test]
fn a_statement_followed_by_a_question_still_passes() {
    // A statement before a question passes; questions alone do not.
    let d = detector();
    assert!(d.carries_fact(
        "I am a long term investor and I really love tech stocks. Why should I buy an ETF?"
    ));
    assert!(d.carries_fact("Which broker is cheapest? I live in Brooklyn"));
    assert!(!d.carries_fact("Is there a robotics ETF? Does it pay dividends?"));
}

#[test]
fn a_first_person_marker_has_to_start_a_word() {
    // The i at the end of ai is not the asker.
    let d = detector();
    assert!(!d.carries_fact("The fund managers love robotics and meet weekly"));
    assert!(!d.carries_fact("Is the market open? The fund managers like Thursdays"));
}

#[test]
fn a_sentence_with_no_first_person_subject_is_not_about_the_asker() {
    let d = detector();
    assert!(!d.carries_fact("The Nasdaq is open until 4pm on weekdays"));
    assert!(!d.carries_fact("The fund rebalances on Thursdays"));
}

#[test]
fn a_marker_inside_another_word_does_not_count() {
    // Imagine and limited do not match a first person I.
    let d = detector();
    assert!(!d.carries_fact("Imagine the market was always open"));
    assert!(!d.carries_fact("Financial aid is limited this year"));
}

#[test]
fn the_rules_come_from_configuration() {
    let cfg = config::Detector::default();
    let rules = Rules::from(&cfg);
    assert_eq!(rules.min_words, cfg.min_words);
    assert!(
        !rules.subjects.is_empty(),
        "an empty list falls back to the built-in one"
    );
    assert_ne!(rules.cues.len(), 0);

    // A deployment can narrow the gate to what it cares about.
    let narrow = RuleDetector::new(Rules {
        subjects: vec!["i ".into()],
        cues: vec!["dividend".into()],
        min_words: 3,
    });
    assert!(narrow.carries_fact("I am a dividend investor"));
    assert!(!narrow.carries_fact("I prefer mornings"));
}

#[test]
fn a_turn_shorter_than_the_floor_is_skipped() {
    let d = RuleDetector::new(Rules {
        min_words: 5,
        ..Rules::default()
    });
    assert!(!d.carries_fact("I like cats"));
    assert!(d.carries_fact("I like cats and long walks"));
}
