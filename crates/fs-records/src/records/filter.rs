//! The filter tree of a query (Nachzug 17.11., Suche/Filterbaum): JMAP Core `/query` (RFC 8620 §5.5) `FilterOperator`
//! with `AND`, `OR`, `NOT` and their `conditions`, nested; plus the NAMED extension `XOR` (JMAP has none; user, Gespräch 4
//! Z. 73: „exklusives oder … der Standard und ggf. eine Erweiterung“). Leaves are typed [`Condition`]s; no expression.
//!
//! Semantics (n-ary, as JMAP): `And` = all hold; `Or` = at least one; `Not` = NONE holds (JMAP `NOT` over its list);
//! `Xor` = EXACTLY ONE holds (decision, draft §14.2: for two operands it is the usual XOR, `(A OR B) AND NOT (A AND B)`;
//! for three or more „genau eins“ is what a person means by „entweder … oder“, odd parity would make three true count as
//! true). An operator needs at least one operand (`filter-invalid{what=empty-operator}`; JMAP leaves the empty case open).
//!
//! Pattern: Composite (operators over leaves) + Specification ([`Filter::check`]: depth, nodes, form).

use super::codes::{RecordsCode, keys};
use super::query::Condition;
use crate::coded::Coded;

/// One node of the tree.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum Filter {
    /// All operands hold.
    And(Vec<Filter>),
    /// At least one holds.
    Or(Vec<Filter>),
    /// None holds (JMAP `NOT`; with one operand the plain negation).
    Not(Vec<Filter>),
    /// Exactly one holds (named extension `XOR`).
    Xor(Vec<Filter>),
    /// A typed condition on one field (or a text term).
    Is(Condition),
}

impl Filter {
    /// The depth: a leaf is 1, an operator 1 + its deepest operand.
    #[must_use]
    pub fn depth(&self) -> usize {
        match self {
            Self::Is(_) => 1,
            Self::And(n) | Self::Or(n) | Self::Not(n) | Self::Xor(n) => {
                1 + n.iter().map(Self::depth).max().unwrap_or(0)
            }
        }
    }

    /// The number of nodes (operators and leaves).
    #[must_use]
    pub fn nodes(&self) -> usize {
        match self {
            Self::Is(_) => 1,
            Self::And(n) | Self::Or(n) | Self::Not(n) | Self::Xor(n) => {
                1 + n.iter().map(Self::nodes).sum::<usize>()
            }
        }
    }

    /// Every leaf, depth first.
    #[must_use]
    pub fn conditions(&self) -> Vec<&Condition> {
        match self {
            Self::Is(c) => vec![c],
            Self::And(n) | Self::Or(n) | Self::Not(n) | Self::Xor(n) => {
                n.iter().flat_map(Self::conditions).collect()
            }
        }
    }

    /// Form and size: no empty operator, depth ≤ `depth_max`, nodes ≤ `nodes_max`. Depth is checked first, so a deep
    /// tree is refused before it is walked further.
    ///
    /// # Errors
    /// `filter-too-deep{depth, max}`, `too-many{what=nodes, max}`, `filter-invalid{what=empty-operator}`.
    pub fn check(&self, depth_max: usize, nodes_max: usize) -> Result<(), Coded<RecordsCode>> {
        let depth = self.depth();
        if depth > depth_max {
            return Err(Coded::new(RecordsCode::FilterTooDeep)
                .with_number(keys::DEPTH, depth as u64)
                .with_number(keys::MAX, depth_max as u64));
        }
        if self.nodes() > nodes_max {
            return Err(Coded::new(RecordsCode::TooMany)
                .with_text(keys::WHAT, "nodes")
                .with_number(keys::MAX, nodes_max as u64));
        }
        if self.has_empty_operator() {
            return Err(
                Coded::new(RecordsCode::FilterInvalid).with_text(keys::WHAT, "empty-operator")
            );
        }
        Ok(())
    }

    fn has_empty_operator(&self) -> bool {
        match self {
            Self::Is(_) => false,
            Self::And(n) | Self::Or(n) | Self::Not(n) | Self::Xor(n) => {
                n.is_empty() || n.iter().any(Self::has_empty_operator)
            }
        }
    }

    /// Evaluates the tree with `leaf` for the conditions (the provider's rule for one record).
    pub fn holds(&self, leaf: &dyn Fn(&Condition) -> bool) -> bool {
        match self {
            Self::Is(c) => leaf(c),
            Self::And(n) => n.iter().all(|f| f.holds(leaf)),
            Self::Or(n) => n.iter().any(|f| f.holds(leaf)),
            Self::Not(n) => !n.iter().any(|f| f.holds(leaf)),
            Self::Xor(n) => n.iter().filter(|f| f.holds(leaf)).count() == 1,
        }
    }

    /// `self AND other` (flattens an existing `And`).
    #[must_use]
    pub fn and(self, other: Filter) -> Filter {
        match self {
            Self::And(mut n) => {
                n.push(other);
                Self::And(n)
            }
            first => Self::And(vec![first, other]),
        }
    }
}
