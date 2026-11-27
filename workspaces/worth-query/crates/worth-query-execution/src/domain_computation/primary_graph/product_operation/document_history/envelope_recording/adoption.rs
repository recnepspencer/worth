//! The envelope's one commit becomes the document head through one adoption.
use super::super::{
    sealing::{BoundSource, PreparedBinding},
    WorthQueryCompletedRecordingOutputs, WorthQueryDocumentHistory,
    WorthQueryDocumentHistoryAdoptionFailure, WorthQueryDocumentHistoryDenial as Denial,
    WorthQueryDocumentHistoryReservation, WorthQueryHistoryStep,
};
use crate::{
    basis::WorthQueryProductBranchReadIdentity,
    domain_computation::primary_graph::{
        WorthQueryAdvancementPhase, WorthQueryApplicationCommitReceipt,
        WorthQueryExpectedProductBranch,
    },
    publication_boundary::WorthQueryProgramPublicationAccess,
};
use std::sync::Arc;
use worth_query_declaration::facade::{
    application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema,
};
use worth_query_execution_lineage::facade::ports::CommittedProductPublicationInternalPort as _;
use worth_query_execution_runtime::facade::ports::ProductBranchLeaseInternalPort as _;

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    /// Complete the reserved record with the candidate's committed endpoint and
    /// move the document head to it. A refusal returns both owners unchanged.
    pub fn adopt_history_envelope(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>> {
        let prepared = self.prepare_envelope_binding(phase, &reserved, &candidate, receipt, None);
        self.adopt_envelope_binding(phase, reserved, candidate, prepared)
    }

    /// As `adopt_history_envelope`, for a document with bound output trees: the
    /// step's endpoint is the one endpoint every tree was recorded at, and the
    /// step keeps the forest's discovery recipes.
    pub fn adopt_history_envelope_output_forest(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
        completed: &WorthQueryCompletedRecordingOutputs,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>> {
        let prepared =
            self.prepare_envelope_binding(phase, &reserved, &candidate, receipt, Some(completed));
        self.adopt_envelope_binding(phase, reserved, candidate, prepared)
    }

    fn adopt_envelope_binding(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: WorthQueryDocumentHistoryReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
        prepared: Result<PreparedBinding, Denial>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryAdoptionFailure<'runtime, S>> {
        match prepared {
            Ok(prepared) => self.adopt_prepared_history(phase, reserved, candidate, prepared),
            Err(denial) => Err(WorthQueryDocumentHistoryAdoptionFailure {
                reservation: reserved,
                candidate,
                denial,
            }),
        }
    }

    /// Without a forest the endpoint is the envelope's commit, still selected
    /// on the candidate. With one it is the forest's endpoint, which must come
    /// from this reservation, this candidate and this commit. The step is
    /// indexed here, at adoption, and nowhere else.
    pub(super) fn prepare_envelope_binding(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
        completed: Option<&WorthQueryCompletedRecordingOutputs>,
    ) -> Result<PreparedBinding, Denial> {
        let bound = self.bind_envelope_source(phase, reserved, candidate, receipt, completed)?;
        let step = self.index_bound_step(
            phase,
            reserved,
            bound.source_commit.as_ref(),
            bound.basis_commit,
            &bound.endpoint,
        )?;
        // A correction's commit posts one aftermath-causality fact, which
        // navigation retires and restores with this step (ruling (u)).
        let posted_fact = receipt.aftermath_causality().and_then(|causality| {
            match causality.fact_record() {
                worth_relational::facade::transactions::RecordRef::Entity(id) => Some(*id),
                worth_relational::facade::transactions::RecordRef::Relation(_) => None,
            }
        });
        let discovery_recipes = completed.map(|completed| completed.discovery_recipes.clone());
        Ok(bound.into_prepared(step, posted_fact, discovery_recipes))
    }

    /// The checks an envelope's binding relies on, and its source, endpoint
    /// and basis. It indexes nothing: the output origin calls it before
    /// adoption.
    pub(super) fn bind_envelope_source(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
        completed: Option<&WorthQueryCompletedRecordingOutputs>,
    ) -> Result<BoundSource, Denial> {
        self.validate_reservation(reserved)?;
        let runtime = self.application();
        if !std::ptr::eq(runtime, candidate.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if candidate.expected_source() != reserved.expected {
            return Err(Denial::HeadDiverged);
        }
        let (selected, retained) = match completed {
            Some(completed) => {
                completed.require_envelope_source(reserved, candidate, receipt)?;
                let retained = completed.endpoint();
                let selected = runtime
                    .select_application_read_observation(phase, retained)
                    .map_err(Denial::CandidateSelection)?;
                (selected, Some(Arc::clone(retained)))
            }
            None => {
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
                (selected, None)
            }
        };
        Ok(BoundSource {
            endpoint: WorthQueryProductBranchReadIdentity::from_observation(
                selected.product().observation(),
            ),
            exact: selected.product().read_lease(),
            source_commit: Some(receipt.committed_product_publication().composite_commit().clone()),
            // The basis a later navigation to this step selects (its retained source).
            basis_commit: selected.product().relational_basis().observation().commit_id(),
            root_posture: None,
            source: retained.unwrap_or_else(|| selected.retain_application_read()),
        })
    }
}
