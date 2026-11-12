//! History output reads retain the original authority and request refusal.
mod ownership;
mod recipe_capture;
use super::*;
use crate::http::server::{advancement_denial, relational_execution_denial, world_denial};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationRequestQueryDenial, WorthQueryDocumentHistoryDenial,
        WorthQueryHistoryNavigationDenial, WorthQueryHistoryOutputControlDenial,
        WorthQueryHistoryOutputReadDenial,
    },
    primary_graph::{RelationalBranchBasisDenial, WorthQueryRequiredSetCurrentnessDenial},
};

pub(super) fn denial(cause: WorthQueryHistoryOutputReadDenial) -> BankHttpDenial {
    use WorthQueryApplicationRequestQueryDenial as Query;
    use WorthQueryHistoryOutputReadDenial as D;
    match cause {
        D::Authority(cause) => authority(cause),
        D::RecipeCapture(cause) => recipe_capture::denial(cause),
        D::Admission(cause) => query(Query::Admission(cause)),
        D::Read(cause) => managed_index::managed_index(&cause),
        D::Control(cause) => control(cause),
        D::Demand(cause) => query(Query::OutputSettlement(cause)),
        D::Principal(cause) => query(Query::PrincipalResolution(cause)),
        D::Scope(cause) => query(Query::ScopeResolution(cause)),
        D::Preparation(cause) => relational_execution_denial::denial(cause),
        D::Resource(cause) => managed_index::native_admission(&cause),
        D::Dependencies(cause) => match cause {
            WorthQueryRequiredSetCurrentnessDenial::Output(cause) => {
                query(Query::OutputSettlement(cause))
            }
            WorthQueryRequiredSetCurrentnessDenial::Source(cause) => managed_index::source(&cause),
        },
    }
}

fn query(cause: WorthQueryApplicationRequestQueryDenial) -> BankHttpDenial {
    super::query_denial(BankApplicationQueryDenial::from_request_query(cause))
}

fn control(cause: WorthQueryHistoryOutputControlDenial) -> BankHttpDenial {
    use WorthQueryHistoryOutputControlDenial as D;
    match cause {
        D::Cancelled => cancelled(),
        D::DeadlineExceeded => deadline(),
        D::StaleInstalledQuery | D::StalePrincipal => stale(),
        D::ForeignPlan => permission_denied(),
        D::ExpiredBasis | D::BasisUnavailable => unavailable(),
    }
}

fn authority(cause: WorthQueryHistoryNavigationDenial) -> BankHttpDenial {
    use WorthQueryHistoryNavigationDenial as D;
    match cause {
        D::Document(cause) => document(cause),
        D::Stopped(cause) => advancement_denial::advancement(cause),
        D::Authorization(cause) => authorization_denial(cause.kind().into()),
        D::ForeignDocument | D::DiscoveryRecipeForeignDocument { .. } => permission_denied(),
        D::DiscoveryRecipeUnsupportedScope { .. } | D::DiscoveryRecipeInvalidScope { .. } => {
            malformed()
        }
        D::DiscoveryRecipeSealed { .. }
        | D::DiscoveryRecipeMissing { .. }
        | D::DiscoveryRecipeCorrupt { .. } => operator(),
        D::OutputLineage(cause) => relational_execution_denial::companion(cause),
        D::RestoredOutputCurrentness(cause) => query(
            WorthQueryApplicationRequestQueryDenial::OutputSettlement(cause),
        ),
        D::RestoredOutputRead(cause) => restored_read(cause),
        D::IntentDrift => BankHttpDenial::new(Kind::Stale, Next::CorrectRequest),
        D::HeadChanged => stale(),
        // The authority read producers do not perform navigation, mutation or fork work.
        // A failure from those families at this read boundary is a host contract fault.
        D::RequiresSemanticCorrection(_)
        | D::DocumentClosed
        | D::NoParent
        | D::NotChild
        | D::Busy
        | D::UnfinishedAttemptLimitExceeded
        | D::Discarded
        | D::DisposalInProgress
        | D::AlreadySelected
        | D::Value(_)
        | D::CapabilityInstallation(_)
        | D::OperationInstallation(_)
        | D::ProductAdmission(_)
        | D::CandidateCreation(_)
        | D::ForkRecovery
        | D::Transaction(_)
        | D::Preparation(_)
        | D::OwnedGraph(_)
        | D::GraphPreparation(_)
        | D::OutputPublicationCorrespondence(_)
        | D::SelectorSnapshot(_)
        | D::Publication(_)
        | D::Recovery(_)
        | D::RecoveryAffinity
        | D::RecoveryPreparation(_)
        | D::Adoption(_)
        | D::Cleanup => operator(),
    }
}

