use super::super::UiAllocationStreamPolicy;
use super::{
    UiAllocationCadenceBudget, UiAllocationCadenceKind, UiAllocationCommitTarget,
    UiAllocationEvidenceCadence, UiAllocationPartialSettlementLaw, UiAllocationResolvedCommitLane,
    UiAllocationStreamCollapseLaw, UiAllocationStreamFamily, UiResolvedAllocationStreamPolicy,
};

pub(super) fn resolved_family_policy(
    family: UiAllocationStreamFamily,
) -> UiResolvedAllocationStreamPolicy {
    let policy = UiAllocationStreamPolicy::for_family(family);
    UiResolvedAllocationStreamPolicy {
        commit_lane: match family {
            UiAllocationStreamFamily::ViewportObservation => {
                UiAllocationResolvedCommitLane::ViewportDerived
            }
            UiAllocationStreamFamily::ResizePreview => {
                UiAllocationResolvedCommitLane::ResizePreview
            }
            UiAllocationStreamFamily::DurableResize => {
                UiAllocationResolvedCommitLane::DurableResize
            }
            UiAllocationStreamFamily::TextInput
            | UiAllocationStreamFamily::QueryProjection
            | UiAllocationStreamFamily::ScrollExtentObservation
            | UiAllocationStreamFamily::PortalAnchorObservation
            | UiAllocationStreamFamily::HostMeasurementReplacement => {
                UiAllocationResolvedCommitLane::Ordinary
            }
        },
        target: policy.target(),
        cadence: policy.cadence(),
        budget: policy.budget(),
        evidence_cadence: policy.evidence_cadence(),
        collapse_law: policy.collapse_law(),
        partial_settlement_law: policy.partial_settlement_law(),
    }
}

pub(super) fn join_contract_policies(
    left: UiResolvedAllocationStreamPolicy,
    right: UiResolvedAllocationStreamPolicy,
) -> UiResolvedAllocationStreamPolicy {
    UiResolvedAllocationStreamPolicy {
        commit_lane: join_commit_lane(left.commit_lane, right.commit_lane),
        target: join_target(left.target, right.target),
        cadence: join_cadence(left.cadence, right.cadence),
        budget: UiAllocationCadenceBudget::contract(
            left.budget
                .ingress_window()
                .min(right.budget.ingress_window()),
            left.budget
                .max_resolved_plans()
                .max(right.budget.max_resolved_plans()),
            left.budget
                .max_committed_receipts()
                .max(right.budget.max_committed_receipts()),
            left.budget
                .max_durable_mutations()
                .max(right.budget.max_durable_mutations()),
            left.budget
                .max_lag_frames()
                .min(right.budget.max_lag_frames()),
        )
        .with_max_invalidation_targets(
            left.budget
                .max_invalidation_targets()
                .max(right.budget.max_invalidation_targets()),
        ),
        evidence_cadence: join_evidence(left.evidence_cadence, right.evidence_cadence),
        collapse_law: join_collapse(left.collapse_law, right.collapse_law),
        partial_settlement_law: join_partial_settlement(
            left.partial_settlement_law,
            right.partial_settlement_law,
        ),
    }
}

fn join_commit_lane(
    left: UiAllocationResolvedCommitLane,
    right: UiAllocationResolvedCommitLane,
) -> UiAllocationResolvedCommitLane {
    use UiAllocationResolvedCommitLane::*;
    match (left, right) {
        (DurableResize, ResizePreview) | (ResizePreview, DurableResize) => DragResize,
        (DragResize, ResizePreview | DurableResize)
        | (ResizePreview | DurableResize, DragResize) => DragResize,
        (same, other) if same == other => same,
        _ => Ordinary,
    }
}

fn join_target(
    left: UiAllocationCommitTarget,
    right: UiAllocationCommitTarget,
) -> UiAllocationCommitTarget {
    use UiAllocationCommitTarget::*;
    match (left, right) {
        (PreviewOnly, PreviewOnly) => PreviewOnly,
        (SemanticAndAllocation, _) | (_, SemanticAndAllocation) => SemanticAndAllocation,
        _ => AllocationOnly,
    }
}

fn join_cadence(
    left: UiAllocationCadenceKind,
    right: UiAllocationCadenceKind,
) -> UiAllocationCadenceKind {
    use UiAllocationCadenceKind::*;
    match (left, right) {
        (Immediate, _) | (_, Immediate) => Immediate,
        (Threshold, _) | (_, Threshold) => Threshold,
        (CoalescedWindow, _) | (_, CoalescedWindow) => CoalescedWindow,
        _ => Terminal,
    }
}

fn join_evidence(
    left: UiAllocationEvidenceCadence,
    right: UiAllocationEvidenceCadence,
) -> UiAllocationEvidenceCadence {
    use UiAllocationEvidenceCadence::*;
    match (left, right) {
        (EveryInput, _) | (_, EveryInput) => EveryInput,
        (PerResolvedFrame, _) | (_, PerResolvedFrame) => PerResolvedFrame,
        _ => PerCommittedReceipt,
    }
}

fn join_collapse(
    left: UiAllocationStreamCollapseLaw,
    right: UiAllocationStreamCollapseLaw,
) -> UiAllocationStreamCollapseLaw {
    use UiAllocationStreamCollapseLaw::*;
    match (left, right) {
        (PreserveEveryInput, _) | (_, PreserveEveryInput) => PreserveEveryInput,
        (TerminalOnly, _) | (_, TerminalOnly) => TerminalOnly,
        (CoalesceWithinResolvedFrame, _) | (_, CoalesceWithinResolvedFrame) => {
            CoalesceWithinResolvedFrame
        }
        _ => LatestWinsWithinResolvedFrame,
    }
}

fn join_partial_settlement(
    left: UiAllocationPartialSettlementLaw,
    right: UiAllocationPartialSettlementLaw,
) -> UiAllocationPartialSettlementLaw {
    if left == UiAllocationPartialSettlementLaw::StaleButBounded
        || right == UiAllocationPartialSettlementLaw::StaleButBounded
    {
        UiAllocationPartialSettlementLaw::StaleButBounded
    } else {
        UiAllocationPartialSettlementLaw::NotApplicable
    }
}
