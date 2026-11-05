//! Fresh Relational owner reads for committed dispatch-outbox rows.

use worth_foundational::facade::AspectValue;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::transactions::RecordRef;

use super::WorthQueryPrimaryGraphProvider;
use {crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxLayout, crate::domain_computation::vocabulary::application_aftermath::outbox::WorthQueryDispatchOutboxRecord};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryCommittedProductPublication,
};

mod correlation;

pub(crate) mod restoration;

mod work;

use restoration::{required_fields, restore_record};

pub use work::WorthQueryCommittedDispatchOutboxReadWork;

/// Fresh Query-provider observation of one authoritative Relational outbox row.
///
/// This is read-only owner evidence, not publication authority. Only this
/// provider owner module can seal production observations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxObservation {
    owner: WorthQueryCommittedDispatchOutboxOwnerObservation,
    committed_product_publication: WorthQueryCommittedProductPublication,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryCommittedDispatchOutboxOwnerObservation {
    record: WorthQueryDispatchOutboxRecord,
    commit: RelationalCommitReceipt,
    record_ref: RecordRef,
    relational_runtime_instance_id: u64,
    work: WorthQueryCommittedDispatchOutboxReadWork,
}


use WorthQueryCommittedDispatchOutboxReadDenial as Denial;

impl WorthQueryCommittedDispatchOutboxObservation {
    const fn seal(
        owner: WorthQueryCommittedDispatchOutboxOwnerObservation,
        committed_product_publication: WorthQueryCommittedProductPublication,
    ) -> Self {
        Self {
            owner,
            committed_product_publication,
        }
    }

    pub const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        self.owner.record()
    }

    pub const fn commit_reference(&self) -> &RelationalCommitReceipt {
        self.owner.commit_reference()
    }

    pub const fn committed_product_publication(&self) -> &WorthQueryCommittedProductPublication {
        &self.committed_product_publication
    }

    pub const fn record_ref(&self) -> &RecordRef {
        self.owner.record_ref()
    }

    pub const fn relational_runtime_instance_id(&self) -> u64 {
        self.owner.relational_runtime_instance_id()
    }

    pub const fn work(&self) -> WorthQueryCommittedDispatchOutboxReadWork {
        self.owner.work()
    }

    /// Corruption probe for proving the World axis is checked independently
    /// from the Relational runtime axis.
    pub fn with_relational_runtime_instance_for_test(mut self, runtime: u64) -> Self {
        self.owner.relational_runtime_instance_id = runtime;
        self
    }

    /// Substitution probe for proving the exact World publication is paired
    /// with its own Relational owner row before transport admission.
    pub fn with_product_publication_for_test(
        mut self,
        publication: WorthQueryCommittedProductPublication,
    ) -> Self {
        self.committed_product_publication = publication;
        self
    }
}

impl WorthQueryCommittedDispatchOutboxOwnerObservation {
    const fn seal(
        record: WorthQueryDispatchOutboxRecord,
        commit: RelationalCommitReceipt,
        record_ref: RecordRef,
        relational_runtime_instance_id: u64,
        work: WorthQueryCommittedDispatchOutboxReadWork,
    ) -> Self {
        Self {
            record,
            commit,
            record_ref,
            relational_runtime_instance_id,
            work,
        }
    }

    pub const fn record(&self) -> &WorthQueryDispatchOutboxRecord {
        &self.record
    }

    pub const fn commit_reference(&self) -> &RelationalCommitReceipt {
        &self.commit
    }

    pub const fn record_ref(&self) -> &RecordRef {
        &self.record_ref
    }

    pub const fn relational_runtime_instance_id(&self) -> u64 {
        self.relational_runtime_instance_id
    }

    pub const fn work(&self) -> WorthQueryCommittedDispatchOutboxReadWork {
        self.work
    }
}

impl WorthQueryPrimaryGraphProvider {
    pub(in crate::domain_computation::primary_graph) fn committed_dispatch_outbox(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let Some(binding) = receipt.committed_dispatch_outbox() else {
            return Ok(None);
        };
        self.observe_expected(
            execution,
            binding,
            receipt.commit_reference(),
            receipt.provider_runtime_instance_id(),
        )
        .map(|owner| {
            WorthQueryCommittedDispatchOutboxObservation::seal(
                owner,
                receipt.committed_product_publication().clone(),
            )
        })
        .map(Some)
    }

    pub(in crate::domain_computation::primary_graph) fn committed_dispatch_outbox_for_binding(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleBinding,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        let committed = binding.committed_dispatch_outbox().ok_or(Denial::Missing)?;
        self.observe_expected(
            execution,
            committed,
            binding.commit_reference(),
            binding.runtime_instance_id(),
        )
        .map(|owner| {
            WorthQueryCommittedDispatchOutboxObservation::seal(
                owner,
                binding.committed_product_publication().clone(),
            )
        })
    }

    fn observe_expected(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        binding: &crate::domain_computation::vocabulary::primary_graph::outbox_commit::WorthQueryCommittedDispatchOutboxBinding,
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
        self.graph.with_runtime_mut(execution, |runtime| {
            crate::domain_computation::execution_runtime::product_world::record_installed_source_read_for_test();
            CommittedOutboxRead {
                runtime,
                layout: &layout,
                binding,
                expected_commit: &expected_commit,
                expected_runtime,
                retained_basis: &retained_basis,
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

pub(in crate::domain_computation) trait CommittedDispatchOutboxScopedOps<Schema>:
    crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    fn observe_committed_dispatch_outbox_for_correlation(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        correlation: &crate::domain_computation::vocabulary::application_aftermath::correlation::ExternalEffectCorrelationIdentity,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        self.primary_provider()
            .committed_dispatch_outbox_for_correlation(execution, correlation)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema> + ?Sized,
    > CommittedDispatchOutboxScopedOps<Schema> for HostRuntime
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
}

pub trait CommittedDispatchOutboxOps<Schema>:
    crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    fn observe_committed_dispatch_outbox(
        &self,
        phase: &crate::domain_computation::request_execution::WorthQueryAdvancementPhase<'_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let phase_request = phase
            .execution_request_for(self.product_runtime())
            .map_err(|_| Denial::ForeignPhase)?;
        self.observe_committed_dispatch_outbox_with_request(&phase_request, receipt)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema> + ?Sized,
    > CommittedDispatchOutboxOps<Schema> for HostRuntime
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
}

pub(in crate::domain_computation::primary_graph) trait ProviderCommittedDispatchOutboxScopedOps<
    Schema,
>:
    crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema> where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    fn observe_committed_dispatch_outbox_with_request(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        self.primary_provider()
            .committed_dispatch_outbox(execution, receipt)
    }
}

impl<
        Schema,
        HostRuntime: crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema> + ?Sized,
    > ProviderCommittedDispatchOutboxScopedOps<Schema> for HostRuntime
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
}

mod record_read;
use record_read::CommittedOutboxRead;
use crate::domain_computation::vocabulary::primary_graph::committed_dispatch_outbox::WorthQueryCommittedDispatchOutboxReadDenial;

#[cfg(test)]
#[path = "committed_dispatch_outbox/restoration_tests.rs"]
mod restoration_tests;

mod court_doors_for_test;
