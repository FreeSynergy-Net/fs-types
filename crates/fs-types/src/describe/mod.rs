//! The type description a program reports for each data type it owns (ADR 0066 point 3).

mod check;
mod command;
mod describes;
mod edit;
mod edit_wire;
mod field;
mod problem;
mod record_commands;
mod registry;
mod type_description;
mod validate;
mod wire;

pub use crate::types_capnp::CommandKind;
pub use check::{RecordIssue, RecordProblem};
pub use command::CommandDescription;
pub use describes::DescribesType;
pub use edit::{Edit, FieldEdit};
pub use field::FieldDescription;
pub use problem::DescriptionProblem;
pub use registry::TypeRegistry;
pub use type_description::TypeDescription;
