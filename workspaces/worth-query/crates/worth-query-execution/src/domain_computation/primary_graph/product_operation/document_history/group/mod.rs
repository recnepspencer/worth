//! Ordered group correspondence owns one reserved edge, never an SDK operation.
use super::*;
use crate::{
    basis::WorthQueryProductBranchReadIdentity,
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationIdempotencyBinding,
        WorthQueryApplicationReadObservation, WorthQueryExpectedProductBranch,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_history::{ApplicationHistoryBinding, ApplicationHistoryModel},
    application_operation::{
        ApplicationMutationBinding, ApplicationMutationScopeBinding, NoApplicationMutationSource,
    },
    application_schema::ApplicationSchema,
};
use worth_query_installation::facade::WorthQueryInstalledDocumentHistory;

mod admission;
mod document_identity;
pub use document_identity::WorthQueryDocumentHistoryIdentity;
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
    completed: Vec<Arc<WorthQueryApplicationReadObservation>>,
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
    _allocation: allocation::HistoryInsertionAllocation,
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
