use super::WorthUiActiveApplicationSession;

mod detached;
mod preparation;
mod theme;

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
    ThemeSwitch {
        theme: crate::runtime::appearance::UiThemeSwitchChange,
        reconciliation: Box<[crate::mounting::UiMountedSurfaceReconciliationBinding]>,
    },
    AuthoredSuccessor {
        authority:
            crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        appearance_succession: super::UiPreparedAppearanceGenerationSuccession,
        overlay_bindings: crate::runtime::portal::UiPortalOverlayBindingLifecycle,
        occurrence_geometry: crate::mounting::UiMountedOccurrenceGeometryState,
        pointer_succession:
            crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        owners: crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
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
    fn new(
        session: &'session mut WorthUiActiveApplicationSession,
        frame: crate::mounting::UiPreparedMountedFrame,
        publication: WorthUiMountedContentPublication,
    ) -> Self {
        Self {
            session,
            frame,
            publication,
        }
    }

    pub(in crate::facade::entry) fn prepare_authored(
        session: &'session mut WorthUiActiveApplicationSession,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
        successor: crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        appearance_succession: super::UiPreparedAppearanceGenerationSuccession,
        overlay_bindings: crate::runtime::portal::UiPortalOverlayBindingLifecycle,
        occurrence_geometry: crate::mounting::UiMountedOccurrenceGeometryState,
        pointer_succession: crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        owners: crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
    ) -> Result<Self, crate::runtime::rebind::UiRebindPreparationDenial> {
        let frame = session.prepare_authored_content_frame(
            semantic_content,
            session.mounted_frame_request(),
            &pointer_succession,
            &owners,
            &occurrence_geometry,
        )?;
        Ok(Self {
            session,
            frame,
            publication: WorthUiMountedContentPublication::AuthoredSuccessor {
                authority: successor,
                appearance_succession,
                overlay_bindings,
                occurrence_geometry,
                pointer_succession,
                owners,
            },
        })
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
        if let WorthUiMountedContentPublication::ThemeSwitch { theme, .. } = &self.publication {
            if self.frame.prepared_theme_binding() != Some(theme.prepared().successor())
                || self
                    .session
                    .presentation
                    .appearance_theme_state()
                    .is_none_or(|state| state.validate_prepared_switch(theme.prepared()).is_err())
            {
                return WorthUiMountedContentRebindOutcome::AdmissionDenied {
                    denial: crate::mounting::UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged,
                    retry: self,
                };
            }
        }
        if let WorthUiMountedContentPublication::AuthoredSuccessor {
            pointer_succession,
            owners,
            appearance_succession,
            authority,
            ..
        } = &self.publication
        {
            if pointer_succession
                .validate_predecessor(
                    &self.session.active_generation_identity(),
                    self.session.pointer_affordance_snapshot.as_ref(),
                    self.session
                        .observation_clock
                        .as_ref()
                        .map(|clock| clock.sample_millis()),
                    &self.session.mounted,
                )
                .is_err()
                || !owners.matches_projection(self.session.appearance_owner_snapshot.as_ref())
                || self
                    .session
                    .prepare_retained_appearance_owners(authority, appearance_succession)
                    .is_err()
            {
                return WorthUiMountedContentRebindOutcome::AdmissionDenied {
                    denial: crate::mounting::UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged,
                    retry: self,
                };
            }
        }
        let Self {
            session,
            frame,
            publication,
        } = *self;
        let outcome = match &publication {
            WorthUiMountedContentPublication::ThemeSwitch { reconciliation, .. }
                if !reconciliation.is_empty() =>
            {
                session
                    .present_prepared_mounted_frame_for_reconciliation(
                        frame,
                        reconciliation,
                        deadline,
                        now,
                    )
                    .expect("prepared theme reconciliation retains its accepted predecessor")
            }
            _ => session.present_prepared_mounted_frame_internal(frame, deadline, now),
        };
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
    if matches!(
        &outcome,
        crate::mounting::UiMountedFrameOutcome::Reconciled(_)
    ) {
        assert!(
            matches!(&publication, WorthUiMountedContentPublication::ThemeSwitch { reconciliation, .. } if !reconciliation.is_empty()),
            "only a prepared theme reconciliation may settle content as reconciled"
        );
    }
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(receipt)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(receipt) => {
            let authored_generations = match publication {
                WorthUiMountedContentPublication::RetainedGeneration => None,
                WorthUiMountedContentPublication::ThemeSwitch { theme, .. } => {
                    session.presentation.commit_published_appearance_theme_switch(theme.into_prepared())
                        .expect("the exclusively held accepted frame retains its admitted theme predecessor");
                    None
                }
                WorthUiMountedContentPublication::AuthoredSuccessor {
                    authority,
                    appearance_succession,
                    overlay_bindings,
                    occurrence_geometry,
                    pointer_succession,
                    owners,
                } => {
                    let generations = session.application.commit_evidence_only_rebind(authority);
                    session.commit_retained_appearance_succession(appearance_succession, owners);
                    session.authored_overlay_bindings = overlay_bindings;
                    session.pointer_affordance_snapshot = pointer_succession.into_snapshot();
                    session
                        .mounted
                        .commit_retained_geometry_succession(occurrence_geometry);
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
                retry: Box::new(WorthUiPreparedMountedContentRebind::new(
                    session,
                    rejected.into_frame(),
                    publication,
                )),
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
                retry: Box::new(WorthUiPreparedMountedContentRebind::new(
                    session,
                    rejection.into_frame(),
                    publication,
                )),
            }
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            WorthUiMountedContentRebindOutcome::AdmissionDenied {
                denial: rejection.denial(),
                retry: Box::new(WorthUiPreparedMountedContentRebind::new(
                    session,
                    rejection.into_frame(),
                    publication,
                )),
            }
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) => {
            WorthUiMountedContentRebindOutcome::CompletionDenied(denial)
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {
            unreachable!("explicit content preparation always presents a fresh mounted frame")
        }
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
