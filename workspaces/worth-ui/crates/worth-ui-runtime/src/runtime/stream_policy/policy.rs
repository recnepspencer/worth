use super::UiAllocationStreamFamily;

const VIEWPORT_INVALIDATION_TARGET_CAPACITY: u16 = 128;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationCommitTarget {
    SemanticAndAllocation,
    PreviewOnly,
    AllocationOnly,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationEvidenceCadence {
    EveryInput,
    PerResolvedFrame,
    PerCommittedReceipt,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationCadenceKind {
    Immediate,
    CoalescedWindow,
    Threshold,
    Terminal,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationStreamCollapseLaw {
    PreserveEveryInput,
    CoalesceWithinResolvedFrame,
    LatestWinsWithinResolvedFrame,
    TerminalOnly,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiAllocationPartialSettlementLaw {
    NotApplicable,
    StaleButBounded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationCadenceBudget {
    ingress_window: u16,
    max_resolved_plans: u16,
    max_committed_receipts: u16,
    max_invalidation_targets: u16,
    max_durable_mutations: u16,
    max_lag_frames: u8,
}
impl UiAllocationCadenceBudget {
    pub const fn contract(
        ingress_window: u16,
        max_resolved_plans: u16,
        max_committed_receipts: u16,
        max_durable_mutations: u16,
        max_lag_frames: u8,
    ) -> Self {
        Self {
            ingress_window,
            max_resolved_plans,
            max_committed_receipts,
            max_invalidation_targets: max_committed_receipts,
            max_durable_mutations,
            max_lag_frames,
        }
    }
    pub const fn ingress_window(self) -> u16 {
        self.ingress_window
    }
    pub const fn max_resolved_plans(self) -> u16 {
        self.max_resolved_plans
    }
    pub const fn max_committed_receipts(self) -> u16 {
        self.max_committed_receipts
    }
    pub const fn max_invalidation_targets(self) -> u16 {
        self.max_invalidation_targets
    }
    pub const fn with_max_invalidation_targets(mut self, maximum: u16) -> Self {
        self.max_invalidation_targets = maximum;
        self
    }
    pub const fn max_durable_mutations(self) -> u16 {
        self.max_durable_mutations
    }
    pub const fn max_lag_frames(self) -> u8 {
        self.max_lag_frames
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct UiAllocationStreamPolicy {
    cadence: UiAllocationCadenceKind,
    target: UiAllocationCommitTarget,
    budget: UiAllocationCadenceBudget,
    evidence_cadence: UiAllocationEvidenceCadence,
    collapse_law: UiAllocationStreamCollapseLaw,
    partial_settlement_law: UiAllocationPartialSettlementLaw,
}
impl UiAllocationStreamPolicy {
    pub(in crate::runtime::stream_policy) const fn for_family(
        family: UiAllocationStreamFamily,
    ) -> Self {
        match family {
            UiAllocationStreamFamily::TextInput => Self {
                cadence: UiAllocationCadenceKind::Threshold,
                target: UiAllocationCommitTarget::AllocationOnly,
                budget: UiAllocationCadenceBudget::contract(16, 1, 4, 16, 1),
                evidence_cadence: UiAllocationEvidenceCadence::PerResolvedFrame,
                collapse_law: UiAllocationStreamCollapseLaw::PreserveEveryInput,
                partial_settlement_law: UiAllocationPartialSettlementLaw::NotApplicable,
            },
            UiAllocationStreamFamily::ResizePreview => Self {
                cadence: UiAllocationCadenceKind::CoalescedWindow,
                target: UiAllocationCommitTarget::PreviewOnly,
                budget: UiAllocationCadenceBudget::contract(32, 1, 0, 0, 1)
                    .with_max_invalidation_targets(8),
                evidence_cadence: UiAllocationEvidenceCadence::PerResolvedFrame,
                collapse_law: UiAllocationStreamCollapseLaw::LatestWinsWithinResolvedFrame,
                partial_settlement_law: UiAllocationPartialSettlementLaw::NotApplicable,
            },
            UiAllocationStreamFamily::QueryProjection => Self {
                cadence: UiAllocationCadenceKind::Threshold,
                target: UiAllocationCommitTarget::AllocationOnly,
                budget: UiAllocationCadenceBudget::contract(64, 1, 1, 0, 1),
                evidence_cadence: UiAllocationEvidenceCadence::PerResolvedFrame,
                collapse_law: UiAllocationStreamCollapseLaw::CoalesceWithinResolvedFrame,
                partial_settlement_law: UiAllocationPartialSettlementLaw::StaleButBounded,
            },
            UiAllocationStreamFamily::DurableResize => Self {
                cadence: UiAllocationCadenceKind::CoalescedWindow,
                target: UiAllocationCommitTarget::AllocationOnly,
                budget: UiAllocationCadenceBudget::contract(16, 1, 1, 1, 1),
                evidence_cadence: UiAllocationEvidenceCadence::PerResolvedFrame,
                collapse_law: UiAllocationStreamCollapseLaw::TerminalOnly,
                partial_settlement_law: UiAllocationPartialSettlementLaw::NotApplicable,
            },
            UiAllocationStreamFamily::ViewportObservation => viewport_observation(),
            UiAllocationStreamFamily::ScrollExtentObservation => allocation_observation(),
            UiAllocationStreamFamily::PortalAnchorObservation => allocation_observation(),
            UiAllocationStreamFamily::HostMeasurementReplacement => allocation_observation(),
        }
    }
    pub const fn cadence(self) -> UiAllocationCadenceKind {
        self.cadence
    }
    pub const fn target(self) -> UiAllocationCommitTarget {
        self.target
    }
    pub const fn budget(self) -> UiAllocationCadenceBudget {
        self.budget
    }
    pub const fn evidence_cadence(self) -> UiAllocationEvidenceCadence {
        self.evidence_cadence
    }
    pub const fn collapse_law(self) -> UiAllocationStreamCollapseLaw {
        self.collapse_law
    }
    pub const fn partial_settlement_law(self) -> UiAllocationPartialSettlementLaw {
        self.partial_settlement_law
    }
}

const fn viewport_observation() -> UiAllocationStreamPolicy {
    let source_capacity = crate::runtime::allocation_frame_dispatch::ALLOCATION_FRAME_SOURCE_COUNT;
    UiAllocationStreamPolicy {
        cadence: UiAllocationCadenceKind::CoalescedWindow,
        target: UiAllocationCommitTarget::AllocationOnly,
        budget: UiAllocationCadenceBudget::contract(source_capacity, 1, source_capacity, 0, 1)
            .with_max_invalidation_targets(VIEWPORT_INVALIDATION_TARGET_CAPACITY),
        evidence_cadence: UiAllocationEvidenceCadence::PerResolvedFrame,
        collapse_law: UiAllocationStreamCollapseLaw::CoalesceWithinResolvedFrame,
        partial_settlement_law: UiAllocationPartialSettlementLaw::NotApplicable,
    }
}

const fn allocation_observation() -> UiAllocationStreamPolicy {
    UiAllocationStreamPolicy {
        cadence: UiAllocationCadenceKind::CoalescedWindow,
        target: UiAllocationCommitTarget::AllocationOnly,
        budget: UiAllocationCadenceBudget::contract(16, 1, 1, 0, 1),
        evidence_cadence: UiAllocationEvidenceCadence::PerResolvedFrame,
        collapse_law: UiAllocationStreamCollapseLaw::CoalesceWithinResolvedFrame,
        partial_settlement_law: UiAllocationPartialSettlementLaw::NotApplicable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_policy_has_its_own_bounded_allocation_contract() {
        let viewport =
            UiAllocationStreamPolicy::for_family(UiAllocationStreamFamily::ViewportObservation);
        let generic = UiAllocationStreamPolicy::for_family(
            UiAllocationStreamFamily::HostMeasurementReplacement,
        );
        assert_eq!(viewport.target(), UiAllocationCommitTarget::AllocationOnly);
        assert_eq!(viewport.budget().ingress_window(), 64);
        assert_eq!(viewport.budget().max_committed_receipts(), 64);
        assert_eq!(viewport.budget().max_invalidation_targets(), 128);
        assert_eq!(viewport.budget().max_durable_mutations(), 0);
        assert_eq!(viewport.budget().max_resolved_plans(), 1);
        assert_eq!(viewport.budget().max_lag_frames(), 1);
        assert_eq!(
            viewport.evidence_cadence(),
            UiAllocationEvidenceCadence::PerResolvedFrame
        );
        assert_ne!(viewport.budget(), generic.budget());
    }
}
