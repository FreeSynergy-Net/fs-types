//! K16 of ENTWURF section 5.
use fs_conformance::{ConformanceCode, ErrorCode, message_key};

/// K16: `ALL` lists every code once; names are unique kebab case; message keys are prefix and name.
#[test]
fn k16_all_codes() {
    let all = ConformanceCode::ALL;
    assert_eq!(all.len(), 8);
    let mut names: Vec<&str> = all.iter().map(|c| c.as_str()).collect();
    names.sort_unstable();
    names.dedup();
    assert_eq!(names.len(), all.len());
    for c in all {
        assert!(
            c.as_str()
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b == b'-'),
            "{}",
            c.as_str()
        );
        assert_eq!(message_key(*c), format!("conformance-error-{}", c.as_str()));
    }
    for (i, a) in all.iter().enumerate() {
        assert!(all[i + 1..].iter().all(|b| a != b));
    }
}
