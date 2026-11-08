//! One original selection guard adopts the final endpoint and completes one edge.
use super::super::{journal::RecordedNode, sealing::PreparedBinding};
use super::*;

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    WorthQueryDocumentHistory<'runtime, S, H>
{
    pub fn adopt_history_group<M>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        operation: WorthQueryDocumentHistoryOperation<'_, '_, S, M>,
        group: WorthQueryDocumentHistoryGroupReservation,
        candidate: WorthQueryExpectedProductBranch<'runtime, S>,
    ) -> Result<WorthQueryHistoryStep, WorthQueryDocumentHistoryGroupAdoptionFailure<'runtime, S>>
    where
        M: ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    {
        use WorthQueryDocumentHistoryDenial as Denial;
        let prepared = (|| {
            self.readmit_history_group_operation(phase, access, installed, operation, &group)?;
            if group.completed.len() != group.members.len() {
                return Err(Denial::GroupNotComplete);
            }
            if !std::ptr::eq(self.application(), candidate.application())
                || candidate.expected_source() != group.reservation.expected
                || group.candidate.as_ref().is_none_or(|(branch, initial)| {
                    *branch != candidate.branch() || initial.is_none()
                })
            {
                return Err(Denial::ForeignRecording);
            }
            let last = group.completed.last().ok_or(Denial::GroupNotComplete)?;
            let source = Arc::clone(
                group
                    .outputs
                    .as_ref()
                    .map_or(&last.endpoint, |outputs| &outputs.endpoint),
            );
            if source.branch_incarnation() != candidate.branch().occurrence() {
                return Err(Denial::ReceiptMismatch);
            }
            let selected = self
                .application()
                .select_application_read_observation(phase, &source)
                .map_err(Denial::CandidateSelection)?;
            let endpoint = WorthQueryProductBranchReadIdentity::from_observation(
                selected.product().observation(),
            );
            let exact = selected.product().read_lease();
            let step = self
                .selection
                .lock()
                .journal
                .prepared_step(&group.reservation.record, endpoint.clone());
            Ok(PreparedBinding {
                discovery_recipes: group
                    .outputs
                    .as_ref()
                    .map(|outputs| Arc::clone(&outputs.recipes)),
                node: RecordedNode {
                    step: step.clone(),
                    root_posture: group
                        .outputs
                        .as_ref()
                        .map_or(last.root_posture, |outputs| outputs.root_posture),
                },
                source,
                endpoint,
                exact,
                returned: step,
            })
        })();
        let prepared = match prepared {
            Ok(prepared) => prepared,
            Err(denial) => {
                return Err(WorthQueryDocumentHistoryGroupAdoptionFailure {
                    reservation: group,
                    candidate,
                    denial,
                })
            }
        };
        let WorthQueryDocumentHistoryGroupReservation {
            reservation,
            members,
            scopes,
            completed,
            outputs,
            candidate: bound_candidate,
            _allocation,
        } = group;
        match self.adopt_prepared_history(phase, reservation, candidate, prepared) {
            Ok(step) => Ok(step),
            Err(failure) => {
                let (reservation, candidate, denial) = failure.into_parts();
                Err(WorthQueryDocumentHistoryGroupAdoptionFailure {
                    reservation: WorthQueryDocumentHistoryGroupReservation {
                        reservation,
                        members,
                        scopes,
                        completed,
                        outputs,
                        candidate: bound_candidate,
                        _allocation,
                    },
                    candidate,
                    denial,
                })
            }
        }
    }
}
