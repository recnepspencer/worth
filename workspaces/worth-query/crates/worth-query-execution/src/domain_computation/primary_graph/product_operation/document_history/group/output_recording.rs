//! A complete group chain lends its actual last receipt to final forest qualification.
use super::super::source_evidence::WorthQueryHistorySourceEvidence;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryCompletedRecordingOutputs, WorthQueryCompletedRequiredOutputs,
    WorthQueryHistoryRecordingOutputOrigin,
};
impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub(in crate::domain_computation::primary_graph) fn readmit_history_group_operation<M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        group: &WorthQueryDocumentHistoryGroupReservation,
    ) -> Result<(), WorthQueryDocumentHistoryDenial>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        self.application()
            .assess_document_history_operation::<H, M>(
                phase,
                access,
                self,
                installed,
                operation.selected,
                operation.admitted,
            )?;
        self.validate_reservation(&group.reservation)?;
        let meaning = operation.binding.bind_admission(operation.admitted);
        if !group.members.last().is_some_and(|last| {
            last.canonical_key() == meaning.canonical_key() && last.has_same_intent(meaning)
        }) {
            return Err(WorthQueryDocumentHistoryDenial::IntentDrift);
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn validate_group_recording_origin(
        &self,
        group: &WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        origin: &WorthQueryHistoryRecordingOutputOrigin,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        self.validate_group_output_origin(group, candidate)?;
        if !origin.matches_reservation(&group.reservation, candidate)
            || !group
                .members
                .last()
                .zip(group.completed.last())
                .is_some_and(|(meaning, last)| {
                    origin.matches_group_member(*meaning, &last.endpoint)
                })
        {
            return Err(WorthQueryDocumentHistoryDenial::ForeignRecording);
        }
        Ok(())
    }
    pub fn prepare_history_group_output_recording(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        group: &WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        source: WorthQueryHistorySourceEvidence<'_>,
        required: Option<&WorthQueryCompletedRequiredOutputs>,
        profile: &super::super::WorthQueryHistoryOutputProfile,
    ) -> Result<Arc<WorthQueryHistoryRecordingOutputOrigin>, WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        self.validate_group_output_origin(group, candidate)?;
        if !profile.is_output_forest() || !profile.matches_recording_model(&self.model) {
            return Err(Denial::ForeignModel);
        }
        let last = group.completed.last().ok_or(Denial::GroupNotComplete)?;
        let scope = group.scopes.last().ok_or(Denial::GroupNotComplete)?;
        let meaning = *group.members.last().ok_or(Denial::GroupNotComplete)?;
        if !source.matches(scope, meaning) {
            return Err(Denial::ReceiptMismatch);
        }
        match (last.root_posture, required) {
            (None, None) if source.is_selected_at(last.endpoint.product_observation()) => {}
            (Some(posture), Some(required))
                if match source {
                    WorthQueryHistorySourceEvidence::Live(receipt) => {
                        required.matches_source(receipt)
                    }
                    WorthQueryHistorySourceEvidence::Recorded(_) => false,
                } && required.root_posture() == posture
                    && required.retained_output().product_observation()
                        == last.endpoint.product_observation() => {}
            _ => return Err(Denial::ReceiptMismatch),
        }
        let current = self
            .application()
            .on_branch(candidate.branch())
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        if current.product().observation() != last.endpoint.product_observation() {
            return Err(Denial::ReceiptMismatch);
        }
        let execution = phase
            .checked_execution_request_for(self.application().product_runtime())
            .map_err(Denial::Stopped)?;
        WorthQueryHistoryRecordingOutputOrigin::capture(
            &execution,
            self.model.clone(),
            &group.reservation,
            candidate.branch(),
            match source {
                WorthQueryHistorySourceEvidence::Live(receipt) => receipt.idempotency_binding(),
                WorthQueryHistorySourceEvidence::Recorded(_) => meaning,
            },
            profile,
            Arc::clone(&last.endpoint),
            last.root_posture,
        )
    }

    pub(in crate::domain_computation::primary_graph::product_operation::document_history) fn validate_group_output_origin(
        &self,
        group: &WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        self.validate_reservation(&group.reservation)?;
        if group.completed.len() != group.members.len() || group.completed.is_empty() {
            return Err(Denial::GroupNotComplete);
        }
        if !std::ptr::eq(self.application(), candidate.application())
            || candidate.expected_source() != group.reservation.expected
            || group
                .candidate
                .as_ref()
                .is_none_or(|(branch, initial)| *branch != candidate.branch() || initial.is_none())
        {
            return Err(Denial::ForeignRecording);
        }
        Ok(())
    }

    pub fn capture_history_group_output_forest(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _: &WorthQueryProgramPublicationAccess,
        group: &mut WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        source: WorthQueryHistorySourceEvidence<'_>,
        required: Option<&WorthQueryCompletedRequiredOutputs>,
        completed: &WorthQueryCompletedRecordingOutputs,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        self.validate_group_output_origin(group, candidate)?;
        let scope = group.scopes.last().ok_or(Denial::GroupNotComplete)?;
        let meaning = *group.members.last().ok_or(Denial::GroupNotComplete)?;
        if !source.matches(scope, meaning) {
            return Err(Denial::ReceiptMismatch);
        }
        completed.require_source(&group.reservation, candidate, scope, source, required)?;
        let current = self
            .application()
            .on_branch(candidate.branch())
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        if current.product().observation() != completed.observation().product_observation() {
            return Err(Denial::ReceiptMismatch);
        }
        if let Some(outputs) = &group.outputs {
            if outputs.endpoint.product_observation()
                == completed.observation().product_observation()
                && Arc::ptr_eq(&outputs.recipes, &completed.discovery_recipes)
            {
                return Ok(());
            }
            return Err(Denial::ReceiptMismatch);
        }
        group.outputs = Some(CapturedGroupOutputs {
            endpoint: Arc::clone(completed.observation()),
            recipes: Arc::clone(&completed.discovery_recipes),
            root_posture: completed.root_posture(),
        });
        Ok(())
    }
}
