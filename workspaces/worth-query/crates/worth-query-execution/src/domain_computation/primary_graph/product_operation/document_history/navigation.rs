//! Owned process-local navigation attempts; canonical truth stays with Native and World.
mod admission;
mod control;
mod custody;
mod disposal;
mod eligibility;
pub(crate) mod output_authority;
mod output_completion;
mod output_correspondence;
mod output_coverage;
mod output_custody;
mod output_lookup;
mod output_profile;
pub(in crate::domain_computation::primary_graph) mod output_verification;
mod restored_verification;
pub(in crate::domain_computation::primary_graph) use output_lookup::{
    HistoryOutputLookup, HistoryOutputLookupPhase,
};
mod output_source;
mod pending_capacity;
pub(super) use pending_capacity::PendingNavigationCapacity;
mod progression;
mod publication;
mod registration;
mod selection;
mod target;
mod transition;

#[doc(hidden)]
pub use control::WorthQueryHistoryNavigationPauseForTest;
pub use custody::WorthQueryHistoryNavigationPosture;
use custody::{NavigationAttempt, NavigationPhase, NavigationProgress, SharedNavigationAttempt};
pub(super) use disposal::NavigationDisposal;
pub use output_authority::WorthQueryHistoryOutputAuthority;
pub(in crate::domain_computation::primary_graph) use output_authority::{
    HistoryBorrowedReadStop, HistoryProducerCutover,
};
pub use output_completion::{
    WorthQueryCompletedHistoryOutputs, WorthQueryHistoryOutputQualificationDenial,
    WorthQueryRetainedHistoryOutputTree,
};
pub use output_profile::{
    WorthQueryHistoryOutputNavigationAdvance, WorthQueryHistoryOutputProfile,
};
pub use output_source::WorthQueryHistoryRootOutputAdmissionDenial;
pub(in crate::domain_computation::primary_graph) use output_source::{
    HistoryDependentParent, HistoryDependentSource,
};
pub use output_source::{WorthQueryAdmittedHistoryProgramOutput, WorthQueryHistoryOutputSource};
pub use output_source::{
    WorthQueryHistoryDependentOutputRead, WorthQueryHistoryOutputDiscovery,
    WorthQueryVerifiedHistoryParent,
};
pub use output_source::{
    WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryProgramOutputAdvance,
    WorthQuerySettledHistoryProgramOutput,
};
pub(super) use registration::NavigationJournal;
pub use selection::{WorthQueryHistoryNavigationCompletion, WorthQueryHistorySelection};
pub use target::WorthQueryHistoryNavigationIntent;

use super::{selection::WeakHistorySelectionCell, WorthQueryDocumentHistoryDenial};
use std::sync::Arc;

pub use worth_query_execution_query::facade::domain_computation::primary_graph::product_operation::document_history::{
    WorthQueryHistoryNavigationDenial, WorthQueryHistoryNavigationEligibilityDenial,
};

/// A live attempt handle, issued only after fresh READ, metadata and NAV.
/// The document's registry retains actual candidate custody if this handle is dropped.
#[must_use = "advance or explicitly discard the registered navigation attempt"]
pub struct WorthQueryDocumentHistoryNavigation<'runtime, S> {
    application:
        crate::domain_computation::primary_graph::product_operation::WorthQueryRuntimeCustody<
            'runtime,
            S,
        >,
    cell: WeakHistorySelectionCell,
    attempt: SharedNavigationAttempt,
}

impl<S: worth_query_installation::facade::ApplicationSchema>
    WorthQueryDocumentHistoryNavigation<'_, S>
{
    pub fn posture(&self) -> WorthQueryHistoryNavigationPosture {
        match self.attempt.progress.try_lock() {
            Ok(progress) => progress.posture(),
            Err(std::sync::TryLockError::WouldBlock) => {
                WorthQueryHistoryNavigationPosture::Advancing
            }
            Err(std::sync::TryLockError::Poisoned(poison)) => poison.into_inner().posture(),
        }
    }
}

impl<S> std::fmt::Debug for WorthQueryDocumentHistoryNavigation<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryNavigation")
            .finish_non_exhaustive()
    }
}

pub(in crate::domain_computation) use output_authority::HistoryProducerInvocation;
