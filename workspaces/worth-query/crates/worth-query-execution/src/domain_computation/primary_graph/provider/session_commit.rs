//! Named responsibilities for committing one prepared provider session.

mod evidence;
mod outbox_commit;
mod receipt_basis_retention;

pub(crate) use super::application_attempt_state::WorthQueryRetainedPreImageSeal;
pub(in crate::domain_computation::primary_graph) use super::application_attempt_state::{
    WorthQueryMutationWorkCommitSeal, WorthQueryPreImageRetentionWork,
};
pub(in crate::domain_computation::primary_graph) use evidence::WorthQueryCompletedCommitEvidenceStore;
pub(in crate::domain_computation) use outbox_commit::WorthQueryCommittedDispatchOutboxBinding;
pub(in crate::domain_computation::primary_graph) use outbox_commit::{
    WorthQueryCommittedDispatchOutboxBindingDenial, WorthQueryCommittedDispatchOutboxReceiptSeal,
    WorthQueryCommittedDispatchOutboxResolution,
};
pub(in crate::domain_computation::primary_graph) use receipt_basis_retention::{
    WorthQueryReceiptBasisRetentionStore, WorthQueryRetainedApplicationCommitBasis,
};

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::{
    WorthQueryProviderSessionFailure, WorthQueryProviderSessionProtocolStage,
};

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
        allocation_policy,
    )
}

pub(super) fn provider_failure(
    stage: WorthQueryProviderSessionProtocolStage,
    detail: &'static str,
) -> WorthQueryProviderSessionFailure {
    WorthQueryProviderSessionFailure::new(
        crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
        stage,
        detail,
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
}

pub(super) fn snapshot_admission_failure(
    stage: WorthQueryProviderSessionProtocolStage,
    denial: crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial,
    detail: &'static str,
) -> WorthQueryProviderSessionFailure {
    let kind = match denial {
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => crate::domain_computation::WorthQueryProviderSessionDenialKind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionCapacityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::RetentionIdentityExhausted
        }
        crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
            crate::domain_computation::WorthQueryProviderSessionDenialKind::SnapshotIdentityExhausted
        }
        _ => crate::domain_computation::WorthQueryProviderSessionDenialKind::ProviderRejected,
    };
    WorthQueryProviderSessionFailure::new(
        kind,
        stage,
        detail,
        crate::domain_computation::WorthQueryProviderSessionProtocolCounters::default(),
    )
}
