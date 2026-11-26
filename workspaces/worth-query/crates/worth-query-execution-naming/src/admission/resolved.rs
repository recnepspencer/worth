//! Admission of names the reader's derived name index resolved, after
//! verifying each one's subject class and declared kind.
//!
//! Every name admitted here is a key Relational's index recorded from a
//! canonical committed birth; bare bytes never reach this boundary.

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::persistent_name_admission_authority::admit_minted_digest;
use worth_relational::facade::lineage::{PersistentNameKey, PersistentNameSubjectKind};

use super::RecordedNames;

use crate::tracked_resolution::ExecutionNameReadDenial;

/// The subject class and declared subject name a read requests for `K`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RequestedSubject<'kind> {
    pub(crate) kind: PersistentNameSubjectKind,
    pub(crate) name: &'kind str,
}

/// Verifies that `key`, recorded in `names`, was born under an installed
/// scheme governing exactly `requested`.
///
/// Complexity: O(1) expected index probe plus one registry lookup.
pub(crate) fn verify_resolved_subject(
    names: &impl RecordedNames,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<(), ExecutionNameReadDenial> {
    if key.subject_kind() != requested.kind {
        return Err(ExecutionNameReadDenial::SubjectKindMismatch {
            expected: requested.kind,
            actual: key.subject_kind(),
        });
    }
    let scheme_id = names.scheme_of(key);
    let Some((installed_kind, installed_subject)) =
        scheme_id.and_then(|scheme| names.scheme_subject(scheme))
    else {
        return Err(ExecutionNameReadDenial::UnknownScheme {
            scheme_id: scheme_id.map(str::to_owned),
        });
    };
    if installed_kind != requested.kind {
        return Err(ExecutionNameReadDenial::SubjectKindMismatch {
            expected: requested.kind,
            actual: installed_kind,
        });
    }
    if installed_subject != requested.name {
        return Err(ExecutionNameReadDenial::KindIdentityMismatch {
            scheme_id: scheme_id.unwrap_or_default().to_owned(),
            expected_subject: installed_subject,
            requested_subject: requested.name.to_owned(),
        });
    }
    Ok(())
}

/// The kind-typed name a resolved key was born as, after verification.
pub(crate) fn kind_verified_resolved_name<K>(
    names: &impl RecordedNames,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<WorthQueryName<K>, ExecutionNameReadDenial> {
    admit_resolved_name_key(names, key, requested)
}

/// Admits a verified resolved key as the kind-typed name it was born as.
fn admit_resolved_name_key<K>(
    names: &impl RecordedNames,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<WorthQueryName<K>, ExecutionNameReadDenial> {
    verify_resolved_subject(names, key, requested)?;
    Ok(admit_minted_digest(*key.digest()))
}

/// The live group a verified resolved group target names, as the kind-typed
/// handle its resolution answers with.
pub(crate) fn resolved_group_handle<G>(
    target: &worth_relational::facade::lineage::PersistentNameTarget,
) -> Option<WorthQueryName<G>> {
    match target {
        worth_relational::facade::lineage::PersistentNameTarget::Group { group } => {
            Some(admit_minted_digest(*group.bytes()))
        }
        _ => None,
    }
}
