//! `fs_types::interface`: name and version, name and need (ADR 0045).
use fs_types::interface::{InterfaceProblem, InterfaceReq, InterfaceVersion};

fn v(t: &str) -> InterfaceVersion {
    t.parse().unwrap()
}
fn r(t: &str) -> InterfaceReq {
    t.parse().unwrap()
}

#[test]
fn two_part_version_is_refused() {
    assert!(matches!(
        "tree@1.0".parse::<InterfaceVersion>(),
        Err(InterfaceProblem::BadVersion(_))
    ));
}

#[test]
fn three_part_version_is_read() {
    let x = v("tree@1.0.0");
    assert_eq!(x.name(), "tree");
    assert_eq!(x.version().to_string(), "1.0.0");
}

#[test]
fn range_with_comma_matches_inside_not_at_upper_bound() {
    let need = r("tasks@>=0.2.0, <0.3.0");
    assert!(need.matches(&v("tasks@0.2.9")));
    assert!(!need.matches(&v("tasks@0.3.0")));
    assert!(!need.matches(&v("other@0.2.9")));
}

#[test]
fn caret_on_zero_x_is_a_minor_line() {
    assert!(!r("x@^0.2.0").matches(&v("x@0.3.0")));
    assert!(r("x@^0.2.0").matches(&v("x@0.2.5")));
}

#[test]
fn caret_on_one_x_is_a_major_line() {
    let need = r("x@^1.2.0");
    assert!(need.matches(&v("x@1.9.0")));
    assert!(!need.matches(&v("x@2.0.0")));
}

#[test]
fn bad_names_are_refused() {
    for t in ["Tree@1.0.0", "@1.0.0", "1tree@1.0.0", "tr ee@1.0.0"] {
        assert_eq!(
            t.parse::<InterfaceVersion>(),
            Err(InterfaceProblem::BadName),
            "{t}"
        );
        assert_eq!(
            t.parse::<InterfaceReq>(),
            Err(InterfaceProblem::BadName),
            "{t}"
        );
    }
}

#[test]
fn text_without_at_is_refused() {
    assert_eq!(
        "tree 1.0.0".parse::<InterfaceVersion>(),
        Err(InterfaceProblem::MissingAt)
    );
    assert_eq!(
        "tree".parse::<InterfaceReq>(),
        Err(InterfaceProblem::MissingAt)
    );
}

#[test]
fn display_and_parse_round_trip() {
    for t in ["tree@1.0.0", "a.b-c@0.0.7"] {
        assert_eq!(v(t).to_string(), t);
    }
    for t in ["tasks@>=0.2.0, <0.3.0", "tree@^1.0.0"] {
        assert_eq!(r(t).to_string(), t);
        assert_eq!(r(&r(t).to_string()), r(t));
    }
}

#[test]
fn default_need_for_zero_zero_line_is_up_to_zero_one() {
    let need = v("tasks@0.0.3").default_req();
    assert_eq!(need.to_string(), "tasks@>=0.0.3, <0.1.0");
    assert!(need.matches(&v("tasks@0.0.9")));
    assert!(!need.matches(&v("tasks@0.0.2")));
    assert!(!need.matches(&v("tasks@0.1.0")));
}

#[test]
fn default_need_from_zero_one_is_caret() {
    assert_eq!(v("x@0.2.1").default_req().to_string(), "x@^0.2.1");
    assert_eq!(v("x@1.2.0").default_req().to_string(), "x@^1.2.0");
}

#[test]
fn pre_release_and_build_versions_are_refused() {
    for t in ["tree@1.0.0-rc.1", "tree@1.0.0+b1", "tree@1.0.0-rc.1+b1"] {
        assert!(
            matches!(
                t.parse::<InterfaceVersion>(),
                Err(InterfaceProblem::BadVersion(_))
            ),
            "{t}"
        );
    }
}

#[test]
fn pre_release_in_a_need_is_refused() {
    for t in ["tasks@>=0.2.0-rc.1, <0.3.0", "tree@^1.0.0-rc.1"] {
        assert!(
            matches!(t.parse::<InterfaceReq>(), Err(InterfaceProblem::BadReq(_))),
            "{t}"
        );
    }
}

#[test]
fn default_req_matches_its_own_version() {
    for t in ["x@0.0.3", "x@0.2.1", "x@1.2.0"] {
        let ver = v(t);
        assert!(ver.default_req().matches(&ver), "{t}");
    }
}

#[test]
fn is_full_needs_all_three_parts_and_no_wildcard() {
    assert!(r("x@>=0.2.0, <0.3.0").is_full());
    assert!(r("x@^1.2.3").is_full());
    assert!(!r("x@0.2").is_full());
    assert!(!r("x@*").is_full());
    assert!(!r("x@<=0.2").is_full());
}
