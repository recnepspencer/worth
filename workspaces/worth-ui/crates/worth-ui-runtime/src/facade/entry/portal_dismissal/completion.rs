use super::*;

pub(super) fn finish_presented<'session>(
    admitted: UiPortalDismissalAdmitted<'session>,
    outcome: crate::mounting::UiMountedFrameOutcome,
    now_tick: u64,
) -> UiPortalDismissalPublicationOutcome<'session> {
    let crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) = outcome else {
        return finish(admitted, outcome);
    };
    if rejected.rejections().is_empty()
        || !rejected.rejections().iter().all(|rejection| {
            rejection.denial()
                == worth_ui_host_contract::UiHostSurfacePresentationDenial::TextAtlasPresentationDeferred
        })
    {
        return finish(admitted, crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected));
    }
    let proposal = admitted
        .proposal
        .as_ref()
        .expect("deferred dismissal retains its proposal");
    let outcome = admitted.session.present_prepared_portal_frame_internal(
        rejected.into_frame(),
        proposal,
        admitted.retain_exit,
        worth_ui_host_contract::UiPresentationDeadline::at_tick(u64::MAX),
        now_tick,
    );
    finish(admitted, outcome)
}

pub(super) fn finish<'session>(
    mut admitted: UiPortalDismissalAdmitted<'session>,
    outcome: crate::mounting::UiMountedFrameOutcome,
) -> UiPortalDismissalPublicationOutcome<'session> {
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(mounted) => {
            let binding_commit = admitted
                .proposal
                .as_ref()
                .expect("published dismissal retains proposal")
                .overlay_binding_commit();
            let crate::runtime::session::UiPublishedPortalSettlement {
                focus,
                motion,
                exit_retention,
                reveal,
            } = admitted
                .session
                .application
                .settle_published_portal_service_proposal(
                    admitted
                        .proposal
                        .take()
                        .expect("published dismissal retains proposal"),
                    &mounted,
                    admitted
                        .session
                        .portal
                        .as_mut()
                        .expect("admitted dismissal retains Portal installation"),
                    admitted
                        .session
                        .focus
                        .as_mut()
                        .expect("admitted dismissal retains Focus installation"),
                    admitted.session.selection.as_mut(),
                    admitted
                        .session
                        .motion
                        .as_mut()
                        .expect("admitted dismissal retains Motion installation"),
                )
                .expect("published dismissal retains exact service proposal");
            binding_commit
                .and_then(|commit| {
                    admitted.session.commit_authored_overlay_binding(
                        commit.with_retained_exit(exit_retention.is_some()),
                    )
                })
                .expect("published dismissal retains its exact overlay binding stage");
            admitted
                .session
                .rebind_portal_after_current_published_frame();
            let focus = admitted
                .session
                .place_committed_semantic_focus(focus, &mounted)
                .expect("dismissal focus restoration retains exact mounted basis");
            admitted
                .session
                .install_portal_exit_retention(exit_retention);
            if let Some(reveal) = reveal {
                admitted.session.stage_focus_reveal_placement(reveal);
            }
            admitted.session.install_committed_motion(motion);
            UiPortalDismissalPublicationOutcome::Published(
                UiPortalDismissalPublicationReceipt::new(mounted, focus),
            )
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(mounted) => {
            UiPortalDismissalPublicationOutcome::InFlight(UiPortalDismissalPublicationCompletion {
                state: Some(Box::new(UiPortalDismissalInFlight { admitted, mounted })),
            })
        }
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
            let proposal = admitted
                .session
                .application
                .settle_indeterminate_portal_service_proposal(
                    admitted
                        .proposal
                        .take()
                        .expect("indeterminate dismissal retains proposal"),
                    admitted
                        .session
                        .portal
                        .as_mut()
                        .expect("admitted dismissal retains Portal installation"),
                    admitted
                        .session
                        .focus
                        .as_mut()
                        .expect("admitted dismissal retains Focus installation"),
                    admitted
                        .session
                        .motion
                        .as_mut()
                        .expect("admitted dismissal retains Motion installation"),
                )
                .expect("indeterminate dismissal retains exact service proposal");
            UiPortalDismissalPublicationOutcome::Indeterminate(
                UiPortalDismissalPublicationRecovery {
                    state: Some(Box::new(UiPortalDismissalIndeterminate {
                        session: admitted.session,
                        frame,
                        proposal,
                    })),
                },
            )
        }
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(_) => {
            settle_rejected(&mut admitted);
            UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::HostRejectedBeforeEffects,
            )
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(_) => {
            settle_rejected(&mut admitted);
            UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::MountedRetention,
            )
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(_) => {
            settle_rejected(&mut admitted);
            UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::MountedPresentation,
            )
        }
        crate::mounting::UiMountedFrameOutcome::Superseded(_) => {
            cancel(admitted);
            UiPortalDismissalPublicationOutcome::Stopped(
                UiPortalDismissalPublicationStop::Superseded,
            )
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(_) => {
            panic!("exact dismissal completion authority became unknown")
        }
        crate::mounting::UiMountedFrameOutcome::Unchanged(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_) => {
            unreachable!("portal dismissal always changes mounted overlay work")
        }
    }
}

fn settle_rejected(admitted: &mut UiPortalDismissalAdmitted<'_>) {
    let proposal = admitted
        .proposal
        .take()
        .expect("rejected dismissal retains proposal");
    admitted
        .session
        .application
        .settle_rejected_portal_service_proposal(
            proposal,
            admitted
                .session
                .focus
                .as_mut()
                .expect("admitted dismissal retains Focus installation"),
            admitted
                .session
                .motion
                .as_mut()
                .expect("admitted dismissal retains Motion installation"),
        )
        .expect("before-effect dismissal rejection retains exact proposal");
}

fn cancel(mut admitted: UiPortalDismissalAdmitted<'_>) {
    admitted.session.application.cancel_portal_service_proposal(
        admitted
            .proposal
            .take()
            .expect("cancelled dismissal retains proposal"),
        admitted
            .session
            .focus
            .as_mut()
            .expect("admitted dismissal retains Focus installation"),
        admitted
            .session
            .motion
            .as_mut()
            .expect("admitted dismissal retains Motion installation"),
    );
}
