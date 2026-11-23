use crate::domain_computation::primary_graph::{
    WorthQueryAdvancementDenial, WorthQueryManagedIndexDenial,
};
use worth_execution::WorkCeilingDenial;
use worth_relational::facade::branch::RelationalBranchBasisDenial;
use worth_runtime_world::facade::{RuntimeWorldBranchAdmissionDenial, RuntimeWorldServiceDenial};

pub use worth_relational::facade::branch::RelationalRecordAdvanceDenial as WorthQueryHistoryContextRefreshDenial;

#[derive(Debug)]
pub enum WorthQueryExpectedProductBranchAdoptionDenial {
    ForeignPhase,
    Stopped(WorthQueryAdvancementDenial),
    /// The application's actual head slot no longer names the selected source.
    HeadReplaced,
    CandidateAdmission(crate::basis::WorthQueryProductBranchAdmissionDenial),
    SourceBasis(RelationalBranchBasisDenial),
    CandidateBasis(RelationalBranchBasisDenial),
    HeadAdmission(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
    HeadMoved,
}

#[derive(Debug)]
pub enum WorthQueryHistoryPageDenial {
    ForeignCursor,
    StaleCursor,
    CursorScopeMismatch,
    StorageLimitExceeded,
    SizeOverflow,
    Allocation(std::collections::TryReserveError),
    Request(WorkCeilingDenial),
}

#[derive(Debug)]
pub enum WorthQueryDocumentHistoryDenial {
    ForeignPhase,
    ForeignRuntime,
    InstalledBindingMismatch,
    UnsupportedReadMode,
    ForeignModel,
    UnownedOperationScope {
        subject: worth_relational::facade::identity::EntityId,
        model: worth_relational::facade::identity::EntityId,
    },
    Ownership(worth_relational::facade::branch::RelationalModelOwnershipDenial),
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
    ContextRefresh(WorthQueryHistoryContextRefreshDenial),
    OperationNotDeclared,
    OperationAdmission(
        crate::domain_computation::authorization::WorthQueryOperationAuthorizationDenial,
    ),
    HeadDiverged,
    Recording(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    RecordingIdentityExhausted,
    AttemptInFlight,
    IntentDrift,
    /// The step metadata exceeds the installed `maximum_step_metadata_bytes`.
    StepMetadataTooLarge {
        length: usize,
        maximum: usize,
    },
    ForeignRecording,
    UnsupportedOperation,
    /// A producer the stage's outputs can reach is workflow control or needs
    /// workflow authority, which no undo could take back.
    WorkflowProducer {
        producer: String,
    },
    /// A producer the stage's outputs can reach declares an external effect,
    /// which no undo could take back.
    ExternalEffectProducer {
        producer: String,
    },
    EmptyGroup,
    GroupMemberLimitExceeded,
    GroupResultLimitExceeded,
    GroupMemberMismatch,
    GroupNotComplete,
    CandidateSelection(crate::basis::WorthQueryProductBranchAdmissionDenial),
    ReceiptMismatch,
    Adoption(WorthQueryExpectedProductBranchAdoptionDenial),
    Stopped(WorthQueryAdvancementDenial),
    NativeBasis(RelationalBranchBasisDenial),
    HeadAdmission(RuntimeWorldServiceDenial<RuntimeWorldBranchAdmissionDenial>),
    Read(WorthQueryManagedIndexDenial),
    Paging(WorthQueryHistoryPageDenial),
    Metadata(crate::domain_computation::primary_graph::WorthQueryApplicationHistoryDenial),
    MetadataCapabilityInstallation(
        worth_query_installation::facade::WorthQueryApplicationCapabilityInstallationDenial,
    ),
}

impl WorthQueryDocumentHistoryDenial {
    pub fn ownership_interruption(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption>
    {
        use crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption as I;
        use worth_execution::MapKernelStop as S;
        match self {
            Self::Ownership(cause) => match cause.kernel_stop() {
                Some(S::Cancelled) => Some(I::Cancelled),
                Some(S::DeadlineElapsed) => Some(I::DeadlineExceeded),
                _ => None,
            },
            _ => None,
        }
    }

    /// Project the Native basis owner's interruption without exposing its type
    /// as a dependency of the ordinary publication frontend.
    pub fn native_basis_interruption(
        &self,
    ) -> Option<crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption>
    {
        use crate::domain_computation::primary_graph::WorthQueryManagedComputationInterruption as Interruption;
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
    DiscoveryRecipeUnsupportedScope {
        step: super::WorthQueryHistoryNodeId,
        binding: &'static str,
    },
    DiscoveryRecipeInvalidScope {
        step: super::WorthQueryHistoryNodeId,
        binding: &'static str,
    },
    DiscoveryRecipeSealed {
        step: super::WorthQueryHistoryNodeId,
    },
    DiscoveryRecipeMissing {
        step: super::WorthQueryHistoryNodeId,
    },
    DiscoveryRecipeCorrupt {
        step: super::WorthQueryHistoryNodeId,
    },
    DiscoveryRecipeForeignDocument {
        step: super::WorthQueryHistoryNodeId,
    },
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
    OwnedGraph(worth_relational::facade::branch::RelationalOwnedModelSelectionDenial),
    GraphPreparation(worth_relational::facade::mvcc::RelationalOwnedGraphSelectionPreparationError),
    /// The selected step retains no exact lineage row for a producer route
    /// Native restores, or its lookup stopped.
    OutputLineage(worth_relational::facade::mvcc::CompanionPreflightStop),
    /// Descriptive Query roles disagree with this selected Native publication.
    OutputPublicationCorrespondence(
        crate::domain_computation::primary_graph::output_lineage::WorthQueryHistoryPublicationCorrespondenceMismatch,
    ),
    RestoredOutputRead(worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial),
    RestoredOutputCurrentness(
        crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial,
    ),
    SelectorSnapshot(worth_relational::facade::snapshots::RelationalSnapshotAdmissionDenial),
    Publication(worth_runtime_world::facade::NoEffectCompositePublication),
    Recovery(worth_runtime_world::facade::RuntimeWorldRecoveryDenial),
    RecoveryAffinity,
    RecoveryPreparation(worth_runtime_world::facade::RuntimeWorldSettledRelationalAdoptionDenial),
    Adoption(WorthQueryExpectedProductBranchAdoptionDenial),
    Stopped(WorthQueryAdvancementDenial),
    Cleanup,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryHistoryOutputControlDenial {
    ForeignPlan,
    StaleInstalledQuery,
    StalePrincipal,
    Cancelled,
    DeadlineExceeded,
    ExpiredBasis,
    BasisUnavailable,
}

/// Keep the original read, admission and authority causes distinct.
#[derive(Debug)]
pub enum WorthQueryHistoryOutputReadDenial {
    Authority(WorthQueryHistoryNavigationDenial),
    RecipeCapture(crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial),
    Admission(crate::domain_computation::primary_graph::WorthQueryApplicationQueryAdmissionDenial),
    Read(WorthQueryManagedIndexDenial),
    Control(WorthQueryHistoryOutputControlDenial),
    Demand(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    Principal(crate::domain_computation::primary_graph::WorthQueryPrincipalResolutionDenial),
    Scope(crate::domain_computation::primary_graph::WorthQueryEntityResolutionDenial),
    Resource(worth_relational::facade::mvcc::CompanionPreflightStop),
    Preparation(worth_relational::facade::transactions::RelationalExecutionDenialCause),
    Dependencies(
        crate::discovery_boundary::discovery_currentness::WorthQueryRequiredSetCurrentnessDenial,
    ),
}

impl From<crate::domain_computation::primary_graph::application_query::SourceQueryObservationStop>
    for WorthQueryHistoryOutputReadDenial
{
    fn from(
        cause: crate::domain_computation::primary_graph::application_query::SourceQueryObservationStop,
    ) -> Self {
        use crate::domain_computation::primary_graph::application_query::SourceQueryObservationStop as Stop;
        match cause {
            Stop::Admission(cause) | Stop::Preparation { cause, .. } => Self::Admission(cause),
            Stop::Principal(cause) => Self::Principal(cause),
            Stop::Scope(cause) => Self::Scope(cause),
            cause => Self::Demand(
                crate::domain_computation::primary_graph::invariant_projection::WorthQueryOutputAvailabilityDenial::source_query_stop(cause).demand_stop(),
            ),
        }
    }
}
