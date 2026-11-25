//! Typed refusal and custody-retaining failure types for preview forks and branch retirement.
use std::fmt;

use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::{
    application_contribution::WorthQueryAdvancementDenial,
    primary_graph::{
        WorthQueryApplicationProductBranchCloseDenial, WorthQueryProductBranchAdmissionDenial,
    },
    product::{
        WorthQueryExpectedProductBranchAdoptionDenial, WorthQueryProductBranchCreateError,
        WorthQueryProductBranchCreationNoEffect, WorthQueryProductBranchCreationRecovery,
        WorthQueryProductBranchCreationRelease, WorthQueryProductBranchRecoveryDenial,
    },
};

use super::WorthQueryPreviewFork;

/// Why creating a preview fork was refused, preserving exact native creation
/// recovery custody and terminal receipts without collapsing any case.
#[must_use = "inspect or settle any retained creation recovery custody"]
#[derive(Debug)]
pub enum WorthQueryPreviewForkDenial {
    Advancement(WorthQueryAdvancementDenial),
    Selection(WorthQueryProductBranchAdmissionDenial),
    Create(WorthQueryProductBranchCreateError),
    ContinuationAdvancement {
        denial: WorthQueryAdvancementDenial,
        recovery: WorthQueryProductBranchCreationRecovery,
    },
    Recovery {
        denial: WorthQueryProductBranchRecoveryDenial,
        recovery: WorthQueryProductBranchCreationRecovery,
    },
    NoEffect(WorthQueryProductBranchCreationNoEffect),
    ReleasedOriginal(WorthQueryProductBranchCreationRelease),
}

impl WorthQueryPreviewForkDenial {
    /// Extracts any retained [`WorthQueryProductBranchCreationRecovery`] custody.
    pub fn into_recovery(self) -> Option<WorthQueryProductBranchCreationRecovery> {
        match self {
            Self::Create(WorthQueryProductBranchCreateError::ProductUnpublished(recovery))
            | Self::ContinuationAdvancement { recovery, .. }
            | Self::Recovery { recovery, .. } => Some(recovery),
            _ => None,
        }
    }
}

/// Why adopting a preview fork as the Home's active head was refused.
#[derive(Debug)]
pub enum WorthQueryPreviewAdoptDenial {
    Advancement(WorthQueryAdvancementDenial),
    Adoption(WorthQueryExpectedProductBranchAdoptionDenial),
}

impl WorthQueryPreviewAdoptDenial {
    /// Returns `true` when adoption was refused because the Home's active head
    /// advanced or its committed source basis changed since this preview forked.
    pub fn is_stale_head(&self) -> bool {
        use worth_relational::facade::branch::RelationalBranchBasisDenial as Basis;
        matches!(
            self,
            Self::Adoption(
                WorthQueryExpectedProductBranchAdoptionDenial::HeadMoved
                    | WorthQueryExpectedProductBranchAdoptionDenial::HeadReplaced
                    | WorthQueryExpectedProductBranchAdoptionDenial::SourceBasis(
                        Basis::StaleReferenceGeneration
                            | Basis::WrongBranchLocalTruthVersion
                            | Basis::EmptyCommittedTargetMismatch
                            | Basis::UnavailableRetainedTarget
                    )
            )
        )
    }
}

/// Failed adoption retains the preview fork intact so the caller can inspect
/// the refusal and either retry or discard it.
#[must_use = "retain or discard the preview fork returned by failed adoption"]
pub struct WorthQueryPreviewAdoptFailure<'home, Schema: ApplicationSchema> {
    pub(super) preview: WorthQueryPreviewFork<'home, Schema>,
    pub(super) denial: WorthQueryPreviewAdoptDenial,
}

impl<'home, Schema: ApplicationSchema> WorthQueryPreviewAdoptFailure<'home, Schema> {
    pub fn preview(&self) -> &WorthQueryPreviewFork<'home, Schema> {
        &self.preview
    }

    pub fn denial(&self) -> &WorthQueryPreviewAdoptDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPreviewFork<'home, Schema>,
        WorthQueryPreviewAdoptDenial,
    ) {
        (self.preview, self.denial)
    }
}

impl<Schema: ApplicationSchema> fmt::Debug for WorthQueryPreviewAdoptFailure<'_, Schema> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthQueryPreviewAdoptFailure")
            .field("label", &self.preview.label())
            .field("branch", &self.preview.branch())
            .field("denial", &self.denial)
            .finish()
    }
}

/// Why discarding a preview fork or closing a branch was refused.
#[derive(Debug)]
pub enum WorthQueryPreviewDiscardDenial {
    /// Refused because `branch` is currently the Home's active head.
    ActiveBranchProtected,
    Advancement(WorthQueryAdvancementDenial),
    Close(WorthQueryApplicationProductBranchCloseDenial),
}

/// Failed discard retains the preview fork intact for retry or inspection.
#[must_use = "retain or retry discarding the preview fork"]
pub struct WorthQueryPreviewDiscardFailure<'home, Schema: ApplicationSchema> {
    pub(super) preview: WorthQueryPreviewFork<'home, Schema>,
    pub(super) denial: WorthQueryPreviewDiscardDenial,
}

impl<'home, Schema: ApplicationSchema> WorthQueryPreviewDiscardFailure<'home, Schema> {
    pub fn preview(&self) -> &WorthQueryPreviewFork<'home, Schema> {
        &self.preview
    }

    pub fn denial(&self) -> &WorthQueryPreviewDiscardDenial {
        &self.denial
    }

    pub fn into_parts(
        self,
    ) -> (
        WorthQueryPreviewFork<'home, Schema>,
        WorthQueryPreviewDiscardDenial,
    ) {
        (self.preview, self.denial)
    }
}

impl<Schema: ApplicationSchema> fmt::Debug for WorthQueryPreviewDiscardFailure<'_, Schema> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WorthQueryPreviewDiscardFailure")
            .field("label", &self.preview.label())
            .field("branch", &self.preview.branch())
            .field("denial", &self.denial)
            .finish()
    }
}
