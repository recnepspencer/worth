//! One original selection guard adopts the final endpoint and completes one edge.
use super::super::sealing::BoundSource;
use super::*;
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;

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
            let (last, earlier) = group
                .completed
                .split_last()
                .ok_or(Denial::GroupNotComplete)?;
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
            // Every member's commit is a correction source for this step; the
            // last one is its node's source commit.
            let bound = BoundSource::selected(
                selected,
                Some(source),
                |_| Some(last.commit.clone()),
                group
                    .outputs
                    .as_ref()
                    .map_or(last.root_posture, |outputs| outputs.root_posture),
            );
            let discovery_recipes = group
                .outputs
                .as_ref()
                .map(|outputs| Arc::clone(&outputs.recipes));
            self.index_binding(
                phase,
                &group.reservation,
                bound,
                earlier.iter().map(|member| &member.commit),
                None,
                discovery_recipes,
            )
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
