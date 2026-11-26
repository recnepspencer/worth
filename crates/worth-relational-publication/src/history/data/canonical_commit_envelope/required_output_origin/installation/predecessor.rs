//! Selected Native predecessor, exact before-image and complete relation custody.
use super::*;
use contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaningOwnerPort as _;
use worth_relational_branch::internal_port::branch::basis::{
    AdmittedRelationalBranchBasis, AdmittedRelationalBranchBasisFieldsPort as _,
};
use worth_relational_contracts::internal_port as contracts;

pub(super) fn validate<'a>(
    candidate: &CanonicalCommitEnvelopeBody,
    runtime: &crate::history::data::canonical_commit_envelope::required_output_origin::history_request::OutputPreparationRequest<'_, '_, '_>,
    basis: &AdmittedRelationalBranchBasis,
    predecessor: &'a crate::mvcc::publication::application_publication::RelationalRequiredOutputPublication,
    meaning: &contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputPublicationMeaning,
    lease: Option<&worth_execution::ExecutionResourceLease<'_>>,
    validated_history: &mut OutputOriginHistory,
    context: &mut MapKernelContext<'_, '_>,
) -> Result<&'a CanonicalRequiredOutputOrigin, TransactionCommitError> {
    if predecessor.runtime_instance_id != runtime.runtime_instance_id {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "foreign native predecessor publication custody",
            ),
        );
    }
    let origin = &predecessor.origin;
    contracts::execution::request_stage::checkpoint(
        context,
        (origin.meaning.family.len() + meaning.family.len() + 96) as u64,
    )
    .map_err(origin_request)?;
    if origin.meaning.family != meaning.family
        || origin.meaning.source_scope != meaning.source_scope
        || origin.meaning.source_partition != meaning.source_partition
    {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "successor changes complete installed family/source correspondence",
            ),
        );
    }
    let position = basis
        .owner_inner()
        .root
        .output_publication_positions
        .get(&origin.meaning, context)?;
    let exact_cold_basis = predecessor
        .readmitted_at
        .as_ref()
        .is_some_and(|readmitted| readmitted.descriptor() == basis.descriptor());
    if position.is_some_and(|selected| {
        selected
            != worth_relational_branch::internal_port::history::data::required_output_positions::RequiredOutputRoutePosition::Published(origin.commit.commit_id)
    }) || (position.is_none() && !exact_cold_basis)
    {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "predecessor is not the publication selected at the native before image",
            ),
        );
    }
    if origin.source_scope_owned
        && origin
            .members
            .iter()
            .any(|member| member.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
    {
        let current = runtime.current_ownership.as_ref().ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial("live owned predecessor requires current Native custody")
        })?;
        let canonical = current
            .origins()
            .find(|other| other.meaning.same_route(&origin.meaning))
            .ok_or_else(|| {
                contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                    "live owned predecessor is absent from current Native custody",
                )
            })?;
        compare_retained_origin(origin, canonical, context)?;
        if let worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence::Current(retained) =
            &predecessor.evidence
            && retained.basis.as_ref() != Some(basis.descriptor())
        {
            return Err(contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "current publication custody has a different Native basis",
            ));
        }
        return Ok(origin.as_ref());
    }
    // The carried holder is checked against its exact performed native coordinate,
    // not an ancestry search or a caller-supplied description.
    contracts::execution::request_stage::checkpoint(context, 1).map_err(origin_request)?;
    let worth_relational_branch::internal_port::publication::ownership::evidence::PublicationEvidence::Canonical(envelope) =
        &predecessor.evidence
    else {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "historical predecessor requires its canonical envelope",
            ),
        );
    };
    if envelope.commit != origin.commit {
        return Err(
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "carried origin differs from its canonical envelope",
            ),
        );
    }
    let canonical = validated_history
        .by_version(
            origin.commit.version_id,
            &mut |version, context| {
                runtime
                    .history
                    .canonical_envelope_for_version(version, context)
            },
            &runtime.history.execution(),
            context,
        )
        .map_err(worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::installation::relation_failure::relation_stop)?
        .ok_or_else(|| {
            contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
                "retained predecessor canonical position absent",
            )
        })?;
    let expected = canonical
        .output_origin_for_route(&origin.meaning, context)
        .map_err(worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::installation::relation_failure::relation_stop)?;
    compare_retained_origin(origin, expected, context)?;
    worth_relational_branch::internal_port::history::data::publication_relations::validate(
        &basis.root_read(),
        None,
        origin,
        Some(
            worth_relational_branch::internal_port::history::data::publication_relations::current_frame::CurrentPublicationFrame::Complete(
                canonical.as_view(),
            ),
        ),
        &mut |commit| runtime.history.canonical_envelope(commit),
        &mut |version, context| {
            runtime
                .history
                .canonical_envelope_for_version(version, context)
        },
        runtime.history.canonical_catalog_len(),
        lease,
        validated_history,
        &runtime.history.execution(),
        context,
    )
    .map_err(|stop| worth_relational_branch::internal_port::history::data::canonical_commit_envelope::required_output_origin::installation::relation_failure::relation_context(stop, "predecessor", basis.descriptor(), candidate))?;
    Ok(origin.as_ref())
}

/// Legacy retirement preserves its exact complete, Retire-only roster contract.
pub(super) fn require_complete_retirement(
    origin: &CanonicalRequiredOutputOrigin,
    roles: &[contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRoleTarget],
) -> Result<(), TransactionCommitError> {
    if roles
        .iter()
        .any(|role| role.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
        || roles.len()
            != origin
                .members
                .iter()
                .filter(|member| member.posture != contracts::mvcc::publication::application_publication::output_meaning::RequiredOutputRolePosture::Retire)
                .count()
    {
        return Err(contracts::transactions::data::outcomes::transaction_commit_error::origin_denial(
            "authored retirement requires every prior live member and only Retire roles",
        ));
    }
    Ok(())
}
