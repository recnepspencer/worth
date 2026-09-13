use crate::branch::OwnerRetirementWork;
use crate::identity::ProductUnpublishedOwnerEffectsIdentity;
use crate::recovery::{
    ProductUnpublishedCause, ProductUnpublishedLiveObligations, ProductUnpublishedNextAction,
};

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
    pub(crate) fn new(
        identity: ProductUnpublishedOwnerEffectsIdentity,
        cause: ProductUnpublishedCause,
        obligations: ProductUnpublishedLiveObligations,
        next_actions: Vec<ProductUnpublishedNextAction>,
    ) -> Self {
        Self {
            identity,
            cause,
            obligations,
            next_actions,
        }
    }

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
    retained_records: Vec<RuntimeWorldRetainedRecordReport>,
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
    pub(crate) fn new(
        retained_records: Vec<RuntimeWorldRetainedRecordReport>,
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
            retained_records,
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
