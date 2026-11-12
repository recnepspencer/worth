//! Query's concrete original preparation, retained in its existing unpublished entry.
use super::*;
use worth_runtime_world::facade::*;

/// Owner calls consume their raw input. A call-in-progress posture never
/// reconstructs that input after unwind; only original recovery is permitted.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph::provider) enum QueryDurableOwnerCall {
    Open,
    NativeBinding,
    NativeCompletion,
    Publication,
    PublicationAdvance,
}

pub(super) enum QueryNativeCustody {
    Proposal(worth_relational::facade::mvcc::ValidatedRelationalProposal),
    OwnerCallInProgress,
    Candidate(worth_relational::facade::mvcc::PreparedRelationalCommitCandidate),
}

/// Contains raw World custody only. Public Query recovery wrappers and their
/// same-store disposition/reservation handles never enter this payload.
pub(in crate::domain_computation::primary_graph::provider) enum QueryDurableWorldCustody {
    PendingOpen(WorldPendingOpenAttempt),
    Opened(WorldOpenedPublicationAttempt),
    NativePreparation(super::QueryBoundWorldPreparation),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    NativeRefused(WorldNativePreparationRefusal),
    Prepared(super::QueryPreparedWorldPublication),
    PendingPublication(WorldPendingDurablePublication),
    Unpublished(ProductUnpublishedOwnerEffects),
    PublicationRecoveryRequired {
        #[allow(
            dead_code,
            reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
        )]
        cause: WorldDurablePublicationDenial,
        #[allow(
            dead_code,
            reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
        )]
        custody: ProductUnpublishedOwnerEffects,
    },
    OwnerCallInProgress(QueryDurableOwnerCall),
    OpenRefused(WorldOpenAttemptRefusal),
    Performed(PerformedCompositePublication),
    RecoveryOnly,
}

pub(in crate::domain_computation) struct QueryPreparingApplicationCommit {
    pub(super) predicate: Option<crate::domain_computation::primary_graph::application_query::derived_view::PredicatePreparationOwner>,
    pub(super) attempt: WorthQueryPrimaryGraphApplicationAttempt,
    pub(super) members: Vec<crate::domain_computation::primary_graph::provider::CoupledApplicationMember>,
    pub(super) group_emissions: Option<crate::domain_computation::primary_graph::application_attempt::WorthQueryAdmittedApplicationEmissionBatch>,
    pub(super) group_evidence: Option<coupled::CoupledEvidenceSlots>,
    pub(super) prepared_group_publications: Option<worth_relational::facade::mvcc::RelationalPreparedOutputPublications>,
    pub(super) native: QueryNativeCustody,
    pub(super) work: WorthQueryPrimaryMutationWorkCounters,
    pub(super) retained_preimage: Option<crate::domain_computation::application_aftermath::WorthQueryRetainedPreImage>,
    pub(super) preimage_retention_work: WorthQueryPreImageRetentionWork,
    pub(super) completion: Option<super::super::super::super::commit_completion::WorthQueryApplicationAttemptCompletion>,
    pub(super) before: precommit_snapshot::WorthQueryPrecommitSnapshot,
    pub(super) required_prerequisites: Option<crate::domain_computation::primary_graph::PreparedPrerequisiteClaims>,
    pub(super) availability_fence: Option<crate::domain_computation::primary_graph::output_lineage::invalidation::SourcePublicationReadFence>,
    pub(super) prepared_output_witness: Option<crate::domain_computation::primary_graph::output_lineage::PreparedNativeOutputWitness>,
    pub(super) source_fact_rebase: Option<super::super::PreparedSourceFactRebase>,
    pub(super) prepared_touched_records: Option<PreparedTouchedRecords>,
    pub(super) index_maintenance_work: Option<worth_relational::facade::indexes::DerivedIndexMaintenanceWork>,
    pub(super) managed_views: Option<managed_views::PreparedViewPublication>,
}

/// The original Query state and World stage share one already-reserved slot.
/// An external checkout guard owns the reservation, store and entry Arcs.
pub(in crate::domain_computation::primary_graph::provider) struct RetainedQueryDurablePreparation {
    pub(super) query: QueryPreparingApplicationCommit,
    pub(super) stage: crate::domain_computation::WorthQueryProductPreparationStage,
    pub(super) world: QueryDurableWorldCustody,
    pub(super) recovery_handle: ProductUnpublishedRecoveryHandle,
    pub(super) control: crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationRequestControl,
    pub(super) terminal: crate::domain_computation::execution_runtime::product_world::WorthQueryReservedProductPublicationReceipt,
    pub(super) prepared_lineage_slot: Option<crate::domain_computation::primary_graph::output_lineage::PreparedOutputLineageSlot>,
    pub(super) query_denial: Option<QueryDurablePreparationDenial>,
    pub(super) authored_completion: Option<worth_runtime_bridge::facade::BridgeDurableDefinitionCompletion>,
}

