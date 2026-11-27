//! Complete-list admission rejects ineligible members before the single fork.
use super::super::{edit::ForkRecovery, recording};
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
    pub(super) fn construct(&mut self) -> Result<(), WorthQueryHistoryGroupDenial> {
        self.frozen = true;
        if self.members.len() == 0 {
            return Err(WorthQueryHistoryGroupDenial::History(
                WorthQueryApplicationHistoryDenial::Document(
                    WorthQueryDocumentHistoryDenial::EmptyGroup,
                ),
            ));
        }
        for index in 0..self.members.len() {
            self.members.member_mut(index).construct(&self.program)?;
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
        let context =
            DocumentRequestContext::bind(&document.owner, request).map_err(Denial::History)?;
        let current = context.request();
        context
            .advance(|phase| {
                document.read_in_advancement(phase, current)?;
                context.admit_metadata(
                    phase,
                    document.access.clone(),
                    &document.payload,
                    Ungoverned,
                    |runtime, installed, read| {
                        runtime.read_document_history_step(
                            phase,
                            &program_publication_access(),
                            &document.owner,
                            read,
                        )?;
                        // This inner result preserves SDK admission errors while the outer
                        // metadata receiver remains the first protected disclosure boundary.
                        Ok((|| {
                            let mut members = Vec::with_capacity(self.members.len());
                            for index in 0..self.members.len() {
                                members.push(self.members.member(index).admit(
                                    phase,
                                    current,
                                    &document.owner,
                                    installed,
                                )?);
                            }
                            self.members.validate_outputs(&self.program)?;
                            let prepared = document
                                .owner
                                .reserve_history_group(
                                    phase,
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
                                            phase,
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
            })
            .map_err(Denial::History)?
    }

    pub(super) fn fork<Payload: WorthQueryHistoryPayloadMode<S, H>, Outputs>(
        &mut self,
        document: &WorthQueryApplicationHistoryDocument<'runtime, S, H, Payload, Outputs>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryHistoryGroupDenial> {
        use WorthQueryHistoryGroupDenial as Denial;
        // The recovery slot leaves the execution while the fork borrows it.
        let mut recovery = self.recovery.take();
        let forked =
            recording::fork_candidate(document, request, &mut recovery, |phase, recovery| {
                let reservation = self.reservation.as_mut().expect("reserved group");
                let created = match document.owner.create_history_group_candidate(
                    phase,
                    &program_publication_access(),
                    reservation,
                ) {
                    Ok(created) => created,
                    Err(cause) => {
                        return Err(Denial::Member(ForkRecovery::creation_refused(
                            recovery, cause,
                        )))
                    }
                };
                self.candidate = Some(created.candidate);
                self.posture = WorthQueryHistoryGroupPosture::Running;
                if let Some(cause) = created.denial {
                    self.posture = WorthQueryHistoryGroupPosture::CandidateUnavailable;
                    return Err(Denial::History(
                        WorthQueryApplicationHistoryDenial::Document(cause),
                    ));
                }
                self.pause_at(WorthQueryHistoryGroupHandoffPointForTest::Candidate);
                document
                    .owner
                    .validate_record_delivery(phase, &program_publication_access())
                    .map_err(|cause| {
                        Denial::History(WorthQueryApplicationHistoryDenial::Document(cause))
                    })
            });
        self.recovery = recovery;
        forked
    }
}
