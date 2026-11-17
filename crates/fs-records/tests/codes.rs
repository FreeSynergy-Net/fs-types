//! K1 of ENTWURF section 5 for `RecordsCode` (the suite's codes are in `suite.rs`).
use fs_records::RecordsCode;
use fs_records::coded::{ErrorCode, message_key};

/// K1: every variant once in `ALL`, `as_str` unique kebab-case, `message_key` = prefix + name; 23 codes (22 of the draft + `limit-refused`).
#[test]
fn k1_records_codes() {
    let all = RecordsCode::ALL;
    assert_eq!(all.len(), 23); // draft 22 (K49) + `limit-refused` (O7)
    let names: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    let mut sorted = names.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), all.len(), "as_str must be unique");
    for (c, n) in all.iter().zip(&names) {
        assert!(
            !n.is_empty() && !n.starts_with('-') && !n.ends_with('-') && !n.contains("--"),
            "{n}"
        );
        assert!(
            n.chars().all(|ch| ch.is_ascii_lowercase() || ch == '-'),
            "{n}"
        );
        assert_eq!(message_key(*c), format!("records-error-{n}"));
    }
}
