//! Ordered group correspondence owns one reserved edge, never an SDK operation.
use super::*;
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationScopeBinding, NoApplicationMutationSource,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;
use {
    crate::basis::WorthQueryProductBranchReadIdentity,
    crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyBinding,
    crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation,
    crate::domain_computation::primary_graph::WorthQueryExpectedProductBranch,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
    crate::publication_boundary::WorthQueryProgramPublicationAccess,
};

mod admission;
mod member;
mod sealing;

/// Exact ordinary admission narrowed to one document before any group effect.
pub struct WorthQueryHistoryGroupMemberAdmission {
    parent: WorthQueryHistoryNodeId,
    expected: WorthQueryProductBranchReadIdentity,
    model: HistoryModelIdentity,
    meaning: WorthQueryApplicationIdempotencyBinding,
    operation_index: usize,
}

#[must_use = "retain the group record until actual adoption or explicit abort"]
pub struct WorthQueryDocumentHistoryGroupReservation {
    reservation: WorthQueryDocumentHistoryReservation,
    members: Box<[WorthQueryApplicationIdempotencyBinding]>,
    completed: Vec<CapturedMember>,
    candidate: Option<(
        crate::basis::WorthQueryProductBranch,
        Option<WorthQueryProductBranchReadIdentity>,
    )>,
    _allocation: allocation::HistoryInsertionAllocation,
}

pub enum WorthQueryDocumentHistoryGroupPreparation {
    Reserved(WorthQueryDocumentHistoryGroupReservation),
    AlreadyRecorded(WorthQueryHistoryStep),
}

#[must_use = "close the actual candidate before cancelling this reserved group edge"]
pub struct WorthQueryDocumentHistoryGroupAborting {
    reservation: WorthQueryDocumentHistoryReservation,
    runtime_authority: u64,
    candidate: Option<crate::basis::WorthQueryProductBranch>,
    _allocation: allocation::HistoryInsertionAllocation,
}

struct CapturedMember {
    endpoint: Arc<WorthQueryApplicationReadObservation>,
    root_posture:
        Option<crate::domain_computation::primary_graph::WorthQueryOutputSettlementPosture>,
}

impl WorthQueryDocumentHistoryGroupAborting {
    /// Disposal correspondence for this original private candidate only. This
    /// grants no execution, output fulfillment or document-selection entrance.
    pub fn validate_candidate_cleanup<S: ApplicationSchema>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        runtime: &dyn crate::domain_computation::primary_graph::product_operation::host::ProductOperationHost<S>,
        branch: crate::basis::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        phase
            .checked_execution_request_for(runtime.product_runtime())
            .map_err(WorthQueryDocumentHistoryDenial::Stopped)?;
        if self.runtime_authority != runtime.authority_identity().as_u64()
            || self.candidate != Some(branch)
        {
            return Err(WorthQueryDocumentHistoryDenial::ForeignRecording);
        }
        Ok(())
    }
}

impl WorthQueryDocumentHistoryGroupReservation {
    pub fn completed_members(&self) -> usize {
        self.completed.len()
    }
}

pub struct WorthQueryDocumentHistoryGroupAdoptionFailure<'runtime, S> {
    pub reservation: WorthQueryDocumentHistoryGroupReservation,
    pub candidate: WorthQueryExpectedProductBranch<'runtime, S>,
    pub denial: WorthQueryDocumentHistoryDenial,
}

/// Creation keeps the real candidate even if its immediate source admission stops.
pub struct WorthQueryHistoryGroupCandidateCreation<'runtime, S> {
    pub candidate: WorthQueryExpectedProductBranch<'runtime, S>,
    pub denial: Option<WorthQueryDocumentHistoryDenial>,
}

impl<S> std::fmt::Debug for WorthQueryDocumentHistoryGroupAdoptionFailure<'_, S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorthQueryDocumentHistoryGroupAdoptionFailure")
            .field("denial", &self.denial)
            .finish_non_exhaustive()
    }
}
