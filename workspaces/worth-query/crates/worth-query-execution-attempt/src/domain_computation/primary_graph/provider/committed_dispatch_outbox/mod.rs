use worth_foundational::facade::AspectValue;
use worth_query_execution_lineage::facade::ports::CommitReceiptInternalPort as _;
use worth_query_execution_lineage::facade::ports::PrimaryGraphIntegrationHandleInternalPort as _;
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
use restoration::{decode_fields, required_fields};

pub use worth_query_execution_lineage::facade::primary_graph::provider::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial as Denial;
pub use worth_query_execution_lineage::facade::primary_graph::provider::committed_dispatch_outbox::*;
pub mod correlation;
pub mod original_attempt;
pub mod original_row;
pub mod readmission;
pub mod record_read;
pub mod recovered;
pub mod recovery;
pub mod restoration;
#[cfg(test)]
mod restoration_tests;

use record_read::CommittedOutboxRead;
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
            crate::domain_computation::primary_graph::relational_execution_kind(cause, None),
        ),
        Kind::EntityKindMismatch => Denial::WrongRecordKind,
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub fn committed_dispatch_outbox(
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

    pub fn observe_expected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &super::WorthQueryCommittedDispatchOutboxBinding,
        expected_commit: &worth_relational::facade::history::RelationalCommitReceipt,
        expected_runtime: u64,
    ) -> Result<WorthQueryCommittedDispatchOutboxOwnerObservation, Denial> {
        let layout = self.graph.layout_ref().provider_dispatch_outbox().clone();
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

impl super::WorthQueryPrimaryGraphProvider {
    #[doc(hidden)]
    pub fn observe_expected_for_test(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &super::super::WorthQueryCommittedDispatchOutboxBinding,
        expected_commit: &worth_relational::facade::history::RelationalCommitReceipt,
        expected_runtime: u64,
    ) -> Result<WorthQueryCommittedDispatchOutboxOwnerObservation, Denial> {
        Self::observe_expected(self, execution, binding, expected_commit, expected_runtime)
    }
}
