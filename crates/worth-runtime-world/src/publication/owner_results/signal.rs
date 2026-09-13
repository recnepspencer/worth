use crate::history::CompositeComponentChangePosture;

use super::super::SignalComponentPlanPosture;

use worth_signal::facade::branch::{SignalBranchAdvanceOutcome, SignalBranchForkOutcome};

/// Exact result of the Signal leg. Advancing and forking are distinct owner
/// operations, so history cannot confuse a created branch with a moved one.
#[derive(Debug)]
pub struct CompositeSignalOwnerResult {
    result: CompositeSignalOwnerResultKind,
}

#[derive(Debug, Clone)]
enum CompositeSignalOwnerResultKind {
    RetainedExact,
    Advanced(std::sync::Arc<SignalBranchAdvanceOutcome>),
    Forked(SignalBranchForkOutcome),
}

impl CompositeSignalOwnerResult {
    /// Exact owner-issued outcome retained by this composite result.
    pub fn advanced_outcome(&self) -> Option<&SignalBranchAdvanceOutcome> {
        match &self.result {
            CompositeSignalOwnerResultKind::Advanced(outcome) => Some(outcome),
            _ => None,
        }
    }
    pub fn fork_outcome(&self) -> Option<&SignalBranchForkOutcome> {
        match &self.result {
            CompositeSignalOwnerResultKind::Forked(outcome) => Some(outcome),
            _ => None,
        }
    }

    pub(super) fn evidence_image(&self) -> Self {
        Self {
            result: self.result.clone(),
        }
    }

    pub(crate) fn retained() -> Self {
        Self {
            result: CompositeSignalOwnerResultKind::RetainedExact,
        }
    }

    pub(crate) fn advanced(result: std::sync::Arc<SignalBranchAdvanceOutcome>) -> Self {
        Self {
            result: CompositeSignalOwnerResultKind::Advanced(result),
        }
    }

    pub(crate) fn forked(result: SignalBranchForkOutcome) -> Self {
        Self {
            result: CompositeSignalOwnerResultKind::Forked(result),
        }
    }

    pub(crate) fn posture(&self) -> CompositeComponentChangePosture {
        match self.result {
            CompositeSignalOwnerResultKind::RetainedExact => {
                CompositeComponentChangePosture::RetainExact
            }
            CompositeSignalOwnerResultKind::Advanced(_)
            | CompositeSignalOwnerResultKind::Forked(_) => {
                CompositeComponentChangePosture::Published
            }
        }
    }

    pub(crate) fn publication_identity(
        &self,
    ) -> Option<crate::history::CompositeSignalPublicationIdentity> {
        match &self.result {
            CompositeSignalOwnerResultKind::RetainedExact => None,
            CompositeSignalOwnerResultKind::Advanced(result) => Some(
                crate::history::CompositeSignalPublicationIdentity::Advanced(
                    result.advanced_basis().admission_identity().clone(),
                ),
            ),
            CompositeSignalOwnerResultKind::Forked(result) => {
                Some(crate::history::CompositeSignalPublicationIdentity::Forked(
                    result.created_basis().admission_identity().clone(),
                ))
            }
        }
    }

    pub(crate) fn matches_plan(&self, posture: SignalComponentPlanPosture) -> bool {
        match posture {
            SignalComponentPlanPosture::RetainExact => {
                self.posture() == CompositeComponentChangePosture::RetainExact
            }
            SignalComponentPlanPosture::AdvanceExact => {
                matches!(self.result, CompositeSignalOwnerResultKind::Advanced(_))
            }
        }
    }

    /// Whether this result is the exact evidence the creation plan asked the
    /// Signal owner to produce.
    pub(crate) fn matches_creation_plan(
        &self,
        plan: &crate::branch::SignalBranchCreationPlan,
    ) -> bool {
        match plan {
            crate::branch::SignalBranchCreationPlan::ReuseExact => {
                matches!(self.result, CompositeSignalOwnerResultKind::RetainedExact)
            }
            crate::branch::SignalBranchCreationPlan::ForkExact { .. } => {
                matches!(self.result, CompositeSignalOwnerResultKind::Forked(_))
            }
        }
    }

    /// Whether this result is honest evidence for a creation that stopped
    /// partway. A leg the denial never reached is untouched; a leg that did
    /// move must still be exactly what its plan asked for.
    pub(crate) fn matches_partial_creation_plan(
        &self,
        plan: &crate::branch::SignalBranchCreationPlan,
    ) -> bool {
        matches!(self.result, CompositeSignalOwnerResultKind::RetainedExact)
            || self.matches_creation_plan(plan)
    }
}
