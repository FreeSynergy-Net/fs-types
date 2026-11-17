//! The trait every program offers next to each capability, beside `TypeCatalog` (ADR 0087 Nachtrag point 4, widened by this
//! draft: describe AND read). One provider, all methods (ADR 0045): what it cannot do is a code, never a missing method.
//!
//! The futures are boxed and not `Send`: a capnp client is `Rc` (fs-conformance O4), and the trait must stay usable as
//! `dyn Records` (one per binding in the host). No default methods (trait-delta rule 6). Every call answers as the asking
//! person: the connection carries her identity (local socket in her runtime dir, or her OAuth token, ADR 0024); a provider
//! never answers with a service account's rights (ADR 0087 Nachtrag point 8).
//!
//! Pattern: Repository (read side; the write side follows in E4w as its own trait).

use std::future::Future;
use std::pin::Pin;

use super::codes::RecordsCode;
use super::limits::Limits;
use super::query::RecordQuery;
use super::shape::{GroupSet, Shape};
use super::view::{RecordPage, RecordView};
use crate::coded::Coded;

/// A pending answer.
pub type Pending<'a, T> = Pin<Box<dyn Future<Output = Result<T, Coded<RecordsCode>>> + 'a>>;

/// Reading the records of the types a program owns.
///
/// The future owns what it needs from the arguments (a capnp client copies the query into its request at once).
pub trait Records {
    /// The provider's own limits; the host meets them with its own (`Limits::meet`).
    fn limits(&self) -> Limits;

    /// The groupings and orders of a listable type.
    ///
    /// # Errors
    /// `unknown-type`, `unavailable`.
    fn shape(&self, data_type: &str) -> Pending<'_, Shape>;

    /// One page of the records the asking person may read, filtered, sorted (ties by id), optionally grouped.
    /// A record she may not read is not there: not counted, not hinted at.
    ///
    /// # Errors
    /// `unknown-type`, `unknown-field`, `unknown-grouping`, `unknown-order`, `value-mismatch`, `too-many`, `page-size`,
    /// `cursor-invalid`, `unsupported`, `unavailable`.
    fn find(&self, query: &RecordQuery) -> Pending<'_, RecordPage>;

    /// One record by id, with the fields asked (empty = all she may read). A record that does not exist and one she may
    /// not read are the same answer: `None` (no existence leak).
    ///
    /// # Errors
    /// `unknown-type`, `unknown-field`, `too-many`, `unavailable`.
    fn get(
        &self,
        data_type: &str,
        id: &str,
        fields: &[Vec<String>],
    ) -> Pending<'_, Option<RecordView>>;

    /// The groups of a grouping now, left to right (in `scope`: tasks columns are per list).
    ///
    /// # Errors
    /// `unknown-type`, `unknown-grouping`, `unavailable`.
    fn groups(&self, data_type: &str, grouping: &str, scope: Option<&str>)
    -> Pending<'_, GroupSet>;

    /// A number that changes whenever a record of the type in `scope` or its group set changed (a write of anyone); the
    /// host reloads when it differs (as `BoardSource::generation`). The push feed (`ChangeListener`) comes with F2.
    ///
    /// # Errors
    /// `unknown-type`, `unavailable`.
    fn generation(&self, data_type: &str, scope: Option<&str>) -> Pending<'_, u64>;
}
