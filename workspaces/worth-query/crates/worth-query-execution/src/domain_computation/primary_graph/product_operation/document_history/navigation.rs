//! Owned process-local navigation attempts; canonical truth stays with Native and World.
mod admission;
mod control;
mod custody;
mod disposal;
mod eligibility;
pub(crate) mod output_authority;
mod output_completion;
mod output_correspondence;
mod output_lookup;
mod output_profile;
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
use custody::{NavigationAttempt, NavigationPhase, NavigationProgress};
pub(super) use disposal::NavigationDisposal;
pub use output_authority::WorthQueryHistoryOutputAuthority;
pub(in crate::domain_computation::primary_graph) use output_authority::{
    HistoryBorrowedReadStop, HistoryProducerCutover, HistoryProducerInvocation,
};
pub use output_completion::WorthQueryCompletedHistoryOutputs;
pub use output_profile::{
    WorthQueryHistoryOutputNavigationAdvance, WorthQueryHistoryOutputProfile,
};
pub(in crate::domain_computation::primary_graph) use output_source::HistoryDependentSource;
pub use output_source::WorthQueryHistoryRootOutputAdmissionDenial;
pub use output_source::{WorthQueryAdmittedHistoryProgramOutput, WorthQueryHistoryOutputSource};
pub use output_source::{WorthQueryHistoryDependentOutputRead, WorthQueryHistoryOutputDiscovery};
pub use output_source::{
    WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryProgramOutputAdvance,
    WorthQuerySettledHistoryProgramOutput,
};
pub(super) use registration::NavigationJournal;
pub use selection::{WorthQueryHistoryNavigationCompletion, WorthQueryHistorySelection};
pub use target::WorthQueryHistoryNavigationIntent;

use super::{selection::HistorySelectionCell, WorthQueryDocumentHistoryDenial};
use std::sync::{Arc, Weak};
use {crate::domain_computation::request_execution::WorthQueryAdvancementDenial, crate::domain_computation::primary_graph::WorthQueryExpectedProductBranchAdoptionDenial};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryNavigationEligibilityDenial {
    NotDeclared,
    ModelDomain,
    LiveControlDomain,
    ProgramUnavailable,
    ProgramChanged,
    RequiredOutputs,
    SignalChanged,
    ExternalEffects,
    Workflow,
}

#[derive(Debug)]
pub enum WorthQueryHistoryNavigationDenial {
    Document(WorthQueryDocumentHistoryDenial),
    RequiresSemanticCorrection(WorthQueryHistoryNavigationEligibilityDenial),
    DocumentClosed,
    ForeignDocument,
    HeadChanged,
    NoParent,
    NotChild,
    IntentDrift,
    Busy,
    UnfinishedAttemptLimitExceeded,
    Discarded,
    DisposalInProgress,
    AlreadySelected,
    Value(worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial),
    CapabilityInstallation(
        worth_query_installation::facade::WorthQueryApplicationCapabilityInstallationDenial,
    ),
    OperationInstallation(
        worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenial,
    ),
    Authorization(crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial),
    ProductAdmission(crate::basis::WorthQueryProductBranchAdmissionDenial),
    CandidateCreation(crate::basis::WorthQueryProductBranchCreateError),
    ForkRecovery,
    Transaction(worth_relational::facade::mvcc::RelationalBranchTransactionAdmissionDenial),
    Preparation(worth_relational::facade::mvcc::RelationalRetainedEntitySelectionPreparationError),
    SelectorSnapshot(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
    SelectorMaintenance(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial),
    Publication(worth_runtime_world::facade::NoEffectCompositePublication),
    Recovery(worth_runtime_world::facade::RuntimeWorldRecoveryDenial),
    RecoveryAffinity,
    RecoveryPreparation(worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial),
    Adoption(WorthQueryExpectedProductBranchAdoptionDenial),
    Stopped(WorthQueryAdvancementDenial),
    Cleanup,
}

/// A live attempt handle, issued only after fresh READ, metadata and NAV.
/// The document's registry retains actual candidate custody if this handle is dropped.
#[must_use = "advance or explicitly discard the registered navigation attempt"]
pub struct WorthQueryDocumentHistoryNavigation<'runtime, S> {
    cell: Weak<HistorySelectionCell<'runtime, S>>,
    attempt: Arc<NavigationAttempt<'runtime, S>>,
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
