use crate::declaration::stable_text_digest;
use crate::evidence::{
    UiBoundReconciliationPosture, UiBoundReconciliationSolveOrder,
    UiConstraintChildIntrinsicContribution, UiConstraintEqualSharePosture,
    UiConstraintEqualShareSolveOrder, UiConstraintParentAvailableSpace,
    UiConstraintPropagationEdgeFamily, UiConstraintSiblingNegotiationFixedPointPolicy,
    UiConstraintSiblingNegotiationSolveOrder, UiPortalAnchorPlanningInputPosture,
    UiPortalAnchorPlanningInputSolveOrder, UiScrollOwnerPlanningInputPosture,
    UiScrollOwnerPlanningInputSolveOrder, UiViewportPlanningInputPosture,
    UiViewportPlanningInputSolveOrder,
};

pub use super::edge_payload_digests::{
    UiConstraintAxisScope, UiConstraintEqualShareDistributionPolicy, UiConstraintResizeInputPosture,
};

use super::edge_payload_digests::{
    axis_scope_digest, bool_digest, bound_reconciliation_posture_digest,
    bound_reconciliation_solve_order_digest, equal_share_policy_digest, equal_share_posture_digest,
    equal_share_solve_order_digest, portal_posture_digest, portal_solve_order_digest,
    resize_input_posture_digest, scroll_posture_digest, scroll_solve_order_digest,
    sibling_fixed_point_policy_digest, sibling_solve_order_digest, viewport_posture_digest,
    viewport_solve_order_digest,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UiConstraintPropagationEdgePayload {
    ParentAvailableSpace(UiConstraintParentAvailableSpace),
    ChildIntrinsicContribution(UiConstraintChildIntrinsicContribution),
    SiblingNegotiation {
        axis_scope: UiConstraintAxisScope,
        group_identity_digest: u64,
        negotiation_identity_digest: u64,
        fixed_point_policy: UiConstraintSiblingNegotiationFixedPointPolicy,
        solve_order: UiConstraintSiblingNegotiationSolveOrder,
    },
    EqualShareDistribution {
        axis_scope: UiConstraintAxisScope,
        policy: UiConstraintEqualShareDistributionPolicy,
        group_identity_digest: u64,
        distribution_identity_digest: u64,
        solve_order: UiConstraintEqualShareSolveOrder,
        posture: UiConstraintEqualSharePosture,
    },
    BoundedReconciliation {
        axis_scope: UiConstraintAxisScope,
        reconciliation_identity_digest: u64,
        solve_order: UiBoundReconciliationSolveOrder,
        posture: UiBoundReconciliationPosture,
    },
    ViewportInput {
        viewport_identity_digest: u64,
        solve_order: UiViewportPlanningInputSolveOrder,
        posture: UiViewportPlanningInputPosture,
        planning_time_only: bool,
    },
    ScrollViewportInput {
        scroll_identity_digest: u64,
        solve_order: UiScrollOwnerPlanningInputSolveOrder,
        posture: UiScrollOwnerPlanningInputPosture,
        planning_time_only: bool,
    },
    PortalAnchorInput {
        portal_identity_digest: u64,
        solve_order: UiPortalAnchorPlanningInputSolveOrder,
        posture: UiPortalAnchorPlanningInputPosture,
        planning_time_only: bool,
    },
    DurableResizeInput {
        durable_identity_digest: u64,
        axis_scope: UiConstraintAxisScope,
        posture: UiConstraintResizeInputPosture,
        planning_time_only: bool,
    },
}

impl UiConstraintPropagationEdgePayload {
    pub(crate) const fn family(self) -> UiConstraintPropagationEdgeFamily {
        match self {
            Self::ParentAvailableSpace(..) => {
                UiConstraintPropagationEdgeFamily::ParentAvailableSpace
            }
            Self::ChildIntrinsicContribution(..) => {
                UiConstraintPropagationEdgeFamily::ChildIntrinsicContribution
            }
            Self::SiblingNegotiation { .. } => {
                UiConstraintPropagationEdgeFamily::SiblingNegotiation
            }
            Self::EqualShareDistribution { .. } => {
                UiConstraintPropagationEdgeFamily::EqualShareDistribution
            }
            Self::BoundedReconciliation { .. } => {
                UiConstraintPropagationEdgeFamily::BoundedReconciliation
            }
            Self::ViewportInput { .. } => UiConstraintPropagationEdgeFamily::ViewportInput,
            Self::ScrollViewportInput { .. } => {
                UiConstraintPropagationEdgeFamily::ScrollViewportInput
            }
            Self::PortalAnchorInput { .. } => UiConstraintPropagationEdgeFamily::PortalAnchorInput,
            Self::DurableResizeInput { .. } => {
                UiConstraintPropagationEdgeFamily::DurableResizeInput
            }
        }
    }

    pub(crate) fn identity_digest(self) -> u64 {
        match self {
            Self::ParentAvailableSpace(parent_available_space) => {
                stable_text_digest("worth-ui.constraint-edge.payload.parent-available-space")
                    ^ parent_available_space.identity_digest().rotate_left(7)
            }
            Self::ChildIntrinsicContribution(contribution) => {
                stable_text_digest("worth-ui.constraint-edge.payload.child-intrinsic")
                    ^ contribution.identity_digest().rotate_left(7)
            }
            Self::SiblingNegotiation {
                axis_scope,
                group_identity_digest,
                negotiation_identity_digest,
                fixed_point_policy,
                solve_order,
            } => {
                stable_text_digest("worth-ui.constraint-edge.payload.sibling-negotiation")
                    ^ axis_scope_digest(axis_scope).rotate_left(7)
                    ^ group_identity_digest.rotate_left(13)
                    ^ negotiation_identity_digest.rotate_left(19)
                    ^ sibling_fixed_point_policy_digest(fixed_point_policy).rotate_left(23)
                    ^ sibling_solve_order_digest(solve_order).rotate_left(29)
            }
            Self::EqualShareDistribution {
                axis_scope,
                policy,
                group_identity_digest,
                distribution_identity_digest,
                solve_order,
                posture,
            } => {
                stable_text_digest("worth-ui.constraint-edge.payload.equal-share")
                    ^ axis_scope_digest(axis_scope).rotate_left(7)
                    ^ equal_share_policy_digest(policy).rotate_left(13)
                    ^ group_identity_digest.rotate_left(19)
                    ^ distribution_identity_digest.rotate_left(23)
                    ^ equal_share_solve_order_digest(solve_order).rotate_left(29)
                    ^ equal_share_posture_digest(posture).rotate_left(31)
            }
            Self::BoundedReconciliation {
                axis_scope,
                reconciliation_identity_digest,
                solve_order,
                posture,
            } => {
                stable_text_digest("worth-ui.constraint-edge.payload.bounded-reconciliation")
                    ^ axis_scope_digest(axis_scope).rotate_left(7)
                    ^ reconciliation_identity_digest.rotate_left(13)
                    ^ bound_reconciliation_solve_order_digest(solve_order).rotate_left(19)
                    ^ bound_reconciliation_posture_digest(posture).rotate_left(23)
            }
            Self::ViewportInput {
                viewport_identity_digest,
                solve_order,
                posture,
                planning_time_only,
            } => {
                stable_text_digest("worth-ui.constraint-edge.payload.viewport")
                    ^ viewport_identity_digest.rotate_left(7)
                    ^ viewport_solve_order_digest(solve_order).rotate_left(13)
                    ^ viewport_posture_digest(posture).rotate_left(19)
                    ^ bool_digest(planning_time_only).rotate_left(23)
            }
            Self::ScrollViewportInput {
                scroll_identity_digest,
                solve_order,
                posture,
                planning_time_only,
            } => {
                stable_text_digest("worth-ui.constraint-edge.payload.scroll-viewport")
                    ^ scroll_identity_digest.rotate_left(7)
                    ^ scroll_solve_order_digest(solve_order).rotate_left(13)
                    ^ scroll_posture_digest(posture).rotate_left(19)
                    ^ bool_digest(planning_time_only).rotate_left(23)
            }
            Self::PortalAnchorInput {
                portal_identity_digest,
                solve_order,
                posture,
                planning_time_only,
            } => {
                stable_text_digest("worth-ui.constraint-edge.payload.portal-anchor")
                    ^ portal_identity_digest.rotate_left(7)
                    ^ portal_solve_order_digest(solve_order).rotate_left(13)
                    ^ portal_posture_digest(posture).rotate_left(19)
                    ^ bool_digest(planning_time_only).rotate_left(23)
            }
            Self::DurableResizeInput {
                durable_identity_digest,
                axis_scope,
                posture,
                planning_time_only,
            } => {
                stable_text_digest("worth-ui.constraint-edge.payload.durable-resize")
                    ^ durable_identity_digest.rotate_left(5)
                    ^ axis_scope_digest(axis_scope).rotate_left(7)
                    ^ resize_input_posture_digest(posture).rotate_left(13)
                    ^ bool_digest(planning_time_only).rotate_left(17)
            }
        }
    }

    pub const fn parent_available_space(self) -> Option<UiConstraintParentAvailableSpace> {
        match self {
            Self::ParentAvailableSpace(parent_available_space) => Some(parent_available_space),
            Self::ChildIntrinsicContribution(_)
            | Self::SiblingNegotiation { .. }
            | Self::EqualShareDistribution { .. }
            | Self::BoundedReconciliation { .. }
            | Self::ViewportInput { .. }
            | Self::ScrollViewportInput { .. }
            | Self::PortalAnchorInput { .. }
            | Self::DurableResizeInput { .. } => None,
        }
    }

    pub const fn child_intrinsic_contribution(
        self,
    ) -> Option<UiConstraintChildIntrinsicContribution> {
        match self {
            Self::ChildIntrinsicContribution(contribution) => Some(contribution),
            Self::ParentAvailableSpace(_)
            | Self::SiblingNegotiation { .. }
            | Self::EqualShareDistribution { .. }
            | Self::BoundedReconciliation { .. }
            | Self::ViewportInput { .. }
            | Self::ScrollViewportInput { .. }
            | Self::PortalAnchorInput { .. }
            | Self::DurableResizeInput { .. } => None,
        }
    }
}
