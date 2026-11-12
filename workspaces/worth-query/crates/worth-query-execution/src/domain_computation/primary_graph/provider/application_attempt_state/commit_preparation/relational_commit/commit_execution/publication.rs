//! Required publication after the authoritative commit has succeeded.

pub(crate) mod committed_application;
mod provider_result;

pub(in crate::domain_computation::primary_graph) use committed_application::WorthQueryPrimaryGraphCommittedApplication;

use super::{PublishedSnapshotCustody, WorthQueryCommittedApplicationSession};
use crate::domain_computation::primary_graph::application_query::derived_view::{
    PredicatePreparationOwner, PredicatePreparationRetirement,
};
use crate::domain_computation::primary_graph::provider::{
    pending_application_publication::WorthQueryPendingApplicationPublication,
    WorthQueryPrimaryGraphProvider,
};
use crate::domain_computation::WorthQueryProviderSessionFailure;

pub(super) struct WorthQueryCommittedApplicationPublicationSeal {
    runtime_instance_id: u64,
    changed_record_count: usize,
    emitted_effect_count: usize,
    outcome_identity:
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationCommitOutcomeIdentity,
    basis_descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    evidence: super::super::WorthQueryPrimaryGraphCommitEvidence,
    product_publication: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationReceipt,
}

pub(super) struct WorthQueryPublishedApplicationCommit {
    _private: (),
}

pub(super) fn publish(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    committed: WorthQueryCommittedApplicationSession,
    evidence: super::super::WorthQueryPrimaryGraphCommitEvidence,
    predicate: &mut Option<PredicatePreparationOwner>,
    retired: &mut PredicatePreparationRetirement,
) -> Result<WorthQueryPublishedApplicationCommit, WorthQueryProviderSessionFailure> {
    let release_published_snapshot = matches!(
        committed.published_snapshot_custody,
        PublishedSnapshotCustody::LiveOwned
    );
    let native_result = std::sync::Arc::clone(&committed.committed);
    let publication = publish_retained(
        execution, provider, runtime, committed, evidence, predicate, retired,
    );
    if release_published_snapshot {
        crate::relational_snapshot_release::release_query_snapshot(
            runtime,
            &native_result.snapshot,
        );
    }
    publication
}

fn publish_retained(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    mut committed: WorthQueryCommittedApplicationSession,
    evidence: super::super::WorthQueryPrimaryGraphCommitEvidence,
    predicate: &mut Option<PredicatePreparationOwner>,
    retired: &mut PredicatePreparationRetirement,
) -> Result<WorthQueryPublishedApplicationCommit, WorthQueryProviderSessionFailure> {
    committed.finish_group_origins();
    let mut members = std::mem::take(&mut committed.members);
    for member in &mut members {
        let mut session = committed.take_member_session(member);
        if provider
            .application_attempt_operation_control
            .take_withheld_output_witness()
        {
            session.prepared_output_witness = None;
        }
        let evidence =
            super::super::evidence_seal::seal_on_runtime(execution, runtime, &mut session);
        let mut pending = prepare_pending(provider, runtime, session, evidence);
        pending.omit_shared_before_lease();
        member.pending = Some(pending);
    }
    let emissions = committed.group_emissions.take();
    let recovery = committed.attempt.take_publication_recovery_reservation();
    let mut pending = prepare_pending(provider, runtime, committed, evidence);
    pending.retain_members(members, emissions);
    provider.install_and_publish_application(
        execution, runtime, recovery, pending, predicate, retired,
    )?;
    Ok(WorthQueryPublishedApplicationCommit { _private: () })
}

fn prepare_pending(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    committed: WorthQueryCommittedApplicationSession,
    evidence: super::super::WorthQueryPrimaryGraphCommitEvidence,
) -> WorthQueryPendingApplicationPublication {
    let WorthQueryCommittedApplicationSession {
        attempt,
        before,
        next_basis,
        committed,
        product_publication,
        managed_views,
        own_publication,
        group_evidence,
        required_prerequisites,
        mut prepared_lineage_slot,
        prepared_output_witness,
        ..
    } = committed;
    if let (Some(slot), Some(publisher)) = (
        prepared_lineage_slot.as_mut(),
        provider.graph.application_publication.as_ref(),
    ) {
        // Ordinary authored retirement has Native succession custody, but did
        // not run a producer or create a producer checkpoint locator.
        if attempt
            .idempotency()
            .producer_dependency_identity()
            .is_some()
        {
            slot.required_publication = own_publication
                .or_else(|| committed.required_output_publication(&publisher.native));
        }
    }
    if let (Some(witness), Some(slot)) = (prepared_output_witness, prepared_lineage_slot.as_mut())
        && let Some(sealed) = witness.finish(
            evidence.output_correspondence(),
            runtime,
            &committed.snapshot,
        )
    {
        slot.retain_native_output_witness(sealed);
    }
    if let Some(managed_views) = managed_views {
        managed_views.apply(&committed, &next_basis, &product_publication);
    }
    // Native's canonical result counts distinct changed records. A record may
    // have more than one patch, so patch cardinality is not the receipt field.
    let changed_record_count = committed
        .publication()
        .envelope
        .semantic_result()
        .changed_records()
        .len();
    let runtime_instance_id = committed.snapshot.runtime_instance_id();
    let emitted_effect_count = attempt.emitted_effect_count();
    let outcome_identity = attempt.outcome_identity();
    let basis_descriptor = next_basis.descriptor().clone();
    let seal = WorthQueryCommittedApplicationPublicationSeal {
        runtime_instance_id,
        changed_record_count,
        emitted_effect_count,
        outcome_identity,
        basis_descriptor,
        evidence,
        product_publication,
    };
    let mut application = WorthQueryPrimaryGraphCommittedApplication::from_publication(seal);
    if let Some(members) = group_evidence {
        application.retain_coupled_members(members);
    }
    WorthQueryPendingApplicationPublication::new(
        attempt,
        before,
        next_basis,
        committed,
        application,
        required_prerequisites,
        prepared_lineage_slot,
    )
}

pub(super) fn encode(
    provider: &WorthQueryPrimaryGraphProvider,
    published: WorthQueryPublishedApplicationCommit,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    WorthQueryProviderSessionFailure,
> {
    provider_result::encode(provider, published)
}
