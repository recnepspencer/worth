//! Pure committed dispatch-outbox owner observation types.

use worth_relational::facade::history::RelationalCommitReceipt;
use worth_relational::facade::transactions::RecordRef;

use crate::domain_computation::application_aftermath::WorthQueryDispatchOutboxRecord;
use crate::domain_computation::primary_graph::WorthQueryCommittedProductPublication;

pub mod work;
pub use work::WorthQueryCommittedDispatchOutboxReadWork;

/// Fresh Query-provider observation of one authoritative Relational outbox row.
///
/// This is read-only owner evidence, not publication authority. Only the
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

/// Why Query could not establish an authoritative committed outbox row.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryCommittedDispatchOutboxReadDenial {
    ReadExecution(
        Result<
            crate::domain_computation::WorthQueryProviderSessionDenialKind,
            crate::domain_computation::WorthQueryProviderSessionControlStopKind,
        >,
    ),
    ForeignPhase,
    ForeignRuntime,
    Missing,
    AmbiguousCorrelation,
    PendingPublication,
    CommittedIndexUnavailable,
    WrongRecordKind,
    NotAuthoritative,
    ExactCommitUnavailable,
    ActiveSnapshotCapacityExhausted {
        maximum_active_snapshots: usize,
    },
    SnapshotIdentityExhausted,
    Malformed,
    CommitMismatch,
    RecordMismatch,
}

impl WorthQueryCommittedDispatchOutboxObservation {
    /// Discard this observation's duplicate physical receipt, preserving exact
    /// immutable meaning. The provider retains and retires the actual original.
    pub fn into_descriptive(mut self) -> Self {
        self.committed_product_publication = self.committed_product_publication.descriptive();
        self
    }

    pub const fn seal(
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
    pub const fn seal(
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
