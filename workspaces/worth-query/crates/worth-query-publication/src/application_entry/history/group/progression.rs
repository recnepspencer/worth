//! Business work stays on the original ordinary SDK owners of the one private candidate.
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
    pub fn advance<
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs: WorthQueryHistoryOutputsMode<S, H>,
    >(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryHistoryGroupPosture, WorthQueryHistoryGroupDenial> {
        self.validate_document(document, request)?;
        match self.posture {
            WorthQueryHistoryGroupPosture::Constructing => self.construct()?,
            WorthQueryHistoryGroupPosture::Admitting => self.admit(document, request)?,
            WorthQueryHistoryGroupPosture::CreatingCandidate => self.fork(document, request)?,
            WorthQueryHistoryGroupPosture::Running => self.run_member(document, request)?,
            WorthQueryHistoryGroupPosture::QualifyingOutputs => {
                self.qualify_outputs(document, request)?
            }
            _ => {}
        }
        Ok(self.posture)
    }

    fn run_member<
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs: WorthQueryHistoryOutputsMode<S, H>,
    >(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        if self.members.member(self.next).posture() == storage::MemberPosture::Completed {
            return self.capture_member(document, request);
        }
        if self.members.member(self.next).posture() == storage::MemberPosture::Unbound {
            let restriction = document
                .owner
                .application()
                .with_application_advancement(request.scope, |phase| {
                    document
                        .read_in_advancement(&phase, request)
                        .map_err(Denial::History)?;
                    document
                        .owner
                        .history_group_operation_constraint(
                            &phase,
                            &program_publication_access(),
                            self.reservation.as_ref().expect("reserved group"),
                            self.candidate.as_ref().expect("private candidate"),
                            self.next,
                        )
                        .map_err(|cause| {
                            Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
                        })
                })
                .map_err(|cause| {
                    Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
                })??;
            self.members.member_mut(self.next).bind(restriction)?;
            return Ok(());
        }
        let current = WorthQueryApplicationRequest {
            application: document.owner.application(),
            principal: request.principal,
            scope: request.scope,
            branch: self.candidate.as_ref().expect("private candidate").branch(),
        };
        self.members.member_mut(self.next).advance(&current)?;
        if self.members.member(self.next).posture() == storage::MemberPosture::Terminal {
            self.posture = WorthQueryHistoryGroupPosture::Terminal;
        }
        Ok(())
    }

    fn capture_member<
        Payload: WorthQueryHistoryPayloadMode<S, H>,
        Outputs: WorthQueryHistoryOutputsMode<S, H>,
    >(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        document
            .owner
            .application()
            .with_application_advancement(request.scope, |phase| {
                document
                    .read_in_advancement(&phase, request)
                    .map_err(Denial::History)?;
                let current = WorthQueryApplicationRequest {
                    application: document.owner.application(),
                    principal: request.principal,
                    scope: request.scope,
                    branch: self.candidate.as_ref().unwrap().branch(),
                };
                let source = self
                    .members
                    .member(self.next)
                    .recording_source(&phase, &current)?;
                let reservation = self.reservation.as_mut().expect("reserved group");
                let candidate = self.candidate.as_ref().unwrap();
                match source {
                    HistoryRecordingSource::Source(receipt) => {
                        document.owner.capture_history_group_member(
                            &phase,
                            &program_publication_access(),
                            reservation,
                            candidate,
                            receipt,
                        )
                    }
                    HistoryRecordingSource::RequiredOutputs { source, completed } => {
                        document.owner.capture_history_group_required_outputs(
                            &phase,
                            &program_publication_access(),
                            reservation,
                            candidate,
                            source,
                            &completed,
                        )
                    }
                }
                .map_err(|cause| {
                    Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
                })?;
                self.next += 1;
                if self.next == self.members.len() {
                    self.posture = if document.outputs.records_output_forest() {
                        WorthQueryHistoryGroupPosture::QualifyingOutputs
                    } else {
                        WorthQueryHistoryGroupPosture::ReadyToComplete
                    };
                }
                Ok(())
            })
            .map_err(|cause| {
                Denial::History(WorthQueryApplicationHistoryDenial::ExecutionRequest(cause))
            })?
    }

    /// Query the actual staged head under fresh ordinary READ. This creates no
    /// history node or permission and exposes no candidate mutation selector.
    pub fn read_staged<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<Query<S, H>, Rows<S, H>>,
        WorthQueryHistoryGroupDenial,
    > {
        self.validate_document(document, request)?;
        document
            .current_step(request)
            .map_err(WorthQueryHistoryGroupDenial::History)?;
        let candidate = self
            .candidate
            .as_ref()
            .ok_or(WorthQueryHistoryGroupDenial::NotReady)?;
        WorthQueryApplicationRequest {
            application: request.application,
            principal: request.principal,
            scope: request.scope,
            branch: candidate.branch(),
        }
        .query(document.access.clone())
        .execute()
        .map_err(|cause| {
            WorthQueryHistoryGroupDenial::History(WorthQueryApplicationHistoryDenial::Query(cause))
        })
    }
}
