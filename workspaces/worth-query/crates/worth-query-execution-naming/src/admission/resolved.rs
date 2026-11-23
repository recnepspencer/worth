//! Admission of names the reader's derived name index resolved, after
//! verifying each one's subject class and declared kind.
//!
//! Every name admitted here is a key Relational's index recorded from a
//! canonical committed birth; bare bytes never reach this boundary.

use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_declaration::persistent_name_admission_authority::admit_minted_digest;
use worth_relational::facade::lineage::{
    PersistentNameKey, PersistentNameObservationView, PersistentNameSubjectKind,
};
use worth_relational::facade::runtime::RelationalRuntime;

use crate::tracked_resolution::ExecutionNameReadDenial;

/// The subject class and declared subject name a read requests for `K`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct RequestedSubject<'kind> {
    pub(crate) kind: PersistentNameSubjectKind,
    pub(crate) name: &'kind str,
}

/// Verifies that `key`, recorded in `view`, was born under an installed
/// scheme governing exactly `requested`.
///
/// Complexity: O(1) expected index probe plus one registry lookup.
pub(crate) fn verify_resolved_subject(
    runtime: &RelationalRuntime,
    view: &PersistentNameObservationView<'_>,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<(), ExecutionNameReadDenial> {
    if key.subject_kind() != requested.kind {
        return Err(ExecutionNameReadDenial::SubjectKindMismatch {
            expected: requested.kind,
            actual: key.subject_kind(),
        });
    }
    let scheme_id = view.scheme_of(key);
    let Some((installed_kind, installed_subject)) =
        scheme_id.and_then(|scheme| runtime.persistent_name_scheme_subject(scheme))
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
    runtime: &RelationalRuntime,
    view: &PersistentNameObservationView<'_>,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<WorthQueryName<K>, ExecutionNameReadDenial> {
    admit_resolved_name_key(runtime, view, key, requested)
}

/// Admits a verified resolved key as the kind-typed name it was born as.
fn admit_resolved_name_key<K>(
    runtime: &RelationalRuntime,
    view: &PersistentNameObservationView<'_>,
    key: PersistentNameKey,
    requested: RequestedSubject<'_>,
) -> Result<WorthQueryName<K>, ExecutionNameReadDenial> {
    verify_resolved_subject(runtime, view, key, requested)?;
    Ok(admit_minted_digest(*key.digest()))
}
