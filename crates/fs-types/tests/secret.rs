//! `Secret<T>` never shows its value (0.0.5).

use fs_types::Secret;

#[test]
fn debug_and_display_never_show_the_value() {
    let s = Secret::new("hunter2".to_owned());
    assert_eq!(format!("{s}"), "<hidden>");
    assert_eq!(format!("{s:?}"), "Secret(<hidden>)");
    assert_eq!(format!("{s:#?}"), "Secret(<hidden>)");
    assert!(!format!("{s} {s:?} {s:>20} {s:#?}").contains("hunter2"));
}

#[test]
fn a_secret_inside_a_struct_stays_hidden() {
    #[derive(Debug)]
    #[allow(dead_code)] // read through `Debug` only, which is the point
    struct Login {
        user: String,
        password: Secret<String>,
    }
    let l = Login {
        user: "ada".to_owned(),
        password: Secret::new("s3cret!".to_owned()),
    };
    let shown = format!("{l:?} {l:#?}");
    assert!(shown.contains("ada"));
    assert!(!shown.contains("s3cret!"));
    let v = vec![Secret::new(1234_u32)];
    assert!(!format!("{v:?}").contains("1234"));
    assert!(!format!("{:?}", Some(Secret::new(7777_u32))).contains("7777"));
}

#[test]
fn the_only_ways_out_are_named() {
    let s = Secret::new(vec![1_u8, 2, 3]);
    assert_eq!(s.expose(), &vec![1, 2, 3]);
    let copy = s.clone();
    assert_eq!(s.into_exposed(), vec![1, 2, 3]);
    let n = copy.map(|v| v.len());
    assert_eq!(*n.expose(), 3);
    assert_eq!(format!("{n:?}"), "Secret(<hidden>)");
    let from: Secret<&str> = "x".into();
    assert_eq!(*from.expose(), "x");
}

#[test]
fn constant_time_compare_looks_at_every_byte() {
    let a = Secret::new(b"abcdef".to_vec());
    assert!(a.ct_eq(&Secret::new(b"abcdef".to_vec())));
    assert!(!a.ct_eq(&Secret::new(b"abcdeg".to_vec())));
    assert!(!a.ct_eq(&Secret::new(b"bbcdef".to_vec())));
    assert!(!a.ct_eq(&Secret::new(b"abcde".to_vec())));
    assert!(!a.ct_eq(&Secret::new(b"abcdefg".to_vec())));
    assert!(Secret::new(String::new()).ct_eq(&Secret::new(String::new())));
}
