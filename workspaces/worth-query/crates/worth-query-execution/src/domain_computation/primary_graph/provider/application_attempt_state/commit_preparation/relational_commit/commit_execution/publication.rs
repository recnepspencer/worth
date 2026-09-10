//! Required publication after the authoritative commit has succeeded.

mod committed_application;
mod provider_result;

pub(in crate::domain_computation::primary_graph) use committed_application::WorthQueryPrimaryGraphCommittedApplication;

use super::WorthQueryCommittedApplicationSession;
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
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    committed: WorthQueryCommittedApplicationSession,
    evidence: super::super::WorthQueryPrimaryGraphCommitEvidence,
) -> Result<WorthQueryPublishedApplicationCommit, WorthQueryProviderSessionFailure> {
    let published_snapshot = committed.committed.snapshot.clone();
    let publication = publish_retained(provider, runtime, committed, evidence);
    crate::relational_snapshot_release::release_query_snapshot(runtime, &published_snapshot);
    publication
}

fn publish_retained(
    provider: &WorthQueryPrimaryGraphProvider,
    runtime: &mut worth_relational::facade::runtime::RelationalRuntime,
    committed: WorthQueryCommittedApplicationSession,
    evidence: super::super::WorthQueryPrimaryGraphCommitEvidence,
) -> Result<WorthQueryPublishedApplicationCommit, WorthQueryProviderSessionFailure> {
    let WorthQueryCommittedApplicationSession {
        mut attempt,
        branch,
        before,
        next_basis,
        committed,
        product_publication,
        ..
    } = committed;
    let recovery_reservation = attempt.take_publication_recovery_reservation();
    let changed_record_count = committed.patch().len();
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
    let application = WorthQueryPrimaryGraphCommittedApplication::from_publication(seal);
    provider.install_and_publish_application(
        runtime,
        recovery_reservation,
        WorthQueryPendingApplicationPublication::new(
            attempt,
            branch,
            before,
            next_basis,
            committed,
            application,
            emitted_effect_count,
            outcome_identity,
        ),
    )?;
    Ok(WorthQueryPublishedApplicationCommit { _private: () })
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