fn document(cause: WorthQueryDocumentHistoryDenial) -> BankHttpDenial {
    use WorthQueryDocumentHistoryDenial as D;
    match cause {
        D::Stopped(cause) => advancement_denial::advancement(cause),
        D::NativeBasis(cause) => native_basis(cause),
        D::UnownedOperationScope { .. } => permission_denied(),
        D::Ownership(cause) => ownership::denial(cause),
        D::Recording(cause) => relational_execution_denial::denial(cause),
        D::OperationAdmission(cause) => authorization_denial(cause.kind().into()),
        D::Read(cause) => managed_index::managed_index(&cause),
        D::HeadAdmission(cause) => world_denial::service(cause),
        D::HeadDiverged => stale(),
        D::ForeignPhase
        | D::InstalledBindingMismatch
        | D::ForeignModel
        | D::UnsupportedReadMode => operator(),
        // These are valid document-operation descriptions, but no read handoff emits them.
        D::ForeignRuntime
        | D::ForeignHistoryNode
        | D::HistoryNodeUnavailable
        | D::HistoryNodeExcluded
        | D::RetentionMaintenancePending
        | D::RetentionReleaseCapacityExhausted
        | D::BookmarkCapacityExhausted
        | D::RetentionPolicyNotDeclared
        | D::RetentionOperationInstallation(_)
        | D::Allocation(_)
        | D::UnsupportedContextRefresh
        | D::ContextRefresh(_)
        | D::OperationNotDeclared
        | D::RecordingIdentityExhausted
        | D::AttemptInFlight
        | D::IntentDrift
        | D::ForeignRecording
        | D::UnsupportedOperation
        | D::EmptyGroup
        | D::GroupMemberLimitExceeded
        | D::GroupResultLimitExceeded
        | D::GroupMemberMismatch
        | D::GroupNotComplete
        | D::CandidateSelection(_)
        | D::ReceiptMismatch
        | D::Adoption(_)
        | D::Paging(_)
        | D::Metadata(_)
        | D::MetadataCapabilityInstallation(_) => operator(),
    }
}

pub(super) fn native_basis(cause: RelationalBranchBasisDenial) -> BankHttpDenial {
    use RelationalBranchBasisDenial as D;
    match cause {
        D::Cancelled => cancelled(),
        D::TimedOut => deadline(),
        D::RetentionCapacityExhausted | D::OwnerUnavailable | D::MaterializationUnavailable => {
            unavailable()
        }
        D::RetentionIdentityExhausted
        | D::SnapshotIdentityExhausted
        | D::UnavailableRetainedTarget => {
            BankHttpDenial::new(Kind::Unavailable, Next::ContactOperator)
        }
        D::UnknownBranch(_)
        | D::ArchivedBranch(_)
        | D::DeletingBranch(_)
        | D::StaleReferenceGeneration
        | D::WrongBranchLocalTruthVersion => stale(),
        D::MalformedDescriptor
        | D::UnsupportedDescriptorVersion { .. }
        | D::ForeignRuntime { .. }
        | D::EmptyCommittedTargetMismatch
        | D::WrongImmutableTarget
        | D::MixedAxis(_)
        | D::OwnerFailure => operator(),
    }
}

fn restored_read(
    cause: worth_query_host::facade::primary_graph::RelationalHistoricalSelectionReadDenial,
) -> BankHttpDenial {
    use worth_query_host::facade::primary_graph::RelationalHistoricalSelectionReadDenial as D;
    match cause {
        D::Execution(cause) => relational_execution_denial::denial(cause),
        D::Request(cause) => advancement_denial::request(cause),
        D::SelectedBasisMismatch { .. } => stale(),
        D::ForeignOwner | D::NotGraphSelection { .. } => operator(),
    }
}

fn operator() -> BankHttpDenial {
    BankHttpDenial::new(Kind::InternalDenied, Next::ContactOperator)
}
