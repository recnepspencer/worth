//! Actual resource custody is explicit at each original continuation phase.
use super::*;
use crate::{
    branch::{
        ProductBranchHeadProtection, ProductBranchObservation, ProductBranchReferenceCell,
        ProductBranchReferenceSnapshot, WorldProductMovementPermit,
    },
    durability::{WorldDurableMediumBinding, WorldOccurrenceCustody},
    history::{
        CompositeHistoryCatalog, NativeReadmittedPublicationRecord,
        ProductHeadHistoryProtectionObligation, ReadmittedPublicationRecordBacking,
        ReservedCompositeCommitCapacity,
    },
    retention::{
        HistoryRetentionObligation, ProductHeadRetentionTransfer, PublicationRetentionObligation,
    },
};
use std::sync::Arc;
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::RelationalRecoveredPublicationCustody;

pub(crate) struct ContinuationCommon {
    pub(crate) before: ProductBranchObservation,
    pub(crate) after: ProductBranchReferenceSnapshot,
    pub(crate) attempt: crate::identity::CompositePublicationAttemptIdentity,
    pub(crate) binding: Arc<WorldDurableMediumBinding>,
    pub(crate) cell: ProductBranchReferenceCell,
    pub(crate) occurrence: WorldOccurrenceCustody,
    pub(crate) history: CompositeHistoryCatalog,
    pub(crate) capacity: ReservedCompositeCommitCapacity,
    pub(crate) history_pins: Option<HistoryRetentionObligation>,
    pub(crate) publication_pins: PublicationRetentionObligation,
}

pub(crate) struct PreparedOriginalContinuation {
    pub(crate) step: HomeProgramTransitionStepAssociation,
    pub(crate) plan: [u8; 32],
    pub(crate) association: [u8; 32],
    pub(crate) common: ContinuationCommon,
    pub(crate) record_backing: ReadmittedPublicationRecordBacking,
    pub(crate) operation: crate::lifecycle::owner::RuntimeWorldOperationReservation,
    pub(crate) publication_capacity: crate::lifecycle::owner::ReservedPublicationAttemptCapacity,
    pub(crate) memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl PreparedOriginalContinuation {
    /// All quota checks precede this infallible transfer into the same row.
    pub(crate) fn finish(
        self,
        native: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
    ) -> Box<OriginalHomeTransitionContinuation> {
        Box::new(OriginalHomeTransitionContinuation {
            step: self.step,
            plan: self.plan,
            association: self.association,
            progress: Some(OriginalContinuationProgress::Prepared(
                NativeContinuationWork {
                    common: self.common,
                    native,
                    record_backing: self.record_backing,
                },
            )),
            _operation: self.operation,
            _publication_capacity: self.publication_capacity,
            _memory: self.memory,
        })
    }
}

pub(super) struct NativeContinuationWork {
    pub(super) common: ContinuationCommon,
    pub(super) native: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>,
    pub(super) record_backing: ReadmittedPublicationRecordBacking,
}
pub(super) struct SubmittedContinuationWork {
    pub(super) work: NativeContinuationWork,
    pub(super) permit: WorldProductMovementPermit,
}
pub(super) struct MaterializedContinuationWork {
    pub(super) common: ContinuationCommon,
    pub(super) record: NativeReadmittedPublicationRecord,
    pub(super) permit: WorldProductMovementPermit,
}
pub(super) struct HistoryInstalledContinuationWork {
    pub(super) common: ContinuationCommon,
    pub(super) record: NativeReadmittedPublicationRecord,
    pub(super) permit: WorldProductMovementPermit,
    pub(super) history_head: ProductHeadHistoryProtectionObligation,
    pub(super) publication: WorldReadmittedPublication,
}
pub(super) struct TransferredContinuationWork {
    pub(super) common: ContinuationCommon,
    pub(super) record: NativeReadmittedPublicationRecord,
    pub(super) permit: WorldProductMovementPermit,
    pub(super) history_head: ProductHeadHistoryProtectionObligation,
    pub(super) publication: WorldReadmittedPublication,
    pub(super) transfer: ProductHeadRetentionTransfer,
}
pub(super) struct InstalledContinuationWork {
    pub(super) common: ContinuationCommon,
    pub(super) record: NativeReadmittedPublicationRecord,
    pub(super) permit: Option<WorldProductMovementPermit>,
    pub(super) head: Option<ProductBranchHeadProtection>,
    pub(super) publication: Option<WorldReadmittedPublication>,
}

pub(super) enum OriginalContinuationProgress {
    Prepared(NativeContinuationWork),
    Submitted(SubmittedContinuationWork),
    Acknowledged(SubmittedContinuationWork),
    Materialized(MaterializedContinuationWork),
    HistoryInstalled(HistoryInstalledContinuationWork),
    Transferred(TransferredContinuationWork),
    Installed(InstalledContinuationWork),
    ReopenRequired(InstalledContinuationWork),
    Completed(WorldReadmittedPublication),
    Delivered,
}
