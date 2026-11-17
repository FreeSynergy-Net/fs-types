//! Helpers of the rules: each check is a `Result`, the first failure is the verdict; a macro makes a rule of an async body.
//!
//! Pattern: Specification (small predicates) + Template method (the macro: arrange, body, verdict).

use fs_conformance::{Coded, Cx, Need, Verdict};

use super::codes::{SuiteCode, keys};
use super::fixture::{Arrange, Facts, RecordsFixture};
use crate::coded::{Coded as RecCoded, message_key};
use crate::records::RecordsCode;

/// Why a rule body stopped: a failure of the suite, or a verdict of the frame (a missing need).
pub(crate) enum Stop {
    /// A failure with the suite's code.
    Fail(Coded<SuiteCode>),
    /// A verdict as it is (`Cx::missing`).
    Verdict(Verdict),
}

impl From<Coded<SuiteCode>> for Stop {
    fn from(c: Coded<SuiteCode>) -> Self {
        Self::Fail(c)
    }
}

/// The body of a rule: `?` works on every helper.
pub(crate) type Body = Result<(), Stop>;

/// A failure with `expected` and `got`.
pub(crate) fn wrong(code: SuiteCode, expected: impl Into<String>, got: impl Into<String>) -> Stop {
    Stop::Fail(
        Coded::new(code)
            .with_text(keys::EXPECTED, expected)
            .with_text(keys::GOT, got),
    )
}

/// Seeds, or stops with the need `fixture.arrange`.
pub(crate) fn arrange<F: RecordsFixture + ?Sized>(
    f: &mut F,
    cx: &Cx,
    situation: Arrange,
) -> Result<Facts, Stop> {
    f.arrange(situation)
        .ok_or_else(|| Stop::Verdict(cx.missing(Need::new("fixture.arrange"))))
}

/// A call that must succeed.
pub(crate) fn call<T>(r: Result<T, RecCoded<RecordsCode>>) -> Result<T, Stop> {
    r.map_err(|e| {
        Stop::Fail(Coded::new(SuiteCode::CallFailed).with_text(keys::GOT, message_key(e.code())))
    })
}

/// A call that must fail with `want`.
pub(crate) fn refused<T>(r: Result<T, RecCoded<RecordsCode>>, want: RecordsCode) -> Body {
    match r {
        Err(e) if e.code() == want => Ok(()),
        Err(e) => Err(wrong(
            SuiteCode::CodeWrong,
            message_key(want),
            message_key(e.code()),
        )),
        Ok(_) => Err(wrong(SuiteCode::CodeWrong, message_key(want), "ok")),
    }
}

/// A rule: `rule!(Type, "id", |fixture, cx| { ...; Ok(()) })`; the body is async and returns a [`Body`].
macro_rules! rule {
    ($(#[$doc:meta])* $t:ident, $id:literal, |$f:ident, $cx:ident| $body:block) => {
        $(#[$doc])*
        pub(crate) struct $t;

        #[async_trait::async_trait(?Send)]
        impl<F: $crate::suite::RecordsFixture + ?Sized> fs_conformance::Rule<F> for $t {
            fn id(&self) -> fs_conformance::RuleId {
                fs_conformance::RuleId::new($id)
            }

            async fn check(&self, $f: &mut F, $cx: &fs_conformance::Cx) -> fs_conformance::Verdict {
                // A nested function, not a closure: `?` works and nothing is captured.
                async fn body<F: $crate::suite::RecordsFixture + ?Sized>(
                    $f: &mut F,
                    $cx: &fs_conformance::Cx,
                ) -> $crate::suite::check::Body $body
                match body($f, $cx).await {
                    Ok(()) => fs_conformance::Verdict::Pass,
                    Err($crate::suite::check::Stop::Fail(code)) => fs_conformance::Verdict::Fail(code.erase()),
                    Err($crate::suite::check::Stop::Verdict(v)) => v,
                }
            }
        }
    };
}

pub(crate) use rule;
