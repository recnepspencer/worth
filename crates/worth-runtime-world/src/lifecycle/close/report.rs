use crate::branch::OwnerRetirementWork;
use crate::identity::ProductUnpublishedOwnerEffectsIdentity;
use crate::recovery::{
    ProductUnpublishedCause, ProductUnpublishedLiveObligations, ProductUnpublishedNextAction,
};

/// Original cold custody retained by the owner at close. This row is data;
/// the catalog, rather than this report, owns the actual Native capability.
#[derive(Debug)]
pub struct RuntimeWorldRecoveredCustodyReport {
    pub(crate) identity: ProductUnpublishedOwnerEffectsIdentity,
    pub(crate) attempt: crate::identity::CompositePublicationAttemptIdentity,
    pub(crate) commit: crate::identity::CompositeCommitIdentity,
    pub(crate) native_receipt: Option<crate::lifecycle::RecoveredCloseReceipt>,
}
impl RuntimeWorldRecoveredCustodyReport {
    pub fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.identity
    }
    pub fn attempt_identity(&self) -> &crate::identity::CompositePublicationAttemptIdentity {
        &self.attempt
    }
    pub fn reserved_commit_identity(&self) -> &crate::identity::CompositeCommitIdentity {
        &self.commit
    }
    pub fn native_receipt(
        &self,
    ) -> Option<&worth_relational::facade::history::RelationalCommitReceipt> {
        self.native_receipt
            .as_ref()
            .map(crate::lifecycle::RecoveredCloseReceipt::receipt)
    }
}

/// One row per retained obligation the owner exposed instead of discarding.
/// A row is exposure only: it carries no capability to settle or delete the
/// obligation it describes.
#[derive(Debug, Clone)]
pub struct RuntimeWorldRetainedRecordReport {
    identity: ProductUnpublishedOwnerEffectsIdentity,
    cause: ProductUnpublishedCause,
    obligations: ProductUnpublishedLiveObligations,
    next_actions: Vec<ProductUnpublishedNextAction>,
}

impl RuntimeWorldRetainedRecordReport {
    pub const fn identity(&self) -> &ProductUnpublishedOwnerEffectsIdentity {
        &self.identity
    }

    pub const fn cause(&self) -> ProductUnpublishedCause {
        self.cause
    }

    /// The exact component pin pair the record holds or reserved.
    pub const fn live_component_obligations(&self) -> usize {
        self.obligations.component()
    }

    /// The recovery slot plus any reserved history capacity or installed
    /// successor history protection the record still owns.
    pub const fn live_composite_obligations(&self) -> usize {
        self.obligations.composite()
    }

    pub fn next_actions(&self) -> &[ProductUnpublishedNextAction] {
        &self.next_actions
    }
}

/// Terminal artifact of a completed close. Enumerating a retained record is
/// exposure, never settlement, and never a discarded owner obligation.
#[derive(Debug)]
#[must_use = "a close report enumerates obligations the caller must still address"]
pub struct RuntimeWorldCloseReport {
    _retired_close: Option<std::sync::Arc<std::sync::Mutex<super::drain::CloseProgress>>>,
    retained_records: Vec<RuntimeWorldRetainedRecordReport>,
    recovered_custody: Vec<RuntimeWorldRecoveredCustodyReport>,
    _recovered_funding: Option<worth_execution::ExecutionMemoryReservation>,
    outstanding_observations: usize,
    settled_records: usize,
    released_product_head_pins: usize,
    released_observation_pins: usize,
    released_history_pins: usize,
    released_unique_component_pins: usize,
    retired_owner_created_custody: usize,
    outstanding_owner_retirement_work: Vec<OwnerRetirementWork>,
}

/// The counted half of a close report. The drain fills every field from what
/// it actually released; no count is inferred from a budget limit.
#[derive(Debug, Default, Clone, Copy)]
pub(crate) struct RuntimeWorldCloseReleaseCounts {
    pub(crate) settled_records: usize,
    pub(crate) released_product_head_pins: usize,
    pub(crate) released_observation_pins: usize,
    pub(crate) released_history_pins: usize,
    pub(crate) released_unique_component_pins: usize,
    pub(crate) retired_owner_created_custody: usize,
}

impl RuntimeWorldCloseReport {
    pub(in crate::lifecycle) fn matches_original_task(
        &self,
        task: &std::sync::Arc<std::sync::Mutex<super::drain::CloseProgress>>,
    ) -> bool {
        self._retired_close
            .as_ref()
            .is_some_and(|original| std::sync::Arc::ptr_eq(original, task))
    }

    pub(super) fn retain_close_task(
        &mut self,
        task: std::sync::Arc<std::sync::Mutex<super::drain::CloseProgress>>,
    ) {
        self._retired_close = Some(task);
    }

    pub(crate) fn new(
        retained_records: Vec<RuntimeWorldRetainedRecordReport>,
        recovered_custody: Vec<RuntimeWorldRecoveredCustodyReport>,
        recovered_funding: Option<worth_execution::ExecutionMemoryReservation>,
        counts: RuntimeWorldCloseReleaseCounts,
        outstanding_observations: usize,
        outstanding_owner_retirement_work: Vec<OwnerRetirementWork>,
    ) -> Self {
        let RuntimeWorldCloseReleaseCounts {
            settled_records,
            released_product_head_pins,
            released_observation_pins,
            released_history_pins,
            released_unique_component_pins,
            retired_owner_created_custody,
        } = counts;
        Self {
            _retired_close: None,
            retained_records,
            recovered_custody,
            _recovered_funding: recovered_funding,
            outstanding_observations,
            settled_records,
            released_product_head_pins,
            released_observation_pins,
            released_history_pins,
            released_unique_component_pins,
            retired_owner_created_custody,
            outstanding_owner_retirement_work,
        }
    }

    /// Observation obligations still alive at the close snapshot. Clones share
    /// one obligation; callers may release them after close returns.
    pub const fn outstanding_observations(&self) -> usize {
        self.outstanding_observations
    }

    pub fn retained_records(&self) -> &[RuntimeWorldRetainedRecordReport] {
        &self.retained_records
    }
    pub fn recovered_custody(&self) -> &[RuntimeWorldRecoveredCustodyReport] {
        &self.recovered_custody
    }

    pub const fn settled_records(&self) -> usize {
        self.settled_records
    }

    pub const fn released_product_head_pins(&self) -> usize {
        self.released_product_head_pins
    }

    pub const fn released_observation_pins(&self) -> usize {
        self.released_observation_pins
    }

    pub const fn released_history_pins(&self) -> usize {
        self.released_history_pins
    }

    pub const fn released_unique_component_pins(&self) -> usize {
        self.released_unique_component_pins
    }

    pub const fn retired_owner_created_custody(&self) -> usize {
        self.retired_owner_created_custody
    }

    pub fn outstanding_owner_retirement_work(&self) -> &[OwnerRetirementWork] {
        &self.outstanding_owner_retirement_work
    }
}
