use super::{
    WorthUiActiveApplicationSession, WorthUiMountedApplicationReplacementOutcome,
    WorthUiMountedReplacementAdmissionDenial, WorthUiMountedReplacementRetentionDenial,
    WorthUiPreparedApplicationActivation, WorthUiPreparedMountedApplicationReplacement,
};

pub(super) struct WorthUiMountedReplacementAdmissionInput<'session> {
    pub(super) session: &'session mut WorthUiActiveApplicationSession,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
    pub(super) frame: crate::mounting::UiPreparedMountedFrame,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
    pub(super) owners: super::super::owner_succession::UiPreparedApplicationOwnerSuccession,
}

pub(super) struct WorthUiAdmittedMountedReplacement<'session> {
    pub(super) session: &'session mut WorthUiActiveApplicationSession,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) mounted: crate::mounting::UiMountedGraphReplacementAdmission,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
    pub(super) owners: super::super::owner_succession::UiPreparedApplicationOwnerSuccession,
}

pub(super) fn prepare_replacement_presentation(
    input: WorthUiMountedReplacementAdmissionInput<'_>,
    deadline: worth_ui_host_contract::UiPresentationDeadline,
    now: u64,
) -> Result<
    WorthUiAdmittedMountedReplacement<'_>,
    Box<WorthUiMountedApplicationReplacementOutcome<'_>>,
> {
    let WorthUiMountedReplacementAdmissionInput {
        session,
        application,
        mounted_successor,
        frame,
        lifecycle,
        owners,
    } = input;
    if !owners.is_current(session) {
        return Err(Box::new(WorthUiMountedApplicationReplacementOutcome::AdmissionDenied(
            WorthUiMountedReplacementAdmissionDenial {
                denial: crate::mounting::UiMountedPresentationAdmissionDenial::PreparedFrameBasisChanged,
                replacement: Box::new(WorthUiPreparedMountedApplicationReplacement {
                    session, application, mounted_successor, frame, lifecycle, owners,
                }),
            },
        )));
    }
    let authority = application.candidate_replacement_authority();
    let overlay_sources = session.prepare_replacement_overlay_appearance_sources(
        authority,
        &mounted_successor,
        &lifecycle.overlay_bindings,
    );
    let themes = application
        .appearance_succession
        .as_ref()
        .expect("replacement retains admitted theme succession")
        .theme();
    let mut overlay_attempt = None;
    let prepared = session.mounted.prepare_graph_replacement_presentation(
        mounted_successor,
        frame,
        &session.host_session,
        deadline,
        now,
        |attempt, surfaces| {
            overlay_attempt = Some(attempt);
            overlay_sources.as_ref().map_err(|_| ())?.lower_with_themes(
                attempt,
                surfaces,
                &mut session.overlay_composition_owners,
                authority.generation_identity(),
                session.portal.as_ref(),
                session.motion.as_ref(),
                &session.presentation,
                authority.capabilities(),
                owners.snapshot(),
                Some(themes),
                None,
            )
        },
    );
    if !matches!(
        prepared,
        crate::mounting::UiMountedGraphReplacementPreparation::Admitted(_)
    ) {
        if let Some(attempt) = overlay_attempt {
            session.overlay_composition_owners.discard(attempt);
        }
    }
    match prepared {
        crate::mounting::UiMountedGraphReplacementPreparation::Admitted(mounted) => {
            Ok(WorthUiAdmittedMountedReplacement {
                session,
                application,
                mounted,
                lifecycle,
                owners,
            })
        }
        crate::mounting::UiMountedGraphReplacementPreparation::AdmissionDenied {
            appearance,
            denial,
            successor,
            frame,
            observation,
        } => {
            crate::facade::entry::mounted_publication::record_mounted_observation(
                &mut session.host_exchange,
                observation,
            );
            if let Some(batch) = appearance {
                session
                    .appearance_inspection
                    .record_pre_effect_denials(batch.into_parts().1);
            }
            Err(Box::new(
                WorthUiMountedApplicationReplacementOutcome::AdmissionDenied(
                    WorthUiMountedReplacementAdmissionDenial {
                        denial,
                        replacement: Box::new(WorthUiPreparedMountedApplicationReplacement {
                            session,
                            application,
                            mounted_successor: successor,
                            frame,
                            lifecycle,
                            owners,
                        }),
                    },
                ),
            ))
        }
        crate::mounting::UiMountedGraphReplacementPreparation::RetentionDenied {
            denial,
            successor,
            frame,
            observation,
        } => {
            crate::facade::entry::mounted_publication::record_mounted_observation(
                &mut session.host_exchange,
                observation,
            );
            Err(Box::new(
                WorthUiMountedApplicationReplacementOutcome::RetentionDenied(
                    WorthUiMountedReplacementRetentionDenial {
                        denial,
                        replacement: Box::new(WorthUiPreparedMountedApplicationReplacement {
                            session,
                            application,
                            mounted_successor: successor,
                            frame,
                            lifecycle,
                            owners,
                        }),
                    },
                ),
            ))
        }
    }
}
