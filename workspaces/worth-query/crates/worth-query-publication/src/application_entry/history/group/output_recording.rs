//! One final forest qualifies the group's actual candidate and freezes its recipes.
use super::*;
impl<'runtime, S, H, P, M> WorthQueryHistoryGroupExecution<'runtime, S, H, P, M>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    P: ApplicationProgramDefinition<S>,
    M: WorthQueryHistoryGroupMembers<S, H, P>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub(super) fn qualify_outputs<Payload, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial>
    where
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs: WorthQueryHistoryOutputsMode<S, H>,
    {
        use WorthQueryHistoryGroupDenial as Denial;
        let runtime = document.owner.application();
        runtime
            .with_application_advancement(request.scope, |phase| {
                document
                    .read_in_advancement(&phase, request)
                    .map_err(Denial::History)?;
                let installed = runtime
                    .installed_schema()
                    .installed_history::<H>()
                    .map_err(|cause| {
                        Denial::History(WorthQueryApplicationHistoryDenial::Installation(cause))
                    })?;
                let current = WorthQueryApplicationRequest {
                    application: runtime,
                    principal: request.principal,
                    scope: request.scope,
                    branch: document.owner.branch(),
                };
                let candidate = self.candidate.as_ref().expect("actual private candidate");
                let candidate_request = WorthQueryApplicationRequest {
                    application: runtime,
                    principal: request.principal,
                    scope: request.scope,
                    branch: candidate.branch(),
                };
                let member = self.members.member(self.members.len() - 1);
                document
                    .outputs
                    .with_recording(&phase, |forest| {
                        let result = (|| {
                            let Some(forest) = forest else {
                                self.posture = WorthQueryHistoryGroupPosture::ReadyToComplete;
                                return Ok(());
                            };
                            if forest.needs_origin() {
                                let source = member.recording_source(&phase, &current)?;
                                let (receipt, required) = source.parts();
                                let origin = document
                                    .owner
                                    .prepare_history_group_output_recording(
                                        &phase,
                                        &program_publication_access(),
                                        self.reservation.as_ref().expect("reserved group"),
                                        candidate,
                                        receipt,
                                        required,
                                        forest.profile(),
                                    )
                                    .map_err(|cause| {
                                        Denial::History(
                                            WorthQueryApplicationHistoryDenial::Document(cause),
                                        )
                                    })?;
                                forest.bind_origin(origin).map_err(Denial::Outputs)?;
                            }
                            let origin =
                                std::sync::Arc::clone(forest.origin().expect("bound origin"));
                            let complete = document
                                .payload
                                .with_read(
                                    current.query(document.access.clone()),
                                    &phase,
                                    None,
                                    |_, binding, external, plan| {
                                        let read = WorthQueryDocumentHistoryRead::new(
                                            &installed, binding, external, plan,
                                        );
                                        Ok(member.with_recording_outputs(
                                            &phase,
                                            &current,
                                            &document.owner,
                                            read,
                                            self.reservation.as_ref().expect("reserved group"),
                                            candidate,
                                            &origin,
                                            &mut |authority| {
                                                forest.advance(
                                                    &phase,
                                                    &candidate_request,
                                                    authority,
                                                )
                                            },
                                        ))
                                    },
                                )
                                .map_err(|cause| {
                                    Denial::History(WorthQueryApplicationHistoryDenial::Query(
                                        cause,
                                    ))
                                })??;
                            if !complete {
                                return Ok(());
                            }
                            let endpoint =
                                crate::application_entry::WorthQueryApplicationReadObservation::new(
                                    std::sync::Arc::clone(
                                        forest.completed().expect("complete forest").observation(),
                                    ),
                                );
                            let source = member.recording_source_at(&phase, &current, &endpoint)?;
                            let (receipt, required) = source.parts();
                            forest
                                .drain_for_selection(&phase, candidate.branch())
                                .map_err(Denial::Outputs)?;
                            document
                                .owner
                                .capture_history_group_output_forest(
                                    &phase,
                                    &program_publication_access(),
                                    self.reservation.as_mut().expect("reserved group"),
                                    candidate,
                                    receipt,
                                    required,
                                    forest.completed().expect("complete forest"),
                                )
                                .map_err(|cause| {
                                    Denial::History(WorthQueryApplicationHistoryDenial::Document(
                                        cause,
                                    ))
                                })?;
                            self.posture = WorthQueryHistoryGroupPosture::ReadyToComplete;
                            Ok(())
                        })();
                        (result, false)
                    })
                    .map_err(Denial::Outputs)?
            })
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
            })?
    }
}
