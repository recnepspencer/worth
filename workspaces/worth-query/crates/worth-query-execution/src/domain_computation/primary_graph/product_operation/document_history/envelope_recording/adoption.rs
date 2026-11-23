//! The envelope's one commit becomes the document head through one adoption.
use super::super::{
    journal::RecordedNode, sealing::PreparedBinding, WorthQueryDocumentHistory,
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
        match self.prepare_envelope_binding(phase, &reserved, &candidate, receipt) {
            Ok(prepared) => self.adopt_prepared_history(phase, reserved, candidate, prepared),
            Err(denial) => Err(WorthQueryDocumentHistoryAdoptionFailure {
                reservation: reserved,
                candidate,
                denial,
            }),
        }
    }

    fn prepare_envelope_binding(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        reserved: &WorthQueryDocumentHistoryReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<PreparedBinding, Denial> {
        self.validate_reservation(reserved)?;
        let runtime = self.application();
        if !std::ptr::eq(runtime, candidate.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if candidate.expected_source() != reserved.expected {
            return Err(Denial::HeadDiverged);
        }
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
        let endpoint =
            WorthQueryProductBranchReadIdentity::from_observation(selected.product().observation());
        let exact = selected.product().read_lease();
        let step = self
            .selection
            .lock()
            .journal
            .prepared_step(&reserved.record, endpoint.clone());
        let node = RecordedNode {
            step,
            root_posture: None,
        };
        Ok(PreparedBinding {
            discovery_recipes: None,
            returned: node.step.clone(),
            node,
            source: selected.retain_application_read(),
            endpoint,
            exact,
        })
    }
}
