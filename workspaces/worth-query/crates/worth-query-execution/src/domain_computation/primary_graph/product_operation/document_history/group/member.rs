//! A member can execute only on its group's original candidate and exact intent.
use super::super::source_evidence::HistorySourceEvidence;
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryCompletedRequiredOutputs,
    WorthQueryRecoveredApplicationCommitReceipt,
};

enum MemberCompletion<'a> {
    Source(HistorySourceEvidence<'a>),
    RequiredOutputs {
        receipt: &'a WorthQueryApplicationCommitReceipt,
        completed: &'a WorthQueryCompletedRequiredOutputs,
    },
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn history_group_operation_constraint(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        group: &WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        member: usize,
    ) -> Result<WorthQueryDocumentHistoryOperationConstraint, WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        self.validate_reservation(&group.reservation)?;
        if member != group.completed.len() {
            return Err(Denial::GroupMemberMismatch);
        }
        let meaning = *group
            .members
            .get(member)
            .ok_or(Denial::GroupMemberMismatch)?;
        if !std::ptr::eq(self.application(), candidate.application()) {
            return Err(Denial::ForeignRuntime);
        }
        if candidate.expected_source() != group.reservation.expected {
            return Err(Denial::HeadDiverged);
        }
        let Some((branch, Some(initial))) = &group.candidate else {
            return Err(Denial::GroupMemberMismatch);
        };
        if *branch != candidate.branch() {
            return Err(Denial::ForeignRecording);
        }
        let expected = group
            .completed
            .last()
            .map(|source| {
                WorthQueryProductBranchReadIdentity::from_observation(
                    source.endpoint.product_observation(),
                )
            })
            .unwrap_or_else(|| initial.clone());
        let selected = self
            .application()
            .on_branch(candidate.branch())
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        if !expected.matches_observation(selected.product().observation()) {
            return Err(Denial::HeadDiverged);
        }
        Ok(WorthQueryDocumentHistoryOperationConstraint::for_group(
            self.model.clone(),
            candidate.retained_expected_source(),
            candidate.branch(),
            meaning,
            expected,
        ))
    }

    /// The actual ordinary completion must still name the selected candidate
    /// endpoint. A payload-equal later publication is not interchangeable.
    pub fn capture_history_group_member(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        group: &mut WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        self.capture_member(
            phase,
            group,
            candidate,
            MemberCompletion::Source(HistorySourceEvidence::Live(receipt)),
        )
    }

    /// Fresh history admission compares the original owner's recorded endpoints.
    pub fn capture_history_group_original_member(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        group: &mut WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryRecoveredApplicationCommitReceipt,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        self.capture_member(
            phase,
            group,
            candidate,
            MemberCompletion::Source(HistorySourceEvidence::Recorded(receipt)),
        )
    }

    /// The original complete-set proof binds the member's endpoint, including
    /// real root and child publications after its source mutation.
    pub fn capture_history_group_required_outputs(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        _access: &WorthQueryProgramPublicationAccess,
        group: &mut WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        receipt: &WorthQueryApplicationCommitReceipt,
        completed: &WorthQueryCompletedRequiredOutputs,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        self.capture_member(
            phase,
            group,
            candidate,
            MemberCompletion::RequiredOutputs { receipt, completed },
        )
    }

    fn capture_member(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        group: &mut WorthQueryDocumentHistoryGroupReservation,
        candidate: &WorthQueryExpectedProductBranch<'runtime, S>,
        completion: MemberCompletion<'_>,
    ) -> Result<(), WorthQueryDocumentHistoryDenial> {
        use WorthQueryDocumentHistoryDenial as Denial;
        self.validate_reservation(&group.reservation)?;
        let expected = group
            .members
            .get(group.completed.len())
            .ok_or(Denial::GroupMemberMismatch)?;
        let source = match &completion {
            MemberCompletion::Source(source) => *source,
            MemberCompletion::RequiredOutputs { receipt, .. } => {
                HistorySourceEvidence::Live(receipt)
            }
        };
        if !source.matches(&self.model, *expected) {
            return Err(Denial::ReceiptMismatch);
        }
        if !std::ptr::eq(self.application(), candidate.application())
            || candidate.expected_source() != group.reservation.expected
        {
            return Err(Denial::ForeignRecording);
        }
        let Some((branch, Some(initial))) = &group.candidate else {
            return Err(Denial::GroupMemberMismatch);
        };
        if *branch != candidate.branch() {
            return Err(Denial::ForeignRecording);
        }
        let previous = group
            .completed
            .last()
            .map(|source| {
                WorthQueryProductBranchReadIdentity::from_observation(
                    source.endpoint.product_observation(),
                )
            })
            .unwrap_or_else(|| initial.clone());
        if !source.follows(&previous) {
            return Err(Denial::ReceiptMismatch);
        }
        let selected = self
            .application()
            .on_branch(candidate.branch())
            .select(phase)
            .map_err(Denial::CandidateSelection)?;
        let captured = match completion {
            MemberCompletion::Source(source) => {
                if !source.is_selected_at(selected.product().observation()) {
                    return Err(Denial::ReceiptMismatch);
                }
                CapturedMember {
                    endpoint: selected.retain_application_read(),
                    root_posture: None,
                }
            }
            MemberCompletion::RequiredOutputs { receipt, completed } => {
                let retained = completed.retained_output();
                if !completed.matches_source(receipt)
                    || retained.branch_incarnation() != candidate.branch().occurrence()
                    || retained.product_observation() != selected.product().observation()
                {
                    return Err(Denial::ReceiptMismatch);
                }
                CapturedMember {
                    endpoint: Arc::clone(retained),
                    root_posture: Some(completed.root_posture()),
                }
            }
        };
        group.completed.push(captured);
        Ok(())
    }
}
