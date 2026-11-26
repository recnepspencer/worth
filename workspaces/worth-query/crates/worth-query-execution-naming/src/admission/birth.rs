//! Admission of a recorded name's birth, read at the reader's basis.
//!
//! The birth's parents and every name its discriminator references are
//! admitted only as names the same index recorded and shows visible at the
//! same basis, each reference under the subject class and declared subject
//! of its installed scheme; bare digests open nothing.

use worth_query_declaration::facade::application_naming::{
    AdmittedNameReferences, ApplicationNameSubjectKind, RecordedNameBirth,
};
use worth_query_declaration::persistent_name_admission_authority::{
    admit_recorded_birth, recorded_discriminator_references,
};
use worth_relational::facade::lineage::{PersistentNameKey, PersistentNameSubjectKind};

use super::{verify_resolved_subject, RecordedNames, RequestedSubject};
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
    names: &impl RecordedNames,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<RecordedNameBirth<K>, ExecutionNameReadDenial> {
    verify_resolved_subject(names, key, requested)?;
    let parts = names
        .birth_of(key)
        .ok_or(ExecutionNameReadDenial::UnknownName)?;
    let parents = parts
        .parent_identities()
        .iter()
        .map(|parent| {
            let recorded = names
                .recorded_name(requested.kind, parent.as_bytes())
                .ok_or(ExecutionNameReadDenial::UnrecordedBirthReference {
                    digest: *parent.as_bytes(),
                })?;
            verify_resolved_subject(names, recorded, requested)?;
            Ok(*recorded.digest())
        })
        .collect::<Result<Vec<_>, ExecutionNameReadDenial>>()?;
    // Malformed recorded bytes admit nothing; a typed decode refuses them.
    let references = recorded_discriminator_references(parts.canonical_discriminator())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|digest| Some((digest, admitted_reference(names, &digest)?)))
        .collect::<AdmittedNameReferences>();
    Ok(admit_recorded_birth(
        *key.digest(),
        parts.occurrence_key().to_owned(),
        parts.role_path().to_owned(),
        parents,
        parts.canonical_discriminator().to_vec(),
        references,
    ))
}

/// The subject class and declared subject `digest` was born under, when it
/// is a name recorded and visible at the basis under an installed scheme of
/// that same class. Anything else is not admitted, so it never decodes.
///
/// Complexity: O(1) expected index probes plus one registry lookup.
fn admitted_reference(
    names: &impl RecordedNames,
    digest: &[u8; 32],
) -> Option<(ApplicationNameSubjectKind, String)> {
    SUBJECT_KINDS.iter().find_map(|kind| {
        let key = names.recorded_name(*kind, digest)?;
        let (installed, subject) = names.scheme_subject(names.scheme_of(key)?)?;
        (installed == *kind).then(|| (application_subject(installed), subject))
    })
}

const fn application_subject(kind: PersistentNameSubjectKind) -> ApplicationNameSubjectKind {
    match kind {
        PersistentNameSubjectKind::Entity => ApplicationNameSubjectKind::Entity,
        PersistentNameSubjectKind::Relation => ApplicationNameSubjectKind::Relation,
        PersistentNameSubjectKind::Group => ApplicationNameSubjectKind::Group,
    }
}
