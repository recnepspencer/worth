//! Selected Native predecessor, exact before-image and complete relation custody.
use super::*;

pub(super) fn validate<'a>(
    candidate: &CanonicalCommitEnvelope,
    runtime: &crate::runtime::RelationalPreparationRuntime,
    basis: &AdmittedRelationalBranchBasis,
    predecessor: &'a crate::mvcc::RelationalRequiredOutputPublication,
    meaning: &crate::mvcc::RequiredOutputPublicationMeaning,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<&'a CanonicalRequiredOutputOrigin, TransactionCommitError> {
    if predecessor.runtime_instance_id != runtime.runtime_instance_id() {
        return Err(crate::mvcc::origin_denial(
            "foreign native predecessor publication custody",
        ));
    }
    let origin = predecessor
        .envelope
        .required_output_origin
        .as_ref()
        .ok_or_else(|| {
            crate::mvcc::origin_denial("predecessor lacks native publication membership")
        })?;
    context
        .checkpoint((origin.meaning.family.len() + meaning.family.len() + 96) as u64)
        .map_err(origin_request)?;
    if origin.meaning.family != meaning.family
        || origin.meaning.source_scope != meaning.source_scope
        || origin.meaning.source_partition != meaning.source_partition
    {
        return Err(crate::mvcc::origin_denial(
            "successor changes complete installed family/source correspondence",
        ));
    }
    let position = basis
        .inner
        .root
        .output_publication_positions
        .get(&origin.meaning, context)?;
    let exact_cold_basis = predecessor
        .readmitted_at
        .as_ref()
        .is_some_and(|readmitted| readmitted.descriptor() == basis.descriptor());
    if position.is_some_and(|selected| {
        selected
            != crate::history::data::RequiredOutputRoutePosition::Published(origin.commit.commit_id)
    }) || (position.is_none() && !exact_cold_basis)
    {
        return Err(crate::mvcc::origin_denial(
            "predecessor is not the publication selected at the native before image",
        ));
    }
    // The carried holder is checked against its exact performed native coordinate,
    // not an ancestry search or a caller-supplied description.
    context.checkpoint(1).map_err(origin_request)?;
    let canonical = runtime
        .history
        .canonical_envelope_for_version(origin.commit.version_id)
        .ok_or_else(|| {
            crate::mvcc::origin_denial("retained predecessor canonical position absent")
        })?;
    let expected = canonical.required_output_origin.as_deref().ok_or_else(|| {
        crate::mvcc::origin_denial("retained predecessor native membership absent")
    })?;
    compare_retained_origin(origin, expected, context)?;
    canonical
        .validate_output_membership_header()
        .map_err(super::relation_stop)?;
    crate::history::data::publication_relations::validate(
        &basis.inner.root,
        None,
        origin,
        None,
        &mut |commit| runtime.history.canonical_envelope(commit),
        &mut |version| runtime.history.canonical_envelope_for_version(version),
        runtime.history.canonical_catalog_len(),
        lease,
        context,
    )
    .map_err(|stop| super::relation_context(stop, "predecessor", basis, candidate))?;
    Ok(origin.as_ref())
}
