use super::WorthUiActiveApplicationSession;

pub(crate) struct WorthUiPreparedMountedContentRebind<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    frame: crate::mounting::UiPreparedMountedFrame,
    publication: WorthUiMountedContentPublication,
}

pub(crate) struct WorthUiMountedContentRebindInFlight<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    mounted: crate::mounting::UiMountedPresentationInFlight,
    publication: WorthUiMountedContentPublication,
}

pub(crate) struct WorthUiDetachedPreparedMountedContentRebind {
    session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    publication: WorthUiMountedContentPublication,
}

pub(crate) struct WorthUiDetachedMountedContentRebindInFlight {
    session_identity: crate::facade::WorthUiActiveApplicationSessionIdentity,
    mounted: crate::mounting::UiMountedPresentationInFlight,
    publication: WorthUiMountedContentPublication,
}

pub(crate) struct WorthUiMountedContentRebindIndeterminate<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    frame: crate::mounting::UiMountedIndeterminateFrame,
}

enum WorthUiMountedContentPublication {
    RetainedGeneration,
    AuthoredSuccessor {
        authority:
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        appearance_succession: super::UiPreparedAppearanceGenerationSuccession,
    },
}

pub(crate) struct WorthUiMountedContentPublicationReceipt {
    mounted: crate::mounting::UiMountedFramePublicationReceipt,
    authored_generations: Option<(
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    )>,
}

pub(crate) enum WorthUiMountedContentRebindOutcome<'session> {
    Published(WorthUiMountedContentPublicationReceipt),
    RejectedBeforeEffects {
        rejections: Box<[crate::mounting::UiMountedSurfacePresentationRejection]>,
        retry: Box<WorthUiPreparedMountedContentRebind<'session>>,
    },
    InFlight(Box<WorthUiMountedContentRebindInFlight<'session>>),
    PresentationIndeterminate(Box<WorthUiMountedContentRebindIndeterminate<'session>>),
    RetentionDenied {
        denial: crate::mounting::UiMountedFrameRetentionDenial,
        retry: Box<WorthUiPreparedMountedContentRebind<'session>>,
    },
    AdmissionDenied {
        denial: crate::mounting::UiMountedPresentationAdmissionDenial,
        retry: Box<WorthUiPreparedMountedContentRebind<'session>>,
    },
    CompletionDenied(crate::mounting::UiMountedPresentationCompletionDenial),
}

impl<'session> WorthUiPreparedMountedContentRebind<'session> {
    pub(crate) fn new(
        session: &'session mut WorthUiActiveApplicationSession,
        frame: crate::mounting::UiPreparedMountedFrame,
    ) -> Self {
        Self {
            session,
            frame,
            publication: WorthUiMountedContentPublication::RetainedGeneration,
        }
    }

    pub(crate) fn authored(
        session: &'session mut WorthUiActiveApplicationSession,
        frame: crate::mounting::UiPreparedMountedFrame,
        successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        appearance_succession: super::UiPreparedAppearanceGenerationSuccession,
    ) -> Self {
        Self {
            session,
            frame,
            publication: WorthUiMountedContentPublication::AuthoredSuccessor {
                authority: successor,
                appearance_succession,
            },
        }
    }

    pub(crate) fn frame(&self) -> &crate::mounting::UiPreparedMountedFrame {
        &self.frame
    }

    pub(crate) fn detach(self: Box<Self>) -> WorthUiDetachedPreparedMountedContentRebind {
        let Self {
            session,
            frame: _,
            publication,
        } = *self;
        WorthUiDetachedPreparedMountedContentRebind {
            session_identity: session.session_identity(),
            publication,
        }
    }

    pub(crate) fn present(
        self: Box<Self>,
        deadline: worth_ui_host_contract::UiPresentationDeadline,
        now: u64,
    ) -> WorthUiMountedContentRebindOutcome<'session> {
        let Self {
            session,
            frame,
            publication,
        } = *self;
        let outcome = session.present_prepared_mounted_frame_internal(frame, deadline, now);
        finish(session, outcome, publication)
    }
}

impl<'session> WorthUiMountedContentRebindInFlight<'session> {
    pub(crate) fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.mounted.attempt()
    }

    pub(crate) fn deadline(&self) -> worth_ui_host_contract::UiPresentationDeadline {
        self.mounted.deadline()
    }

    pub(crate) fn detach(self: Box<Self>) -> WorthUiDetachedMountedContentRebindInFlight {
        let Self {
            session,
            mounted,
            publication,
        } = *self;
        WorthUiDetachedMountedContentRebindInFlight {
            session_identity: session.session_identity(),
            mounted,
            publication,
        }
    }

    pub(crate) fn complete(
        self: Box<Self>,
        now: u64,
    ) -> WorthUiMountedContentRebindOutcome<'session> {
        let Self {
            session,
            mounted,
            publication,
        } = *self;
        let outcome = session.complete_mounted_presentation(mounted, now);
        finish(session, outcome, publication)
    }

    pub(crate) fn cancel(self: Box<Self>) -> WorthUiMountedContentRebindOutcome<'session> {
        let Self {
            session,
            mounted,
            publication,
        } = *self;
        let outcome = session.supersede_mounted_presentation(mounted);
        finish(session, outcome, publication)
    }
}

