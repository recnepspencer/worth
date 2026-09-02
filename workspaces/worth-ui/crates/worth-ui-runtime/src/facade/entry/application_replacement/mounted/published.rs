use super::{
    WorthUiActiveApplicationSession, WorthUiMountedApplicationReplacementOutcome,
    WorthUiPreparedApplicationActivation,
};

pub(super) struct WorthUiPresentedApplicationReplacement<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    application: Box<WorthUiPreparedApplicationActivation>,
    lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
    mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
    mounted_receipt: crate::mounting::UiMountedFramePublicationReceipt,
    focus: Option<crate::runtime::focus::UiPreparedFocusMountedReconciliation>,
    scroll: super::super::scroll_replacement::UiPreparedScrollReplacement,
    selection: super::super::selection_replacement::UiPreparedSelectionReplacement,
}

impl<'session> WorthUiPresentedApplicationReplacement<'session> {
    pub(super) fn new(
        session: &'session mut WorthUiActiveApplicationSession,
        application: Box<WorthUiPreparedApplicationActivation>,
        lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
        mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
        mounted_receipt: crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Self {
        let focus = application
            .candidate_service_policy_plan()
            .focus()
            .map(|_| {
                let snapshot = mounted_successor
                    .focus_participation_snapshot()
                    .expect("a presented mounted successor retains Focus participation");
                session
                    .focus
                    .as_ref()
                    .map_or_else(
                        || {
                            crate::runtime::focus::UiFocusRuntimeState::new_session_restore_candidate()
                                .prepare_mounted_reconciliation(&snapshot)
                        },
                        |focus| focus.prepare_mounted_reconciliation(&snapshot),
                    )
                    .expect("mounted Focus participation fits bounded counters")
            });
        let scroll = session.prepare_scroll_replacement(
            &application,
            &mounted_successor,
            Some(&mounted_receipt),
        );
        let selection =
            session.prepare_selection_replacement(&application, &mounted_successor, true);
        Self {
            session,
            application,
            lifecycle,
            mounted_successor,
            mounted_receipt,
            focus,
            scroll,
            selection,
        }
    }

    pub(super) fn commit_once(self) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        let application = self.session.commit_application_activation(
            self.application,
            self.mounted_successor,
            self.lifecycle,
            self.scroll,
            self.selection,
        );
        if let Some(focus) = self.focus {
            self.session
                .reconcile_prepared_focus_after_published_frame(focus, &self.mounted_receipt);
        }
        WorthUiMountedApplicationReplacementOutcome::Published {
            application,
            mounted: self.mounted_receipt,
        }
    }
}
