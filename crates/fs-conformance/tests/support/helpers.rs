//! Small helper rules, a suite over a closure and expected-value builders for the K tests.
#![allow(dead_code)]

use std::cell::Cell;
use std::rc::Rc;

use async_trait::async_trait;
use fs_conformance::{
    Code, Coded, ConformanceCode, Cx, Param, Rule, RuleId, Suite, SuiteId, Verdict, keys,
};

use super::{Counter, Fx};

/// A rule with a fixed verdict.
pub struct Fixed(pub &'static str, pub Verdict);

#[async_trait(?Send)]
impl Rule<dyn Counter> for Fixed {
    fn id(&self) -> RuleId {
        RuleId::new(self.0)
    }
    async fn check(&self, _f: &mut Fx, _cx: &Cx) -> Verdict {
        self.1.clone()
    }
}

/// A rule that counts how often its body ran.
pub struct Count(pub &'static str, pub Rc<Cell<u32>>);

#[async_trait(?Send)]
impl Rule<dyn Counter> for Count {
    fn id(&self) -> RuleId {
        RuleId::new(self.0)
    }
    async fn check(&self, _f: &mut Fx, _cx: &Cx) -> Verdict {
        self.1.set(self.1.get() + 1);
        Verdict::Pass
    }
}

/// A rule that calls a closure with the context (for `missing` and `setup_failed`).
pub struct WithCx<G: Fn(&Cx) -> Verdict>(pub &'static str, pub G);

#[async_trait(?Send)]
impl<G: Fn(&Cx) -> Verdict> Rule<dyn Counter> for WithCx<G> {
    fn id(&self) -> RuleId {
        RuleId::new(self.0)
    }
    async fn check(&self, _f: &mut Fx, cx: &Cx) -> Verdict {
        (self.1)(cx)
    }
}

/// A suite over a list-making closure.
pub struct Custom<G: Fn() -> Vec<Box<dyn Rule<dyn Counter>>>>(pub G);

impl<G: Fn() -> Vec<Box<dyn Rule<dyn Counter>>>> Suite<dyn Counter> for Custom<G> {
    fn id(&self) -> SuiteId {
        SuiteId::new("custom")
    }
    fn rules(&self) -> Vec<Box<dyn Rule<dyn Counter>>> {
        (self.0)()
    }
}

/// A frame code with text parameters, erased (the expected value in tests).
pub fn frame(code: ConformanceCode, params: &[(&str, &str)]) -> Code {
    let mut c = Coded::new(code);
    for (k, v) in params {
        c = c.with_text(k, *v);
    }
    c.erase()
}

pub fn text(v: &str) -> Param {
    Param::Text(v.to_owned())
}

pub const RULE: &str = keys::RULE;
