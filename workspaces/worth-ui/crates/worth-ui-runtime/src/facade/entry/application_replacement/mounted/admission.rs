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
}

pub(super) struct WorthUiAdmittedMountedReplacement<'session> {
    pub(super) session: &'session mut WorthUiActiveApplicationSession,
    pub(super) application: Box<WorthUiPreparedApplicationActivation>,
    pub(super) mounted: crate::mounting::UiMountedGraphReplacementAdmission,
    pub(super) lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
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
    } = input;
    let prepared = session.mounted.prepare_graph_replacement_presentation(
        mounted_successor,
        frame,
        &session.host_session,
        deadline,
        now,
    );
    match prepared {
        crate::mounting::UiMountedGraphReplacementPreparation::Admitted(mounted) => {
            Ok(WorthUiAdmittedMountedReplacement {
                session,
                application,
                mounted,
                lifecycle,
            })
        }
        crate::mounting::UiMountedGraphReplacementPreparation::AdmissionDenied {
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
                WorthUiMountedApplicationReplacementOutcome::AdmissionDenied(
                    WorthUiMountedReplacementAdmissionDenial {
                        denial,
                        replacement: Box::new(WorthUiPreparedMountedApplicationReplacement {
                            session,
                            application,
                            mounted_successor: successor,
                            frame,
                            lifecycle,
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
                        }),
                    },
                ),
            ))
        }
    }
}
