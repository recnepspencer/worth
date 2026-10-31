//! Owned process-local navigation attempts; canonical truth stays with Native and World.
mod admission;
mod control;
mod custody;
mod disposal;
mod eligibility;
mod progression;
mod publication;
mod registration;
mod selection;
mod transition;

#[doc(hidden)]
pub use control::WorthQueryHistoryNavigationPauseForTest;
pub use custody::WorthQueryHistoryNavigationPosture;
use custody::{NavigationAttempt, NavigationPhase, NavigationProgress};
pub(super) use disposal::NavigationDisposal;
pub(super) use registration::NavigationJournal;
pub use selection::{WorthQueryHistoryNavigationCompletion, WorthQueryHistorySelection};

use super::{selection::HistorySelectionCell, WorthQueryDocumentHistoryDenial};
use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial, WorthQueryExpectedProductBranchAdoptionDenial,
};
use std::sync::{Arc, Weak};

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
    IntentDrift,
    Busy,
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
