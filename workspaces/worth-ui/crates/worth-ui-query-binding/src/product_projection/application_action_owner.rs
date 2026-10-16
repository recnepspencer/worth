use std::num::NonZeroUsize;
use worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope;
use worth_query_host::facade::primary_graph::WorthQueryApplicationLiveCloseOutcome;

use crate::{
    WorthUiScalarProjectionActionEvidence, WorthUiStatusActionRequest, WorthUiStatusPublication,
};

use super::WorthUiStatusSourceOwner;
use super::application_authentication::request_scope;
use super::status_mutation::{WorthUiStatusMutation, WorthUiStatusMutationResult};

#[derive(Debug)]
pub struct WorthUiStatusActionExecution {
    publication: WorthUiStatusPublication,
    evidence: WorthUiScalarProjectionActionEvidence,
}

#[derive(Debug)]
pub enum WorthUiStatusActionOutcome {
    Committed(WorthUiStatusActionExecution),
    DeniedRevisionMismatch {
        active_revision: u64,
        submitted_revision: u64,
        live_close: Option<WorthQueryApplicationLiveCloseOutcome>,
    },
}

impl WorthUiStatusActionExecution {
    pub(super) fn from_publication(
        publication: WorthUiStatusPublication,
        source_revision: u64,
        status: String,
    ) -> Self {
        Self {
            evidence: WorthUiScalarProjectionActionEvidence::from_application_publication(
                source_revision,
                status,
                &publication,
            ),
            publication,
        }
    }
    pub(super) fn from_parts(
        publication: WorthUiStatusPublication,
        evidence: WorthUiScalarProjectionActionEvidence,
    ) -> Self {
        Self {
            publication,
            evidence,
        }
    }
    pub fn into_parts(
        self,
    ) -> (
        WorthUiStatusPublication,
        WorthUiScalarProjectionActionEvidence,
    ) {
        (self.publication, self.evidence)
    }
}

impl WorthUiStatusSourceOwner {
    /// Captures original action identity, source and live subscription without executing.
    pub fn prepare_action(
        &self,
        action: WorthUiStatusActionRequest,
        scope: &WorthQueryRequestScope,
    ) -> Result<WorthUiStatusMutation, WorthUiStatusMutation> {
        WorthUiStatusMutation::action(&self.application, action).prepare(scope)
    }

    pub fn execute_action(
        &self,
        action: WorthUiStatusActionRequest,
        maximum_history_work: NonZeroUsize,
    ) -> Result<WorthUiStatusActionOutcome, WorthUiStatusMutation> {
        let scope = request_scope();
        let operation = self.prepare_action(action, &scope)?;
        match operation.advance(&scope, maximum_history_work)? {
            WorthUiStatusMutationResult::Action(outcome) => Ok(outcome),
            WorthUiStatusMutationResult::Source(_) => {
                unreachable!("action construction retains action custody")
            }
        }
    }
}