/// Narrow causes cannot embed a public same-store Query recovery wrapper.
pub(in crate::domain_computation::primary_graph::provider) enum QueryDurablePreparationDenial {
    ConditionalActivation(crate::domain_computation::execution_runtime::product_world::activation::WorthQueryProductActivationDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Request(worth_execution::WorkCeilingDenial),
    OriginalMeaning(crate::facade::program_support::QueryProgramSupportDenial),
    CorrectionMaterial(crate::facade::correction_material::QueryCorrectionMaterialDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Candidate(QueryCandidatePreparationDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    Prerequisite(crate::domain_computation::primary_graph::WorthQueryOutputDemandDenial),
    #[allow(
        dead_code,
        reason = "The typed diagnostic or custody payload stays owned through return and Drop; this owner does not inspect it"
    )]
    QueryEvidence(WorthQueryProviderSessionFailure),
}

#[derive(Debug)]
pub(super) enum QueryCandidatePreparationDenial {
    ConsumedOutput(consumed_output::ConsumedOutputPreparationDenial),
    CoupledSupport(coupled::CoupledSupportPreparationDenial),
    Dispatch(
        crate::domain_computation::primary_graph::live_delivery::WorthQueryLiveAdmissionDenial,
    ),
    Correspondence(correspondence::OutputCorrespondencePreparationDenial),
    ManagedSource(managed_source::ManagedSourcePreparationDenial),
    Native(worth_relational::facade::mvcc::TransactionCommitError),
    OutputWitness(worth_relational::facade::mvcc::CompanionPreflightStop),
    SourceFacts(super::super::PreparedRebaseDenial),
    Index(worth_relational::facade::indexes::DerivedIndexMaintenanceDenial),
    TouchedRecords(touched_records::TouchedRecordsPreparationDenial),
}

impl QueryPreparingApplicationCommit {
    pub(in crate::domain_computation) fn is_unprepared(&self) -> bool {
        matches!(self.native, QueryNativeCustody::Proposal(_))
    }
}
impl std::fmt::Debug for RetainedQueryDurablePreparation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let original_meaning = match &self.query_denial {
            Some(QueryDurablePreparationDenial::OriginalMeaning(cause)) => Some(cause),
            _ => None,
        };
        let correction_material = match &self.query_denial {
            Some(QueryDurablePreparationDenial::CorrectionMaterial(cause)) => Some(cause),
            _ => None,
        };
        formatter
            .debug_struct("RetainedQueryDurablePreparation")
            .field("correction_material", &correction_material)
            .field("original_meaning", &original_meaning)
            .field("stage", &self.stage())
            .field("query_denial", &self.query_denial)
            .finish_non_exhaustive()
    }
}

impl RetainedQueryDurablePreparation {
    pub(in crate::domain_computation::primary_graph::provider) fn stage(
        &self,
    ) -> crate::domain_computation::WorthQueryProductPreparationStage {
        use crate::domain_computation::WorthQueryProductPreparationStage as Stage;
        match &self.world {
            QueryDurableWorldCustody::OwnerCallInProgress(call) => match call {
                QueryDurableOwnerCall::Open => Stage::Opening,
                QueryDurableOwnerCall::NativeBinding => Stage::BindingNativeProposal,
                QueryDurableOwnerCall::NativeCompletion => Stage::CompletingNative,
                QueryDurableOwnerCall::Publication => Stage::Publishing,
                QueryDurableOwnerCall::PublicationAdvance => Stage::PublicationPending,
            },
            QueryDurableWorldCustody::Opened(_) => self.stage,
            QueryDurableWorldCustody::NativePreparation(_) => match &self.query.native {
                QueryNativeCustody::Candidate(_) => Stage::PreparingQueryCandidate,
                _ => Stage::PreparingNative,
            },
            QueryDurableWorldCustody::Prepared(_) => Stage::PreparingQueryCandidate,
            QueryDurableWorldCustody::RecoveryOnly => Stage::RecoveryOnly,
            _ => self.stage,
        }
    }
    pub(in crate::domain_computation::primary_graph::provider) fn abandon_world_phase(
        &mut self,
    ) -> QueryDurableWorldCustody {
        std::mem::replace(&mut self.world, QueryDurableWorldCustody::RecoveryOnly)
    }
}

impl std::fmt::Debug for QueryDurablePreparationDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OriginalMeaning(cause) => f.debug_tuple("OriginalMeaning").field(cause).finish(),
            Self::CorrectionMaterial(cause) => {
                f.debug_tuple("CorrectionMaterial").field(cause).finish()
            }
            Self::Request(cause) => f.debug_tuple("Request").field(cause).finish(),
            Self::Candidate(cause) => f.debug_tuple("Candidate").field(cause).finish(),
            Self::Prerequisite(cause) => f.debug_tuple("Prerequisite").field(cause).finish(),
            Self::QueryEvidence(cause) => f.debug_tuple("QueryEvidence").field(cause).finish(),
            Self::ConditionalActivation(cause) => {
                f.debug_tuple("ConditionalActivation").field(cause).finish()
            }
        }
    }
}
