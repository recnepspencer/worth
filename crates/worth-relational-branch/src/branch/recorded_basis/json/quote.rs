//! Native alone describes the nested heaps in its recorded component grammar.
use super::super::*;
use crate::history::data::recorded_json_allocation::{
    parent_heap_bound as parents, string_heap_bound as string,
};
use serde_json::value::RawValue;

pub(super) struct Quote {
    pub(super) retained: usize,
    pub(super) reallocation: usize,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Component<'a> {
    #[serde(borrow)]
    descriptor: &'a RawValue,
    #[serde(borrow)]
    target: &'a RawValue,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Descriptor<'a> {
    #[serde(rename = "descriptor_version")]
    _descriptor_version: serde::de::IgnoredAny,
    #[serde(rename = "runtime_instance_id")]
    _runtime_instance_id: serde::de::IgnoredAny,
    #[serde(borrow)]
    branch_id: &'a RawValue,
    #[serde(borrow)]
    reference: &'a RawValue,
    #[serde(rename = "truth_version")]
    _truth_version: serde::de::IgnoredAny,
    #[serde(rename = "root_identity")]
    _root_identity: serde::de::IgnoredAny,
    #[serde(default)]
    #[serde(rename = "schema_commitment")]
    _schema_commitment: Option<serde::de::IgnoredAny>,
    #[serde(rename = "visibility_commitment")]
    _visibility_commitment: serde::de::IgnoredAny,
    #[serde(rename = "posture")]
    _posture: serde::de::IgnoredAny,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Reference<'a> {
    #[serde(borrow)]
    branch_id: &'a RawValue,
    #[serde(borrow)]
    target: &'a RawValue,
    #[serde(rename = "generation")]
    _generation: serde::de::IgnoredAny,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Basis<'a> {
    #[serde(borrow, rename = "Basis")]
    basis: &'a RawValue,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Target<'a> {
    #[serde(rename = "runtime_instance_id")]
    _runtime_instance_id: serde::de::IgnoredAny,
    #[serde(rename = "commit_id")]
    _commit_id: serde::de::IgnoredAny,
    #[serde(rename = "version_id")]
    _version_id: serde::de::IgnoredAny,
    #[serde(borrow)]
    parent_commit_ids: &'a RawValue,
    #[serde(rename = "roots")]
    _roots: serde::de::IgnoredAny,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Canonical<'a> {
    #[serde(borrow, rename = "Canonical")]
    canonical: &'a RawValue,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CanonicalFields<'a> {
    #[serde(borrow)]
    receipt: &'a RawValue,
    #[serde(rename = "position")]
    _position: serde::de::IgnoredAny,
    #[serde(rename = "artifact_digest")]
    _artifact_digest: serde::de::IgnoredAny,
    #[serde(rename = "roots")]
    _roots: serde::de::IgnoredAny,
}
fn read<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, RelationalRecordedComponentRefusal> {
    serde_json::from_slice(bytes).map_err(|_| copying::malformed())
}
fn empty(value: &RawValue) -> Result<bool, RelationalRecordedComponentRefusal> {
    if !value.get().starts_with('"') {
        return Ok(false);
    }
    let target: RecordedRelationalComponentTarget = read(value.get().as_bytes())?;
    Ok(matches!(target, RecordedRelationalComponentTarget::Empty))
}
pub(super) fn quote(bytes: &[u8]) -> Result<Quote, RelationalRecordedComponentRefusal> {
    let component: Component<'_> = read(bytes)?;
    let descriptor: Descriptor<'_> = read(component.descriptor.get().as_bytes())?;
    let reference: Reference<'_> = read(descriptor.reference.get().as_bytes())?;
    let mut strings = string(descriptor.branch_id)?
        .checked_add(string(reference.branch_id)?)
        .ok_or_else(copying::malformed)?;
    let mut vectors = 0usize;
    if !empty(reference.target)? {
        let basis: Basis<'_> = read(reference.target.get().as_bytes())?;
        let target: Target<'_> = read(basis.basis.get().as_bytes())?;
        vectors = parents(target.parent_commit_ids)?;
    }
    let mut reallocation = vectors;
    if !empty(component.target)? {
        let canonical: Canonical<'_> = read(component.target.get().as_bytes())?;
        let canonical: CanonicalFields<'_> = read(canonical.canonical.get().as_bytes())?;
        let receipt =
            RelationalCommitReceipt::recorded_json_heap_quote(canonical.receipt.get().as_bytes())?;
        strings = strings
            .checked_add(
                usize::try_from(receipt.retained_bytes()).map_err(|_| copying::malformed())?,
            )
            .ok_or_else(copying::malformed)?;
        reallocation = reallocation
            .checked_add(
                usize::try_from(receipt.temporary_bytes()).map_err(|_| copying::malformed())?,
            )
            .ok_or_else(copying::malformed)?;
    }
    Ok(Quote {
        retained: copying::fixed_bytes()?
            .checked_add(strings)
            .and_then(|bytes| bytes.checked_add(vectors))
            .ok_or_else(copying::malformed)?,
        reallocation,
    })
}
