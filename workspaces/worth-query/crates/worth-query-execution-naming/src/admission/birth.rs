//! Admission of a recorded name's birth, read at the reader's basis.
//!
//! The birth's parents and every name its discriminator references are
//! admitted only as names the same index recorded and shows visible at the
//! same basis; bare digests open nothing.

use std::collections::BTreeSet;

use worth_query_declaration::facade::application_naming::RecordedNameBirth;
use worth_query_declaration::persistent_name_admission_authority::{
    admit_recorded_birth, recorded_discriminator_references,
};
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameObservationView, PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::RelationalRuntime;

use super::{verify_resolved_subject, RequestedSubject};
use crate::tracked_resolution::ExecutionNameReadDenial;

const SUBJECT_KINDS: [PersistentNameSubjectKind; 3] = [
    PersistentNameSubjectKind::Entity,
    PersistentNameSubjectKind::Relation,
    PersistentNameSubjectKind::Group,
];

/// The birth `key` was minted from, admitted with its parents and
/// references after verifying `key` and each parent was born under a scheme
/// governing exactly `requested`.
///
/// Complexity: O(1) expected per parent and per referenced name, plus O(B)
/// for the B discriminator bytes.
pub(crate) fn admit_birth_of<K>(
    runtime: &RelationalRuntime,
    view: &PersistentNameObservationView<'_>,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<RecordedNameBirth<K>, ExecutionNameReadDenial> {
    verify_resolved_subject(runtime, view, key, requested)?;
    let parts = view
        .birth_of(key)
        .ok_or(ExecutionNameReadDenial::UnknownName)?;
    let parents = parts
        .parent_identities()
        .iter()
        .map(|parent| {
            let recorded = view
                .recorded_name(requested.kind, parent.as_bytes())
                .ok_or(ExecutionNameReadDenial::UnrecordedBirthReference {
                    digest: *parent.as_bytes(),
                })?;
            verify_resolved_subject(runtime, view, recorded, requested)?;
            Ok(*recorded.digest())
        })
        .collect::<Result<Vec<_>, ExecutionNameReadDenial>>()?;
    // Malformed recorded bytes admit nothing; a typed decode refuses them.
    let references = recorded_discriminator_references(parts.canonical_discriminator())
        .unwrap_or_default()
        .into_iter()
        .filter(|digest| {
            SUBJECT_KINDS
                .iter()
                .any(|kind| view.recorded_name(*kind, digest).is_some())
        })
        .collect::<BTreeSet<_>>();
    Ok(admit_recorded_birth(
        *key.digest(),
        parts.occurrence_key().to_owned(),
        parts.role_path().to_owned(),
        parents,
        parts.canonical_discriminator().to_vec(),
        references,
    ))
}
