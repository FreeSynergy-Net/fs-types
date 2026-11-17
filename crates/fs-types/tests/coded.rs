//! The module `coded`: codes with typed parameters (K-cases of the scaffold draft; each is a mutant target).
use fs_types::coded::{Code, Coded, ErrorCode, Param, message_key};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Demo {
    TooMuch,
    Zero,
}

impl ErrorCode for Demo {
    const PREFIX: &'static str = "demo-error-";
    const ALL: &'static [Self] = &[Self::TooMuch, Self::Zero];
    fn as_str(self) -> &'static str {
        match self {
            Self::TooMuch => "too-much",
            Self::Zero => "zero",
        }
    }
}

/// The message key is the prefix followed by the name, for the enum and for the erased code.
#[test]
fn message_key_is_prefix_and_name() {
    assert_eq!(message_key(Demo::TooMuch), "demo-error-too-much");
    assert_eq!(
        Coded::new(Demo::Zero).erase().message_key(),
        "demo-error-zero"
    );
}

/// Parameters keep the order they were added in, with their types.
#[test]
fn params_keep_order_and_type() {
    let c = Coded::new(Demo::TooMuch)
        .with_text("rule", "a.b")
        .with_number("limit", 10);
    let got: Vec<(&str, &Param)> = c.params().collect();
    assert_eq!(
        got,
        vec![
            ("rule", &Param::Text("a.b".to_owned())),
            ("limit", &Param::Number(10))
        ]
    );
    assert_eq!(c.code(), Demo::TooMuch);
}

/// Erasing the enum loses nothing: prefix, name and parameters are the same; `From` does the same.
#[test]
fn erase_loses_nothing() {
    let c = Coded::new(Demo::TooMuch)
        .with_number("limit", 10)
        .with_text("x", "y");
    let code: Code = c.clone().erase();
    assert_eq!(code.prefix, "demo-error-");
    assert_eq!(code.name, "too-much");
    assert_eq!(
        code.params,
        vec![
            ("limit".to_owned(), Param::Number(10)),
            ("x".to_owned(), Param::Text("y".to_owned())),
        ]
    );
    assert_eq!(Code::from(c.clone()), c.erase());
}

/// `param` finds the first parameter with the key and nothing for an unknown key.
#[test]
fn param_finds_the_first() {
    let code = Coded::new(Demo::Zero)
        .with_text("k", "first")
        .with_text("k", "second")
        .erase();
    assert_eq!(code.param("k"), Some(&Param::Text("first".to_owned())));
    assert_eq!(code.param("none"), None);
}

/// The log text: key alone, or key with the parameters in order; numbers and texts alike.
#[test]
fn display_is_a_log_text() {
    assert_eq!(
        Coded::new(Demo::Zero).erase().to_string(),
        "demo-error-zero"
    );
    let code = Coded::new(Demo::TooMuch)
        .with_text("rule", "a.b")
        .with_number("limit", 10)
        .erase();
    assert_eq!(code.to_string(), "demo-error-too-much (rule=a.b, limit=10)");
}

/// The catalog lists every code once, names are kebab case.
#[test]
fn all_lists_every_code_once() {
    assert_eq!(Demo::ALL.len(), 2);
    assert!(Demo::ALL.iter().all(|c| {
        c.as_str()
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b == b'-')
    }));
}

/// `with_param` adds a parameter of either kind unchanged, at the end.
#[test]
fn with_param_adds_unchanged() {
    let code = Coded::new(Demo::Zero)
        .with_param("a", Param::Number(1))
        .with_param("b", Param::Text("t".to_owned()))
        .erase();
    assert_eq!(code.param("a"), Some(&Param::Number(1)));
    assert_eq!(
        code.params[1],
        ("b".to_owned(), Param::Text("t".to_owned()))
    );
}
