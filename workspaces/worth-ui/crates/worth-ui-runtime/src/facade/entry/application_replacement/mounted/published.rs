use super::{
    WorthUiActiveApplicationSession, WorthUiMountedApplicationReplacementOutcome,
    WorthUiPreparedApplicationActivation,
};

pub(super) struct WorthUiPresentedApplicationReplacement<'session> {
    session: &'session mut WorthUiActiveApplicationSession,
    application: Box<WorthUiPreparedApplicationActivation>,
    lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
    owners: super::super::owner_succession::UiPreparedApplicationOwnerSuccession,
    mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
    mounted_receipt: crate::mounting::UiMountedFramePublicationReceipt,
    scroll: super::super::scroll_replacement::UiPreparedScrollReplacement,
}

impl<'session> WorthUiPresentedApplicationReplacement<'session> {
    pub(super) fn new(
        session: &'session mut WorthUiActiveApplicationSession,
        application: Box<WorthUiPreparedApplicationActivation>,
        mut lifecycle: super::super::portal_lifecycle::WorthUiPreparedApplicationLifecycle,
        owners: super::super::owner_succession::UiPreparedApplicationOwnerSuccession,
        mounted_successor: crate::mounting::UiMountedGraphReplacementSuccessor,
        mounted_receipt: crate::mounting::UiMountedFramePublicationReceipt,
    ) -> Self {
        let staged_scroll = lifecycle.take_staged_scroll();
        let scroll = session.prepare_scroll_replacement(
            &application,
            &mounted_successor,
            Some(&mounted_receipt),
            staged_scroll,
        );
        Self {
            session,
            application,
            lifecycle,
            owners,
            mounted_successor,
            mounted_receipt,
            scroll,
        }
    }

    pub(super) fn commit_once(mut self) -> WorthUiMountedApplicationReplacementOutcome<'session> {
        self.session
            .overlay_composition_owners
            .commit(self.mounted_receipt.attempt());
        let focus = self.owners.focus.take();
        let application = self.session.commit_application_activation(
            self.application,
            self.mounted_successor,
            self.lifecycle,
            self.scroll,
            super::super::owner_succession::UiApplicationOwnerCutover::Mounted(self.owners),
        );
        self.session
            .host_exchange
            .record_presented_frame(self.mounted_receipt.frame());
        if let Some(publication) = self
            .session
            .mounted
            .current_text_publication_for_frame(self.mounted_receipt.frame())
        {
            self.session.presentation.settle_published_text(publication);
        }
        if let Some(focus) = focus {
            self.session
                .reconcile_prepared_focus_after_published_frame(focus, &self.mounted_receipt);
        }
        if let Some(batch) = self
            .session
            .mounted
            .take_replacement_appearance_attempt(self.mounted_receipt.attempt())
        {
            let (invalidation, records) = batch.into_parts();
            self.session
                .appearance_inspection
                .record_frame_attempts(records);
            if let Some(invalidation) = invalidation {
                self.session
                    .presentation
                    .settle_appearance_invalidation(&invalidation);
            }
        }
        WorthUiMountedApplicationReplacementOutcome::Published {
            application,
            mounted: self.mounted_receipt,
        }
    }
}
