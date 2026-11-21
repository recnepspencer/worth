//! Fresh Relational owner reads for committed dispatch-outbox rows.

use worth_foundational::facade::AspectValue;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::transactions::RecordRef;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::{
    WorthQueryDispatchOutboxLayout, WorthQueryDispatchOutboxRecord,
    WorthQueryDispatchOutboxRestoredFields,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryCommittedProductPublication,
};

mod correlation;
mod original_attempt;
mod original_row;
mod readmission;
pub(super) mod recovered;
mod recovery;
pub(crate) mod restoration;

use restoration::{decode_fields, required_fields};

pub use worth_query_execution_lineage::facade::primary_graph::provider::committed_dispatch_outbox::{
    WorthQueryCommittedDispatchOutboxObservation,
    WorthQueryCommittedDispatchOutboxOwnerObservation,
    WorthQueryCommittedDispatchOutboxReadDenial,
    WorthQueryCommittedDispatchOutboxReadWork,
};

use WorthQueryCommittedDispatchOutboxReadDenial as Denial;

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn committed_dispatch_outbox(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let Some(binding) = receipt.committed_dispatch_outbox() else {
            return Ok(None);
        };
        self.observe_held_original(
            execution,
            binding,
            receipt.commit_reference(),
            receipt.provider_runtime_instance_id(),
            receipt.committed_product_publication(),
        )
        .map(|owner| {
            WorthQueryCommittedDispatchOutboxObservation::seal(
                owner,
                receipt.committed_product_publication().clone(),
            )
        })
        .map(Some)
    }

    fn observe_expected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &super::WorthQueryCommittedDispatchOutboxBinding,
        expected_commit: &worth_relational::facade::history::RelationalCommitReceipt,
        expected_runtime: u64,
    ) -> Result<WorthQueryCommittedDispatchOutboxOwnerObservation, Denial> {
        let layout = self.graph.layout.provider_dispatch_outbox().clone();
        let expected_commit = expected_commit.clone();
        let retained_basis = self
            .receipt_basis_retention
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .acquire(expected_commit.commit_id)
            .ok_or(Denial::ExactCommitUnavailable)?;
        self.graph.with_runtime(execution, |runtime| {
            crate::domain_computation::execution_runtime::product_world::record_installed_source_read_for_test();
            CommittedOutboxRead {
                runtime,
                layout: &layout,
                record: binding.record(),
                record_ref: binding.record_ref(),
                expected_commit: &expected_commit,
                expected_runtime,
                retained_basis: retained_basis.lease(),
            }
            .resolve(execution, )
        })
    }
}

fn map_retained_snapshot_denial(
    denial: worth_relational::facade::runtime::RelationalRetainedCommitSnapshotDenial,
) -> Denial {
    use worth_relational::facade::runtime::RelationalRetainedCommitSnapshotDenialKind as Kind;
    match denial.kind() {
        Kind::ForeignRuntime => Denial::ForeignRuntime,
        Kind::VersionUnavailable | Kind::SnapshotNotRetained => Denial::ExactCommitUnavailable,
        Kind::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        } => Denial::ActiveSnapshotCapacityExhausted {
            maximum_active_snapshots,
        },
        Kind::SnapshotIdentityExhausted => Denial::SnapshotIdentityExhausted,
        Kind::BranchMismatch | Kind::CommitMismatch | Kind::SnapshotBindingMismatch => {
            Denial::CommitMismatch
        }
        Kind::Execution(cause) => Denial::ReadExecution(
            crate::facade::primary_graph::relational_execution_kind(cause, None),
        ),
        Kind::EntityKindMismatch => Denial::WrongRecordKind,
    }
}

impl<Schema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    /// Uses the correlation only to select a candidate. The provider then
    /// re-reads that candidate from the exact retained Relational commit.
    pub(in crate::domain_computation) fn observe_committed_dispatch_outbox_for_correlation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        self.runtime
            .primary_provider
            .committed_dispatch_outbox_for_correlation(
                execution,
                correlation,
                &self.runtime.product_runtime.owner.observation_port(),
                &self.runtime.product_runtime.owner.inspection_port(),
            )
    }

    /// Selects a candidate by correlation and verifies its exact original
    /// Native row and World publication. This is read-only owner evidence;
    /// it grants neither dispatch nor fresh publication delivery.
    pub fn observe_committed_dispatch_outbox_by_correlation(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        correlation: &crate::domain_computation::application_aftermath::ExternalEffectCorrelationIdentity,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        let execution = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        self.observe_committed_dispatch_outbox_for_correlation(&execution, correlation)
    }

    /// Reads this receipt's outbox from a fresh provider-owned Relational view.
    pub fn observe_committed_dispatch_outbox(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let phase_request = phase
            .execution_request_for(&self.runtime.product_runtime)
            .map_err(|_| Denial::ForeignPhase)?;
        self.observe_committed_dispatch_outbox_with_request(&phase_request, receipt)
    }

    pub(in crate::domain_computation::primary_graph) fn observe_committed_dispatch_outbox_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        self.runtime
            .primary_provider
            .committed_dispatch_outbox(execution, receipt)
    }
}

mod record_read;
use record_read::CommittedOutboxRead;

mod court_doors_for_test;

#[cfg(test)]
mod restoration_tests;
