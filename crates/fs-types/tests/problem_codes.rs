//! `ValueProblem::code()` and `RecordProblem::code()`: stable translation key suffixes, in the pattern of
//! `TypeProblem::code()` (one module for every program; the consumers drop their own tables).

use fs_types::values::ValueProblem;
use fs_types::{RecordProblem, TypeProblem};

#[test]
fn value_problem_codes_are_stable() {
    let codes = [
        (ValueProblem::Empty, "empty"),
        (ValueProblem::MissingAt, "email.missing-at"),
        (ValueProblem::ManyAt, "email.many-at"),
        (ValueProblem::EmptyLocal, "email.empty-local"),
        (ValueProblem::EmptyDomain, "email.empty-domain"),
        (ValueProblem::LocalCharacter, "email.local-character"),
        (ValueProblem::LocalTooLong, "email.local-too-long"),
        (ValueProblem::BadDomain, "email.bad-domain"),
        (ValueProblem::DomainWithoutDot, "email.domain-without-dot"),
        (ValueProblem::AddressTooLong, "email.too-long"),
        (ValueProblem::NoCountryCode, "phone.no-country-code"),
        (ValueProblem::PhoneCharacter, "phone.character"),
        (ValueProblem::PhoneLength, "phone.length"),
    ];
    for (problem, code) in codes {
        assert_eq!(problem.code(), code);
    }
}

#[test]
fn type_problem_contact_uses_the_value_code() {
    for p in [ValueProblem::Empty, ValueProblem::PhoneLength, ValueProblem::ManyAt] {
        assert_eq!(TypeProblem::Contact(p).code(), p.code());
    }
}

#[test]
fn record_problem_codes_are_stable() {
    let codes = [
        (RecordProblem::UnknownField, "unknown-field"),
        (RecordProblem::Missing, "missing"),
        (RecordProblem::NotMultiple, "not-multiple"),
        (RecordProblem::WrongType, "wrong-type"),
        (RecordProblem::NestedPath, "nested-path"),
        (RecordProblem::ReadOnly, "read-only"),
        (RecordProblem::DuplicatePath, "duplicate-path"),
        (RecordProblem::Value(TypeProblem::BadColor), "color.bad"),
    ];
    for (problem, code) in codes {
        assert_eq!(problem.code(), code);
    }
}
