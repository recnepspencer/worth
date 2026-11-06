//! Complete-list admission rejects ineligible members before the single fork.
use super::*;
use worth_query_execution::facade::product::WorthQueryProductBranchCreateError;

impl<'runtime, S, H, I, P, A> WorthQueryHistoryGroupExecution<'runtime, S, H, I, P, A>
where
    S: ApplicationSchema + 'static,
    H: ApplicationHistoryBinding<S>,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, I::Binding>,
    Mode<S, I, A>: WorthQueryHistoryOperationMode<S, I, P, A>,
    I::Binding: WorthQuerySourceOperationBinding<S>
        + ApplicationMutationBinding<S, SourceExpectation = NoApplicationMutationSource>,
    <I::Binding as ApplicationMutationBinding<S>>::ScopeBinding:
        ApplicationMutationScopeBinding<S, Scope = ApplicationHistoryModel<S, H>>,
    Scope<S, H>: ApplicationQueryScopeResolution<S, Principal<S, H>>,
    Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
{
    pub(super) fn construct(&mut self) -> Result<(), WorthQueryHistoryGroupDenial> {
        for member in &mut self.members {
            if let Member::Description(description) = member {
                match Mode::<S, I, A>::construct(
                    description.take().expect("original description"),
                    &self.program,
                ) {
                    Ok(operation) => *member = Member::Unbound(Some(operation)),
                    Err((original, cause)) => {
                        *description = Some(original);
                        return Err(WorthQueryHistoryGroupDenial::Construction(cause));
                    }
                }
            }
        }
        self.posture = WorthQueryHistoryGroupPosture::Admitting;
        Ok(())
    }

    pub(super) fn admit<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        let runtime = document.owner.application();
        runtime
            .with_application_advancement(request.scope, |phase| {
                document
                    .read_in_advancement(&phase, request)
                    .map_err(Denial::History)?;
                let current = WorthQueryApplicationRequest {
                    application: runtime,
                    principal: request.principal,
                    scope: request.scope,
                    branch: document.owner.branch(),
                };
                super::super::metadata::with_metadata_read::<S, H, _>(
                    &phase,
                    &current,
                    document.access.clone(),
                    &document.payload,
                    |runtime, installed, read| {
                        runtime.read_document_history_step(
                            &phase,
                            &program_publication_access(),
                            &document.owner,
                            read,
                        )?;
                        // This inner result preserves SDK admission errors while the outer
                        // metadata receiver remains the first protected disclosure boundary.
                        Ok((|| {
                            let mut members = Vec::with_capacity(self.members.len());
                            for member in &self.members {
                                let Member::Unbound(Some(operation)) = member else {
                                    unreachable!("all members constructed before admission")
                                };
                                let (prepared, selected) = Mode::<S, I, A>::prepare(
                                    operation, &phase, &current,
                                )
                                .map_err(|cause| {
                                    Denial::History(WorthQueryApplicationHistoryDenial::Mutation(
                                        cause,
                                    ))
                                })?;
                                members.push(
                                    document
                                        .owner
                                        .admit_history_group_member::<I::Binding>(
                                            &phase,
                                            &program_publication_access(),
                                            installed,
                                            WorthQueryDocumentHistoryOperation::new(
                                                &selected,
                                                &prepared.admission,
                                                prepared.idempotency,
                                            ),
                                        )
                                        .map_err(|cause| {
                                            Denial::History(
                                                WorthQueryApplicationHistoryDenial::Document(cause),
                                            )
                                        })?,
                                );
                            }
                            Mode::<S, I, A>::validate_group_outputs(&self.program).map_err(
                                |cause| {
                                    Denial::History(WorthQueryApplicationHistoryDenial::Document(
                                        cause,
                                    ))
                                },
                            )?;
                            let prepared = document
                                .owner
                                .reserve_history_group(
                                    &phase,
                                    &program_publication_access(),
                                    installed,
                                    self.key,
                                    members,
                                )
                                .map_err(|cause| {
                                    Denial::History(WorthQueryApplicationHistoryDenial::Document(
                                        cause,
                                    ))
                                })?;
                            // Store actual custody before final request delivery can stop.
                            match prepared {
                                WorthQueryDocumentHistoryGroupPreparation::Reserved(group) => {
                                    self.reservation = Some(group);
                                    self.posture = WorthQueryHistoryGroupPosture::CreatingCandidate;
                                    self.pause_at(
                                        WorthQueryHistoryGroupHandoffPointForTest::Reservation,
                                    );
                                    document
                                        .owner
                                        .validate_record_delivery(
                                            &phase,
                                            &program_publication_access(),
                                        )
                                        .map_err(|cause| {
                                            Denial::History(
                                                WorthQueryApplicationHistoryDenial::Document(cause),
                                            )
                                        })?;
                                }
                                WorthQueryDocumentHistoryGroupPreparation::AlreadyRecorded(
                                    step,
                                ) => {
                                    self.selected = Some(step);
                                    self.posture = WorthQueryHistoryGroupPosture::AlreadyRecorded;
                                }
                            }
                            Ok(())
                        })())
                    },
                )
                .map_err(Denial::History)?
            })
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
            })?
    }

    pub(super) fn fork<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        if let Err(cause) =
            super::super::edit::ForkRecovery::release_in_request(&mut self.recovery, request)
        {
            return Err(WorthQueryHistoryGroupDenial::Member(cause));
        }
        let runtime = document.owner.application();
        runtime.with_application_advancement(request.scope,|phase| {
            document.read_in_advancement(&phase,request).map_err(WorthQueryHistoryGroupDenial::History)?;
            match document.owner.create_history_group_candidate(&phase,&program_publication_access(),self.reservation.as_mut().expect("reserved group")) {
                Ok(created)=> {
                    self.candidate=Some(created.candidate);
                    self.posture=WorthQueryHistoryGroupPosture::Running;
                    if let Some(cause)=created.denial {
                        self.posture=WorthQueryHistoryGroupPosture::CandidateUnavailable;
                        return Err(WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(cause)));
                    }
                    self.pause_at(WorthQueryHistoryGroupHandoffPointForTest::Candidate);
                    document.owner.validate_record_delivery(&phase,&program_publication_access())
                        .map_err(|cause|WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Document(cause)))?;
                }
                Err(worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryCandidateDenial::Creation(WorthQueryProductBranchCreateError::ProductUnpublished(owner)))=> {
                    self.recovery=Some(super::super::edit::ForkRecovery::Creation(owner));return Err(WorthQueryHistoryGroupDenial::Member(WorthQueryHistoryEditDenial::ForkRecovery));
                }
                Err(cause)=>return Err(WorthQueryHistoryGroupDenial::Member(WorthQueryHistoryEditDenial::Candidate(cause))),
            }
            Ok(())
        }).map_err(|cause|WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause)))??;
        Ok(())
    }
}
