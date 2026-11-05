//! Named responsibilities for committing one prepared provider session.

mod evidence;
pub(crate) mod outbox_commit;
pub(crate) mod receipt_basis_retention;


pub(in crate::domain_computation::primary_graph) use super::application_attempt_state::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPreImageRetentionWork,
};
pub(in crate::domain_computation::primary_graph) use evidence::WorthQueryCompletedCommitEvidenceStore;

pub(in crate::domain_computation::primary_graph) use {outbox_commit::WorthQueryCommittedDispatchOutboxReceiptSeal, outbox_commit::WorthQueryCommittedDispatchOutboxResolution};
pub(in crate::domain_computation::primary_graph) use receipt_basis_retention::{
    WorthQueryReceiptBasisRetentionStore, WorthQueryRetainedApplicationCommitBasis,
};

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn commit_prepared_application_in_history(
        &self,
        session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
        history: crate::domain_computation::primary_graph::product_operation::document_history::HistoryProducerCutover<'_>,
        allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
    ) -> Result<
        crate::domain_computation::WorthQueryProviderTerminalDescription,
        crate::domain_computation::WorthQueryProviderSessionCommitStop,
    > {
        self.application_attempt_work.observe_prepared_commit();
        super::application_attempt_state::commit_prepared_application(
            self,
            session,
            Some(history),
            allocation_policy,
        )
    }
}

pub(super) fn commit_prepared_session(
    provider: &WorthQueryPrimaryGraphProvider,
    session: crate::domain_computation::WorthQueryProviderSessionView<'_>,
    allocation_policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    crate::domain_computation::WorthQueryProviderTerminalDescription,
    crate::domain_computation::WorthQueryProviderSessionCommitStop,
> {
    super::application_attempt_state::commit_prepared_application(
        provider,
        session,
        None,
        allocation_policy,
    )
}

pub(super) fn provider_failure(
    stage: WorthQueryProviderSessionProtocolStage,
    detail: &'static str,
) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind::ProviderRejected,
        stage,
        detail,
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
}

pub(super) fn snapshot_admission_failure(
    stage: WorthQueryProviderSessionProtocolStage,
    denial: crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial,
    detail: &'static str,
) -> WorthQueryProviderSessionFailure {
    let kind = match denial {
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted
        }
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        crate::domain_computation::primary_graph::vocabulary::exact_basis_access::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
            crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        _ => crate::domain_computation::vocabulary::provider_session::protocol_denial::WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        stage,
        detail,
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
}
