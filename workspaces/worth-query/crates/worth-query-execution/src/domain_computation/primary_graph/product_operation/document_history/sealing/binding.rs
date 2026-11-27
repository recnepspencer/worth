//! Binding a completed record to its endpoint. The checks run wherever a
//! caller relies on them; the journal's source index is written exactly
//! once, at adoption (public #453).
use super::super::{
    journal::RecordedNode, source_evidence::WorthQueryHistorySourceEvidence,
    WorthQueryDocumentHistory, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryReservation, WorthQueryHistoryStep,
};
use super::{PreparedBinding, RecordingCompletion};
use crate::{
    basis::{WorthQueryProductBranchReadIdentity, WorthQueryProductObservationLease},
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationReadObservation,
        WorthQueryExpectedProductBranch, WorthQueryOutputSettlementPosture,
    },
};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
use worth_query_execution_lineage::facade::ports::CommittedProductPublicationInternalPort as _;
use worth_query_execution_query::facade::ports::ApplicationRuntimeCoreInternalPort as _;
use worth_query_execution_query::facade::ports::CompletedRequiredOutputsInternalPort as _;
use worth_query_execution_runtime::facade::ports::{
    ProductBranchInternalPort as _, ProductBranchLeaseInternalPort as _,
    ProductObservationLeaseInternalPort as _,
};
use worth_relational::facade::history::CommitId;
use worth_runtime_world::facade::CompositeCommitIdentity;

/// A completed record's checked source and endpoint. Nothing is indexed yet.
pub(in super::super) struct BoundSource {
    pub(in super::super) source: Arc<WorthQueryApplicationReadObservation>,
    pub(in super::super) endpoint: WorthQueryProductBranchReadIdentity,
    pub(in super::super) exact: WorthQueryProductObservationLease,
    pub(in super::super) source_commit: Option<CompositeCommitIdentity>,
    /// The basis a later navigation to this step selects.
    pub(in super::super) basis_commit: Option<CommitId>,
    pub(in super::super) root_posture: Option<WorthQueryOutputSettlementPosture>,
}

