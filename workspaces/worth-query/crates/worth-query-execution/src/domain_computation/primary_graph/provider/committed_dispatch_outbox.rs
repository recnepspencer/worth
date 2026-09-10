//! Fresh Relational owner reads for committed dispatch-outbox rows.

use worth_foundational::facade::AspectValue;
use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::runtime::{ProjectionAspectRequirement, ProjectionAspectScope};
use worth_relational::facade::transactions::RecordRef;

use super::WorthQueryPrimaryGraphProvider;
use crate::domain_computation::application_aftermath::{
    WorthQueryDispatchOutboxLayout, WorthQueryDispatchOutboxRecord,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryCommittedProductPublication,
};

#[cfg(test)]
mod layout_tests;
#[cfg(test)]
mod owner_test_support;
mod restoration;
#[cfg(test)]
mod test_support;
mod work;

use restoration::{required_fields, restore_record};
#[cfg(test)]
pub(in crate::domain_computation) use test_support::{
    commit_distinct_records_and_admit_fixture, commit_observe_and_admit_fixture,
    commit_observe_and_admit_twice_fixture,
};
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
struct WorthQueryCommittedDispatchOutboxOwnerObservation {
    record: WorthQueryDispatchOutboxRecord,
    commit: RelationalCommitReceipt,
    record_ref: RecordRef,
    relational_runtime_instance_id: u64,
    work: WorthQueryCommittedDispatchOutboxReadWork,
}

/// Why Query could not establish an authoritative committed outbox row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCommittedDispatchOutboxReadDenial {
    ForeignRuntime,
    Missing,
    WrongRecordKind,
    NotAuthoritative,
    ExactCommitUnavailable,
    ActiveSnapshotCapacityExhausted { maximum_active_snapshots: usize },
    SnapshotIdentityExhausted,
    Malformed,
    CommitMismatch,
    RecordMismatch,
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
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn with_relational_runtime_instance_for_test(
        mut self,
        runtime: u64,
    ) -> Self {
        self.owner.relational_runtime_instance_id = runtime;
        self
    }

    /// Substitution probe for proving the exact World publication is paired
    /// with its own Relational owner row before transport admission.
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn with_product_publication_for_test(
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
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        let Some(binding) = receipt.committed_dispatch_outbox() else {
            return Ok(None);
        };
        self.observe_expected(
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
        binding: &crate::domain_computation::application_aftermath::WorthQueryRecoveryHandleBinding,
    ) -> Result<WorthQueryCommittedDispatchOutboxObservation, Denial> {
        let committed = binding.committed_dispatch_outbox().ok_or(Denial::Missing)?;
        self.observe_expected(
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
        self.graph.with_runtime_mut(|runtime| {
            CommittedOutboxRead {
                runtime,
                layout: &layout,
                binding,
                expected_commit: &expected_commit,
                expected_runtime,
                retained_basis: &retained_basis,
            }
            .resolve()
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
        Kind::EntityKindMismatch => Denial::WrongRecordKind,
    }
}

impl<Schema> super::super::WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: worth_query_installation::facade::ApplicationSchema,
{
    /// Reads this receipt's outbox from a fresh provider-owned Relational view.
    pub fn observe_committed_dispatch_outbox(
        &self,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<Option<WorthQueryCommittedDispatchOutboxObservation>, Denial> {
        self.primary_provider.committed_dispatch_outbox(receipt)
    }
}

struct CommittedOutboxRead<'a> {
    runtime: &'a mut worth_relational::facade::runtime::RelationalRuntime,
    layout: &'a WorthQueryDispatchOutboxLayout,
    binding: &'a super::WorthQueryCommittedDispatchOutboxBinding,
    expected_commit: &'a worth_relational::facade::history::RelationalCommitReceipt,
    expected_runtime: u64,
    retained_basis: &'a super::WorthQueryRetainedApplicationCommitBasis,
}

impl CommittedOutboxRead<'_> {
    fn resolve(&mut self) -> Result<WorthQueryCommittedDispatchOutboxOwnerObservation, Denial> {
        let RecordRef::Entity(entity_id) = self.binding.record_ref() else {
            return Err(Denial::NotAuthoritative);
        };
        let entity_id = *entity_id;
        let (created_at, values, owner_work) = self.read_record(entity_id)?;
        let record = restore_record(values)?;
        if &record != self.binding.record() {
            return Err(Denial::RecordMismatch);
        }
        let committed = self
            .runtime
            .history()
            .historical_committed_version(created_at)
            .ok_or(Denial::NotAuthoritative)?;
        if committed.commit() != self.expected_commit {
            return Err(Denial::CommitMismatch);
        }
        Ok(WorthQueryCommittedDispatchOutboxOwnerObservation::seal(
            record,
            committed.commit().clone(),
            RecordRef::Entity(entity_id),
            self.expected_runtime,
            WorthQueryCommittedDispatchOutboxReadWork::from_owner(owner_work),
        ))
    }

    fn read_record(
        &mut self,
        entity_id: worth_relational::facade::identity::EntityId,
    ) -> Result<
        (
            worth_relational::facade::identity::VersionId,
            Vec<AspectValue>,
            worth_relational::facade::runtime::RelationalRetainedCommitProjectionWork,
        ),
        Denial,
    > {
        let fields = required_fields(self.layout)?;
        let aspect = self
            .layout
            .correlation_locator
            .aspect()
            .aspect_key()
            .clone();
        let scope =
            ProjectionAspectScope::from_requirements([ProjectionAspectRequirement::fields(
                aspect.clone(),
                fields.clone(),
            )]);
        let projection = self
            .runtime
            .snapshots()
            .project_retained_entity_for_commit(
                self.expected_runtime,
                self.expected_commit,
                self.retained_basis.lease(),
                entity_id,
                self.layout.entity_kind,
                scope,
                |record| {
                    Some(
                        fields
                            .iter()
                            .map(|field| record.aspect_field_value(&aspect, field).cloned())
                            .collect::<Option<Vec<_>>>()
                            .map(|values| (record.created_at_version(), values))
                            .ok_or(Denial::NotAuthoritative),
                    )
                },
            )
            .map_err(map_retained_snapshot_denial)?;
        let (projected, work) = projection.into_parts();
        let (created_at, values) = projected.ok_or(Denial::Missing)??;
        Ok((created_at, values, work))
    }
}

#[cfg(test)]
#[path = "committed_dispatch_outbox_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "committed_dispatch_outbox/restoration_tests.rs"]
mod restoration_tests;

#[cfg(test)]
#[path = "committed_dispatch_outbox/corruption_tests.rs"]
mod corruption_tests;
