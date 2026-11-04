use super::WorthQueryOwnedProductHead;
use {crate::domain_computation::primary_graph::request_execution::WorthQueryAdvancementDenial, crate::domain_computation::primary_graph::WorthQueryManagedIndexDenial};
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};

#[derive(Debug)]
pub enum WorthQueryDocumentHistoryDenial {
    ForeignPhase,
    ForeignRuntime,
    InstalledBindingMismatch,
    UnsupportedReadMode,
    ForeignModel,
    ForeignHistoryNode,
    HistoryNodeUnavailable,
    HistoryNodeExcluded,
    RetentionMaintenancePending,
    RetentionReleaseCapacityExhausted,
    BookmarkCapacityExhausted,
    RetentionPolicyNotDeclared,
    RetentionOperationInstallation(
        worth_query_installation::facade::WorthQueryApplicationOperationInstallationDenial,
    ),
    Allocation(std::collections::TryReserveError),
    UnsupportedContextRefresh,
    ContextRefresh(super::WorthQueryHistoryContextRefreshDenial),
    OperationNotDeclared,
    OperationAdmission(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ),
    HeadDiverged,
    Recording(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    RecordingIdentityExhausted,
    AttemptInFlight,
    IntentDrift,
    ForeignRecording,
    UnsupportedOperation,
    EmptyGroup,
    GroupMemberLimitExceeded,
    GroupResultLimitExceeded,
    GroupMemberMismatch,
    GroupNotComplete,
    CandidateSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    ReceiptMismatch,
    Adoption(
        crate::domain_computation::primary_graph::WorthQueryExpectedProductBranchAdoptionDenial,
    ),
    Stopped(WorthQueryAdvancementDenial),
    NativeBasis(RelationalBranchBasisDenial),
    HeadAdmission(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
    Read(WorthQueryManagedIndexDenial),
    Paging(super::WorthQueryHistoryPageDenial),
    Metadata(crate::domain_computation::primary_graph::WorthQueryApplicationHistoryDenial),
    MetadataCapabilityInstallation(
        worth_query_installation::facade::WorthQueryApplicationCapabilityInstallationDenial,
    ),
}

impl WorthQueryDocumentHistoryDenial {
    /// Project the Native basis owner's interruption without exposing its type
    /// as a dependency of the ordinary publication frontend.
    pub fn native_basis_interruption(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryManagedComputationInterruption>
    {
        use crate::domain_computation::primary_graph::request_execution::execution_refusal::WorthQueryManagedComputationInterruption as Interruption;
        match self {
            Self::NativeBasis(RelationalBranchBasisDenial::Cancelled) => {
                Some(Interruption::Cancelled)
            }
            Self::NativeBasis(RelationalBranchBasisDenial::TimedOut) => {
                Some(Interruption::DeadlineExceeded)
            }
            _ => None,
        }
    }
}

/// No document was attached. The original live head and actual read denial remain owned.
#[must_use = "retain or close the original head after refused attachment"]
pub struct WorthQueryDocumentHistoryAttachmentFailure<'runtime, S> {
    pub(in crate::domain_computation::primary_graph) head: WorthQueryOwnedProductHead<'runtime, S>,
    pub(in crate::domain_computation::primary_graph) denial: WorthQueryDocumentHistoryDenial,
}

impl<'runtime, S> WorthQueryDocumentHistoryAttachmentFailure<'runtime, S> {
    pub fn into_parts(
        self,
    ) -> (
        WorthQueryOwnedProductHead<'runtime, S>,
        WorthQueryDocumentHistoryDenial,
    ) {
        (self.head, self.denial)
    }
    pub fn denial(&self) -> &WorthQueryDocumentHistoryDenial {
        &self.denial
    }
}

impl<S> std::fmt::Debug for WorthQueryDocumentHistoryAttachmentFailure<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryAttachmentFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