impl BoundSource {
    pub(in super::super) fn into_prepared(
        self,
        step: WorthQueryHistoryStep,
        posted_fact: Option<worth_relational::facade::identity::EntityId>,
        discovery_recipes: Option<Arc<super::super::discovery_recipes::HistoryDiscoveryRecipes>>,
    ) -> PreparedBinding {
        let node = RecordedNode {
            step,
            source_commit: self.source_commit,
            posted_fact,
            root_posture: self.root_posture,
        };
        PreparedBinding {
            discovery_recipes,
            returned: node.step.clone(),
            node,
            source: self.source,
            endpoint: self.endpoint,
            exact: self.exact,
        }
    }
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Index the step's source commits and basis, with the fork `after`, and
    /// return its prepared step. The reservation is checked again under the
    /// same lock, so `physical_source` is the source the candidate forked
    /// from: every commit the step authored lies in `(after, basis]`.
    pub(in super::super) fn index_bound_step<'c>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: &WorthQueryDocumentHistoryReservation,
        commits: impl IntoIterator<Item = &'c CompositeCommitIdentity>,
        basis_commit: Option<CommitId>,
        endpoint: &WorthQueryProductBranchReadIdentity,
    ) -> Result<WorthQueryHistoryStep, Denial> {
        let execution = phase
            .checked_execution_request_for(self.application().inner.runtime.product_runtime_ref())
            .map_err(Denial::Stopped)?;
        let mut selection = self.selection.lock();
        selection.validate_reservation(reserved)?;
        let after = selection
            .physical_source
            .relational_basis()
            .observation()
            .commit_id();
        selection
            .journal
            .index_source(
                &execution,
                commits,
                after,
                basis_commit,
                reserved.record.node,
            )
            .map_err(Denial::Recording)?;
        Ok(selection
            .journal
            .prepared_step(&reserved.record, endpoint.clone()))
    }

    /// The ordinary record's binding at adoption: the checks, then the index.
    pub(in super::super) fn prepare_binding(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        completion: RecordingCompletion<'_>,
    ) -> Result<PreparedBinding, Denial> {
        let bound = self.bind_recording_source(phase, reserved, candidate, &completion)?;
        let step = self.index_bound_step(
            phase,
            reserved,
            bound.source_commit.as_ref(),
            bound.basis_commit,
            &bound.endpoint,
        )?;
        let discovery_recipes = match &completion {
            RecordingCompletion::Forest { completed, .. } => {
                Some(completed.discovery_recipes.clone())
            }
            _ => None,
        };
        // A writer's commit posts no reversal fact.
        Ok(bound.into_prepared(step, None, discovery_recipes))
    }

    /// The checks an ordinary record's binding relies on, and its source,
    /// endpoint and basis. It indexes nothing: an output origin calls it
    /// before adoption.
    pub(in super::super) fn bind_recording_source(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        completion: &RecordingCompletion<'_>,
    ) -> Result<BoundSource, Denial> {
        self.validate_reservation(reserved)?;
        let runtime = self.application();
        if !std::ptr::eq(runtime, candidate.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if candidate.expected_source() != reserved.expected {
            return Err(Denial::HeadDiverged);
        }
        if !completion
            .source()
            .matches(reserved.operation_scope()?, reserved.meaning)
        {
            return Err(Denial::ReceiptMismatch);
        }
        let (selected, retained, root_posture) = match completion {
            RecordingCompletion::Forest {
                source,
                required,
                completed,
            } => {
                completed.require_source(
                    reserved,
                    candidate,
                    reserved.operation_scope()?,
                    *source,
                    *required,
                )?;
                let retained = completed.endpoint();
                let selected = runtime
                    .select_application_read_observation(phase, retained)
                    .map_err(Denial::CandidateSelection)?;
                (
                    selected,
                    Some(std::sync::Arc::clone(retained)),
                    completed.root_posture(),
                )
            }

            RecordingCompletion::Source(receipt) => {
                let selected = runtime
                    .on_branch(candidate.branch())
                    .select(phase)
                    .map_err(Denial::CandidateSelection)?;
                if !receipt
                    .committed_product_publication()
                    .is_selected_at(selected.product().observation())
                {
                    return Err(Denial::ReceiptMismatch);
                }
                (selected, None, None)
            }
            RecordingCompletion::RecordedSource(receipt) => {
                let selected = runtime
                    .on_branch(candidate.branch())
                    .select(phase)
                    .map_err(Denial::CandidateSelection)?;
                if !WorthQueryHistorySourceEvidence::Recorded(receipt)
                    .is_selected_at(selected.product().observation())
                {
                    return Err(Denial::ReceiptMismatch);
                }
                (selected, None, None)
            }
            RecordingCompletion::RequiredOutputs { source, completed } => {
                let retained = completed.retained_output();
                if !completed.matches_source(source)
                    || retained.branch_incarnation() != candidate.branch().occurrence()
                {
                    return Err(Denial::ReceiptMismatch);
                }
                let selected = runtime
                    .select_application_read_observation(phase, retained)
                    .map_err(Denial::CandidateSelection)?;
                (
                    selected,
                    Some(std::sync::Arc::clone(retained)),
                    Some(completed.root_posture()),
                )
            }
        };
        let endpoint =
            WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
        let exact = selected.product().read_lease();
        // A writer's step is a correction source as an envelope's is: its
        // source commit (a recovered receipt behind a recorded forest names
        // none) and the basis a later navigation to it selects.
        let source_commit = match (completion, completion.source()) {
            (_, WorthQueryHistorySourceEvidence::Live(receipt)) => Some(
                receipt
                    .committed_product_publication()
                    .composite_commit()
                    .clone(),
            ),
            (
                RecordingCompletion::RecordedSource(_),
                WorthQueryHistorySourceEvidence::Recorded(_),
            ) => Some(endpoint.selected_commit().clone()),
            _ => None,
        };
        let basis_commit = selected
            .product()
            .relational_basis()
            .observation()
            .commit_id();
        Ok(BoundSource {
            source: retained.unwrap_or_else(|| selected.retain_application_read()),
            endpoint,
            exact,
            source_commit,
            basis_commit,
            root_posture,
        })
    }
}
