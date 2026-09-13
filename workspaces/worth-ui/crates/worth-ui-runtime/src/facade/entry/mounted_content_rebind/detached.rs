use super::*;

impl WorthUiDetachedPreparedMountedContentRebind {
    pub(crate) fn is_theme_switch(&self) -> bool {
        matches!(
            self.publication,
            WorthUiMountedContentPublication::ThemeSwitch { .. }
        )
    }

    pub(crate) fn with_theme_reconciliation(
        mut self,
        replacements: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<Self, crate::runtime::rebind::UiRebindPreparationDenial> {
        let WorthUiMountedContentPublication::ThemeSwitch { reconciliation, .. } =
            &mut self.publication
        else {
            return Err(
                crate::runtime::rebind::UiRebindPreparationDenial::CandidateBindingMismatch,
            );
        };
        *reconciliation = replacements.to_vec().into_boxed_slice();
        Ok(self)
    }

    pub(crate) const fn session_identity(
        &self,
    ) -> crate::facade::WorthUiActiveApplicationSessionIdentity {
        self.session_identity
    }

    pub(crate) fn rebase<'session>(
        mut self,
        session: &'session mut WorthUiActiveApplicationSession,
        semantic_content: crate::mounting::UiMountedSemanticContentInput,
    ) -> Result<
        Box<WorthUiPreparedMountedContentRebind<'session>>,
        crate::runtime::rebind::UiRebindPreparationDenial,
    > {
        if let WorthUiMountedContentPublication::AuthoredSuccessor {
            authority,
            overlay_bindings,
            occurrence_geometry,
            pointer_succession,
            appearance_succession,
            owners,
            ..
        } = &mut self.publication
        {
            *occurrence_geometry = session
                .application
                .prepare_retained_layout_succession(&session.mounted, authority, overlay_bindings)
                .map_err(
                    crate::runtime::rebind::UiRebindPreparationDenial::CandidateOccurrenceGeometry,
                )?;
            *pointer_succession = session.prepare_pointer_generation_succession(authority)?;
            *owners =
                session.prepare_retained_appearance_owners(authority, appearance_succession)?;
        }
        let frame = match &mut self.publication {
            WorthUiMountedContentPublication::AuthoredSuccessor {
                pointer_succession,
                occurrence_geometry,
                owners,
                ..
            } => session.prepare_authored_content_frame(
                semantic_content,
                session.mounted_frame_request(),
                pointer_succession,
                owners,
                occurrence_geometry,
            )?,
            WorthUiMountedContentPublication::ThemeSwitch {
                theme,
                reconciliation,
            } => session.prepare_theme_content_frame(semantic_content, theme, reconciliation)?,
            WorthUiMountedContentPublication::RetainedGeneration => session
                .prepare_content_rebind_frame(semantic_content, session.mounted_frame_request())?,
        };
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
