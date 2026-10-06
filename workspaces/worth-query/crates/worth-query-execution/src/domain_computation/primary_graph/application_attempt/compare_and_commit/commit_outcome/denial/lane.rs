//! Commits presented outside the lane their operation's binding requires.

use super::{
    WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialKind,
    WorthQueryApplicationCommitDenialStage,
};

impl WorthQueryApplicationCommitDenial {
    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_transition_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationTransitionRequired,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn delegation_activation_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::DelegationActivationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn capability_revocation_required(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::CapabilityRevocationRequired,
            stage: WorthQueryApplicationCommitDenialStage::DelegationTransition,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_request_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationRequestProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_approval_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationApprovalProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn elevation_close_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::ElevationCloseProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            cause: None,
        }
    }

    pub(in crate::domain_computation::primary_graph::application_attempt) const fn mandatory_review_program_mismatch(
    ) -> Self {
        Self {
            kind: WorthQueryApplicationCommitDenialKind::MandatoryReviewProgramMismatch,
            stage: WorthQueryApplicationCommitDenialStage::ElevationTransition,
            detail: None,
            cause: None,
        }
    }
}
