use worth_ui_host_contract::UiPresentationDeadline;

use super::{
    WorthUiMountedPreviewAdmissionRejection, WorthUiMountedPreviewCompletionRejection,
    WorthUiMountedPreviewDisposition, WorthUiMountedPreviewInFlight, WorthUiMountedPreviewOutcome,
    WorthUiMountedPreviewPorts, WorthUiMountedPreviewRetentionRejection,
    WorthUiPreparedMountedPreview, WorthUiResolvedMountedPreview,
};

impl<'session> WorthUiPreparedMountedPreview<'session> {
    pub fn frame(&self) -> &crate::mounting::UiPreparedMountedFrame {
        &self.frame
    }

    pub fn present(
        self,
        deadline: UiPresentationDeadline,
        now: u64,
    ) -> WorthUiMountedPreviewOutcome<'session> {
        let Self {
            frame,
            transition,
            planning_counters,
            mut ports,
        } = self;
        let before = transition.preview().capture_isolation_basis();
        let publication =
            ports
                .mounted
                .present_prepared_frame(ports.host_session, frame, None, deadline, now);
        let outcome = super::super::mounted_publication::finish_mounted_transition(
            ports.mounted,
            ports.focus.as_deref_mut(),
            ports.portal.as_deref_mut(),
            ports.interaction,
            ports.host_session,
            ports.application_session_identity,
            &ports.generation_identity,
            ports.host_exchange,
            publication,
            None,
            None,
            None,
        );
        finish_preview_outcome(outcome, before, transition, planning_counters, ports)
    }

    pub fn supersede(self) -> WorthUiResolvedMountedPreview {
        resolve_transition(
            WorthUiMountedPreviewDisposition::Superseded,
            self.transition,
            self.planning_counters,
        )
    }
}

impl<'session> WorthUiMountedPreviewInFlight<'session> {
    pub fn attempt(&self) -> worth_ui_host_contract::UiMountedPresentationAttemptIdentity {
        self.handle.attempt()
    }

    pub fn deadline(&self) -> worth_ui_host_contract::UiPresentationDeadline {
        self.handle.deadline()
    }

    pub fn pending_bindings(
        &self,
    ) -> impl ExactSizeIterator<Item = worth_ui_host_contract::UiSurfaceBindingGeneration> + '_
    {
        self.handle.pending_bindings()
    }

    pub fn cost_report(&self) -> crate::mounting::UiMountCostReport {
        self.handle.cost_report()
    }

    pub fn complete(self: Box<Self>, now: u64) -> WorthUiMountedPreviewOutcome<'session> {
        let Self {
            handle,
            before,
            transition,
            planning_counters,
            mut ports,
        } = *self;
        let publication =
            ports
                .mounted
                .complete_presentation(ports.host_session, handle.clone(), now);
        let outcome = super::super::mounted_publication::finish_mounted_transition(
            ports.mounted,
            ports.focus.as_deref_mut(),
            ports.portal.as_deref_mut(),
            ports.interaction,
            ports.host_session,
            ports.application_session_identity,
            &ports.generation_identity,
            ports.host_exchange,
            publication,
            None,
            None,
            None,
        );
        if let crate::mounting::UiMountedFrameOutcome::CompletionDenied(denial) = outcome {
            return WorthUiMountedPreviewOutcome::CompletionDenied(Box::new(
                WorthUiMountedPreviewCompletionRejection {
                    denial,
                    in_flight: WorthUiMountedPreviewInFlight {
                        handle,
                        before,
                        transition,
                        planning_counters,
                        ports,
                    },
                },
            ));
        }
        finish_preview_outcome(outcome, before, transition, planning_counters, ports)
    }
}

fn finish_preview_outcome<'session>(
    outcome: crate::mounting::UiMountedFrameOutcome,
    before: crate::runtime::UiAllocationTruthRevision,
    transition: crate::runtime::UiPendingMountedPreviewTransition<'session>,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
    ports: WorthUiMountedPreviewPorts<'session>,
) -> WorthUiMountedPreviewOutcome<'session> {
    match outcome {
        crate::mounting::UiMountedFrameOutcome::Published(receipt) => resolved(
            WorthUiMountedPreviewDisposition::Published(receipt),
            transition,
            before,
            planning_counters,
        ),
        crate::mounting::UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => resolved(
            WorthUiMountedPreviewDisposition::RejectedBeforeEffects(rejected),
            transition,
            before,
            planning_counters,
        ),
        crate::mounting::UiMountedFrameOutcome::PresentationIndeterminate(indeterminate) => {
            resolved(
                WorthUiMountedPreviewDisposition::PresentationIndeterminate(indeterminate),
                transition,
                before,
                planning_counters,
            )
        }
        crate::mounting::UiMountedFrameOutcome::InFlight(handle) => {
            WorthUiMountedPreviewOutcome::InFlight(Box::new(WorthUiMountedPreviewInFlight {
                handle,
                before,
                transition,
                planning_counters,
                ports,
            }))
        }
        crate::mounting::UiMountedFrameOutcome::AdmissionDenied(rejection) => {
            WorthUiMountedPreviewOutcome::AdmissionDenied(Box::new(
                WorthUiMountedPreviewAdmissionRejection {
                    denial: rejection.denial(),
                    preview: WorthUiPreparedMountedPreview {
                        frame: rejection.into_frame(),
                        transition,
                        planning_counters,
                        ports,
                    },
                },
            ))
        }
        crate::mounting::UiMountedFrameOutcome::RetentionDenied(rejection) => {
            WorthUiMountedPreviewOutcome::RetentionDenied(Box::new(
                WorthUiMountedPreviewRetentionRejection {
                    denial: rejection.denial(),
                    preview: WorthUiPreparedMountedPreview {
                        frame: rejection.into_frame(),
                        transition,
                        planning_counters,
                        ports,
                    },
                },
            ))
        }
        crate::mounting::UiMountedFrameOutcome::CompletionDenied(_)
        | crate::mounting::UiMountedFrameOutcome::Superseded(_)
        | crate::mounting::UiMountedFrameOutcome::Reconciled(_)
        | crate::mounting::UiMountedFrameOutcome::Unchanged(_) => {
            unreachable!("preview publication only yields preview lifecycle outcomes")
        }
    }
}

fn resolved<'session>(
    disposition: WorthUiMountedPreviewDisposition,
    transition: crate::runtime::UiPendingMountedPreviewTransition<'session>,
    before: crate::runtime::UiAllocationTruthRevision,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
) -> WorthUiMountedPreviewOutcome<'session> {
    WorthUiMountedPreviewOutcome::Resolved(Box::new(resolve_transition_from(
        disposition,
        transition,
        before,
        planning_counters,
    )))
}

pub(super) fn resolve_transition(
    disposition: WorthUiMountedPreviewDisposition,
    transition: crate::runtime::UiPendingMountedPreviewTransition<'_>,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
) -> WorthUiResolvedMountedPreview {
    let before = transition.preview().capture_isolation_basis();
    resolve_transition_from(disposition, transition, before, planning_counters)
}

fn resolve_transition_from(
    disposition: WorthUiMountedPreviewDisposition,
    transition: crate::runtime::UiPendingMountedPreviewTransition<'_>,
    before: crate::runtime::UiAllocationTruthRevision,
    planning_counters: crate::runtime::UiFrameworkTransitionPlanningCounters,
) -> WorthUiResolvedMountedPreview {
    let resolved = transition.finish(before);
    WorthUiResolvedMountedPreview {
        disposition,
        isolation: resolved.isolation,
        follow_on: resolved.follow_on,
        planning_counters,
    }
}