impl WorthUiDetachedPreparedMountedContentRebind {
    pub(crate) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session_identity
    }

    pub(crate) fn rebase<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
    ) -> Result<
        Box<WorthUiPreparedMountedContentRebind<'session>>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        let frame_request = session.mounted_frame_request();
        let completion = session.execute_framework_turn(|_| {}).map_err(|_| {
            crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
        })?;
        let mut execution = completion.into_execution().map_err(|_| {
            crate::runtime::rebind::UiRebindPreparationDenial::FrameBoundaryUnavailable
        })?;
        let theme_values = execution.presentation.theme_values_source();
        let frame = execution
            .prepare_mounted_frame_with_content_internal(
                frame_request,
                semantic_content,
                theme_values,
            )
            .map_err(|denial| {
                crate::runtime::rebind::UiRebindPreparationDenial::ContentMountedPreparation(
                    Box::new(denial),
                )
            })?;
        Ok(Box::new(WorthUiPreparedMountedContentRebind {
            session,
            frame,
            publication: self.publication,
        }))
    }
}

impl WorthUiDetachedMountedContentRebindInFlight {
    pub(crate) fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session_identity
    }

    pub(crate) fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.mounted.attempt()
    }

    pub(crate) fn awaits_progress_class(
        &self,
        class: worth_ui_host_contract::UiHostPresentationProgressClass,
    ) -> bool {
        self.mounted.awaits_progress_class(class)
    }

    pub(crate) fn pending_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = worth_ui_host_contract::UiSurfaceBindingGeneration> + '_
    {
        self.mounted.pending_bindings()
    }

    pub(crate) fn complete<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
        now: u64,
    ) -> WorthUiMountedContentRebindOutcome<'session> {
        let outcome = session.complete_mounted_presentation(self.mounted, now);
        finish(session, outcome, self.publication)
    }

    pub(crate) fn cancel<'session>(
        self,
        session: &'session mut WorthUiActiveApplicationSession,
    ) -> WorthUiMountedContentRebindOutcome<'session> {
        let outcome = session.cancel_mounted_presentation(self.mounted);
        finish(session, outcome, self.publication)
    }
}

impl<'session> WorthUiMountedContentRebindIndeterminate<'session> {
    pub(crate) fn frame(&self) -> &crate::mounting::UiMountedIndeterminateFrame {
        &self.frame
    }

    pub(crate) fn into_parts(
        self: Box<Self>,
    ) -> (
        &'session mut WorthUiActiveApplicationSession,
        crate::mounting::UiMountedIndeterminateFrame,
    ) {
        let Self { session, frame } = *self;
        (session, frame)
    }
}

fn finish<'session>(
    session: &'session mut WorthUiActiveApplicationSession,
    outcome: crate::mounting::UiMountedFrameOutcome,
    publication: WorthUiMountedContentPublication,
) -> WorthUiMountedContentRebindOutcome<'session> {
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(receipt) => {
            let authored_generations = match publication {
                WorthUiMountedContentPublication::RetainedGeneration => None,
                WorthUiMountedContentPublication::AuthoredSuccessor {
                    authority,
                    appearance_succession,
                } => {
                    let generations = session.application.commit_evidence_only_rebind(authority);
                    session.commit_appearance_generation_succession(appearance_succession);
                    Some(generations)
                }
            };
            WorthUiMountedContentRebindOutcome::Published(WorthUiMountedContentPublicationReceipt {
                mounted: receipt,
                authored_generations,
            })
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            let rejections = rejected.rejections().to_vec().into_boxed_slice();
            WorthUiMountedContentRebindOutcome::RejectedBeforeEffects {
                rejections,
                retry: Box::new(
                    WorthUiPreparedMountedContentRebind::new(session, rejected.into_frame())
                        .with_publication(publication),
                ),
            }
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(mounted) => {
            WorthUiMountedContentRebindOutcome::InFlight(Box::new(
                WorthUiMountedContentRebindInFlight {
                    session,
                    mounted,
                    publication,
                },
            ))
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
            WorthUiMountedContentRebindOutcome::PresentationIndeterminate(Box::new(
                WorthUiMountedContentRebindIndeterminate { session, frame },
            ))
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            unreachable!("ordinary content rebind cannot overlap a superseding frame")
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejection) => {
            WorthUiMountedContentRebindOutcome::RetentionDenied {
                denial: rejection.denial(),
                retry: Box::new(
                    WorthUiPreparedMountedContentRebind::new(session, rejection.into_frame())
                        .with_publication(publication),
                ),
            }
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            WorthUiMountedContentRebindOutcome::AdmissionDenied {
                denial: rejection.denial(),
                retry: Box::new(
                    WorthUiPreparedMountedContentRebind::new(session, rejection.into_frame())
                        .with_publication(publication),
                ),
            }
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            WorthUiMountedContentRebindOutcome::CompletionDenied(denial)
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            unreachable!("explicit content preparation always presents a fresh mounted frame")
        }
    }
}

impl WorthUiPreparedMountedContentRebind<'_> {
    fn with_publication(mut self, publication: WorthUiMountedContentPublication) -> Self {
        self.publication = publication;
        self
    }
}

impl WorthUiMountedContentPublicationReceipt {
    pub(crate) fn into_parts(
        self,
    ) -> (
        crate::mounting::UiMountedFramePublicationReceipt,
        Option<(
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        )>,
    ){
        (self.mounted, self.authored_generations)
    }
}
