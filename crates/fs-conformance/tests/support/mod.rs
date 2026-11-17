//! Demo area of the probe (ENTWURF section 7.3): a counter fixture, its three rules, its suite and three mutants.
#![allow(dead_code)]

mod helpers;
pub use helpers::*;

use async_trait::async_trait;
use fs_conformance::{
    Applied, Coded, Cx, ErrorCode, Mutant, MutantId, Need, Rule, RuleId, Suite, SuiteId, Verdict,
};

/// The demo error codes of the counter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoCode {
    TooMuch,
    Zero,
    Wrong,
}

impl ErrorCode for DemoCode {
    const PREFIX: &'static str = "demo-error-";
    const ALL: &'static [Self] = &[Self::TooMuch, Self::Zero, Self::Wrong];
    fn as_str(self) -> &'static str {
        match self {
            Self::TooMuch => "too-much",
            Self::Zero => "zero",
            Self::Wrong => "wrong",
        }
    }
}

/// The fixture type of the demo building block (used as `dyn Counter`).
/// The fixture type as the frame sees it: the object lifetime is `'static` (an elided `&mut dyn Counter` in an impl is not).
pub type Fx = dyn Counter;

pub trait Counter {
    fn add(&mut self, n: u32) -> Result<u32, Coded<DemoCode>>;
    fn total(&self) -> u32;
    fn limit(&self) -> Option<u32>;
    fn calls(&self) -> &[&'static str];
    fn log(&mut self, rule: &'static str);
    fn set_mutation(&mut self, mutation: DemoMutation) -> bool;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DemoMutation {
    SumOffByOne,
    NoLimit,
    ZeroAccepted,
}

#[derive(Default)]
pub struct FakeCounter {
    total: u32,
    limit: Option<u32>,
    mutation: Option<DemoMutation>,
    calls: Vec<&'static str>,
}

impl FakeCounter {
    pub fn new() -> Self {
        Self::default().with_limit(10)
    }

    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }

    pub fn without_limit() -> Self {
        Self::default()
    }

    pub fn with_mutation(mut self, mutation: DemoMutation) -> Self {
        self.mutation = Some(mutation);
        self
    }
}

impl Counter for FakeCounter {
    fn add(&mut self, n: u32) -> Result<u32, Coded<DemoCode>> {
        if n == 0 && self.mutation != Some(DemoMutation::ZeroAccepted) {
            return Err(Coded::new(DemoCode::Zero));
        }
        if let Some(limit) = self.limit
            && n > limit
            && self.mutation != Some(DemoMutation::NoLimit)
        {
            return Err(Coded::new(DemoCode::TooMuch).with_number("limit", u64::from(limit)));
        }
        self.total += if self.mutation == Some(DemoMutation::SumOffByOne) {
            n + 1
        } else {
            n
        };
        Ok(self.total)
    }
    fn total(&self) -> u32 {
        self.total
    }
    fn limit(&self) -> Option<u32> {
        self.limit
    }
    fn calls(&self) -> &[&'static str] {
        &self.calls
    }
    fn log(&mut self, rule: &'static str) {
        self.calls.push(rule);
    }
    fn set_mutation(&mut self, mutation: DemoMutation) -> bool {
        self.mutation = Some(mutation);
        true
    }
}

pub const ADD_SUMS: RuleId = RuleId::new("demo.add-sums");
pub const LIMIT: RuleId = RuleId::new("demo.limit");
pub const ZERO_REFUSED: RuleId = RuleId::new("demo.zero-refused");

fn wrong() -> Verdict {
    Verdict::Fail(Coded::new(DemoCode::Wrong).erase())
}

struct AddSums;
struct LimitRule;
struct ZeroRefused;

#[async_trait(?Send)]
impl Rule<dyn Counter> for AddSums {
    fn id(&self) -> RuleId {
        ADD_SUMS
    }
    async fn check(&self, f: &mut Fx, _cx: &Cx) -> Verdict {
        f.log("demo.add-sums");
        let before = f.total();
        if f.add(2).is_err() || f.add(3).is_err() {
            return wrong();
        }
        if f.total() - before == 5 {
            Verdict::Pass
        } else {
            wrong()
        }
    }
}

#[async_trait(?Send)]
impl Rule<dyn Counter> for LimitRule {
    fn id(&self) -> RuleId {
        LIMIT
    }
    async fn check(&self, f: &mut Fx, cx: &Cx) -> Verdict {
        f.log("demo.limit");
        let Some(limit) = f.limit() else {
            return cx.missing(Need::new("demo.limit"));
        };
        match f.add(limit + 1) {
            Err(e) if e.code() == DemoCode::TooMuch => Verdict::Pass,
            _ => wrong(),
        }
    }
}

#[async_trait(?Send)]
impl Rule<dyn Counter> for ZeroRefused {
    fn id(&self) -> RuleId {
        ZERO_REFUSED
    }
    async fn check(&self, f: &mut Fx, _cx: &Cx) -> Verdict {
        f.log("demo.zero-refused");
        match f.add(0) {
            Err(e) if e.code() == DemoCode::Zero => Verdict::Pass,
            _ => wrong(),
        }
    }
}

pub struct DemoSuite;

impl Suite<dyn Counter> for DemoSuite {
    fn id(&self) -> SuiteId {
        SuiteId::new("demo")
    }
    fn rules(&self) -> Vec<Box<dyn Rule<dyn Counter>>> {
        vec![
            Box::new(AddSums),
            Box::new(LimitRule),
            Box::new(ZeroRefused),
        ]
    }
}

/// A mutant that switches one demo mutation on.
pub struct DemoMutant {
    pub id: &'static str,
    pub rule: RuleId,
    pub mutation: DemoMutation,
}

impl Mutant<dyn Counter> for DemoMutant {
    fn id(&self) -> MutantId {
        MutantId::new(self.id)
    }
    fn must_fail(&self) -> Vec<RuleId> {
        vec![self.rule]
    }
    fn apply(&self, f: &mut Fx) -> Applied {
        if f.set_mutation(self.mutation) {
            Applied::Yes
        } else {
            Applied::NotApplicable
        }
    }
}

pub fn demo_mutants() -> Vec<Box<dyn Mutant<dyn Counter>>> {
    vec![
        Box::new(DemoMutant {
            id: "demo.sum-off-by-one",
            rule: ADD_SUMS,
            mutation: DemoMutation::SumOffByOne,
        }),
        Box::new(DemoMutant {
            id: "demo.no-limit",
            rule: LIMIT,
            mutation: DemoMutation::NoLimit,
        }),
        Box::new(DemoMutant {
            id: "demo.zero-accepted",
            rule: ZERO_REFUSED,
            mutation: DemoMutation::ZeroAccepted,
        }),
    ]
}

/// A mutant that does nothing but report `applied` and name `must` (for K10, K13, K14, K19).
pub struct Inert {
    pub id: &'static str,
    pub must: Vec<RuleId>,
    pub applied: Applied,
}

impl Mutant<dyn Counter> for Inert {
    fn id(&self) -> MutantId {
        MutantId::new(self.id)
    }
    fn must_fail(&self) -> Vec<RuleId> {
        self.must.clone()
    }
    fn apply(&self, _f: &mut Fx) -> Applied {
        self.applied
    }
}
