//! Every erased call still reaches its concrete ordinary admission and final receiver.
use super::super::storage::*;
use super::*;
impl<S, I, P, A> Member<S, I, P, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
{
    pub(super) fn construct(
        &mut self,
        program: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        if let Self::Description(description) = self {
            match Mode::<S, I, A>::construct(
                description.take().expect("original description"),
                program,
            ) {
                Ok(operation) => *self = Self::Unbound(Some(operation)),
                Err((original, cause)) => {
                    *description = Some(original);
                    return Err(WorthQueryHistoryGroupDenial::Construction(cause));
                }
            }
        }
        Ok(())
    }
    pub(super) fn admit<H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        owner: &WorthQueryDocumentHistory<'_, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
    ) -> Result<WorthQueryHistoryGroupMemberAdmission, WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        let Self::Unbound(Some(operation)) = self else {
            unreachable!("constructed before admission")
        };
        let (prepared, selected) =
            Mode::<S, I, A>::prepare(operation, phase, request).map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::Mutation(cause))
            })?;
        let admitted = owner
            .admit_history_group_member::<I::Binding>(
                phase,
                &program_publication_access(),
                installed,
                WorthQueryDocumentHistoryOperation::new(
                    &selected,
                    &prepared.admission,
                    prepared.idempotency,
                ),
            )
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
            })?;
        Ok(admitted)
    }
    pub(super) fn adopt<'r, H: ApplicationHistoryBinding<S>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        owner: &WorthQueryDocumentHistory<'r, S, H>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        group: WorthQueryDocumentHistoryGroupReservation,
        candidate: WorthQueryExpectedProductBranch<'r, S>,
    ) -> Result<
        WorthQueryHistoryStep,
        (
            WorthQueryDocumentHistoryGroupReservation,
            WorthQueryExpectedProductBranch<'r, S>,
            WorthQueryHistoryGroupDenial,
        ),
    > {
        let Self::Completed(Some(completed)) = self else {
            unreachable!("all members completed")
        };
        let (prepared, selected) =
            match Mode::<S, I, A>::prepare_completed(completed, phase, request) {
                Ok(admitted) => admitted,
                Err(cause) => {
                    return Err((
                        group,
                        candidate,
                        WorthQueryHistoryGroupDenial::History(
                            WorthQueryApplicationHistoryDenial::Mutation(cause),
                        ),
                    ))
                }
            };
        owner
            .adopt_history_group::<I::Binding>(
                phase,
                &program_publication_access(),
                installed,
                WorthQueryDocumentHistoryOperation::new(
                    &selected,
                    &prepared.admission,
                    prepared.idempotency,
                ),
                group,
                candidate,
            )
            .map_err(|failure| {
                (
                    failure.reservation,
                    failure.candidate,
                    WorthQueryHistoryGroupDenial::History(
                        WorthQueryApplicationHistoryDenial::Document(failure.denial),
                    ),
                )
            })
    }
}
