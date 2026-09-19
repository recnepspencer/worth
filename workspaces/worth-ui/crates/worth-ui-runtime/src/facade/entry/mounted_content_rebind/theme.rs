use super::{
    WorthUiActiveApplicationSession, WorthUiMountedContentPublication,
    WorthUiPreparedMountedContentRebind,
};
use crate::runtime::appearance::UiThemeSwitchChange;
use crate::runtime::rebind::UiRebindPreparationDenial;

impl WorthUiActiveApplicationSession {
    pub(in crate::facade::entry) fn prepare_theme_content_frame(
        &mut self,
        content: crate::mounting::UiMountedSemanticContentInput,
        theme: &mut UiThemeSwitchChange,
        reconciliation: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<crate::mounting::UiPreparedMountedFrame, UiRebindPreparationDenial> {
        self.presentation
            .appearance_theme_state()
            .ok_or(UiRebindPreparationDenial::ThemeSwitch(
                crate::runtime::appearance::UiThemeSwitchDenial::MissingActiveBinding,
            ))?
            .validate_prepared_switch(theme.prepared())
            .map_err(UiRebindPreparationDenial::ThemeSwitch)?;
        theme
            .refresh(
                self.application.prepared_authority(),
                &self.mounted,
                self.graph(),
            )
            .map_err(|_| UiRebindPreparationDenial::CandidateBindingMismatch)?;
        let request = self
            .mounted_frame_request()
            .for_surfaces(vec![theme.prepared().successor().surface()]);
        let presentation = (if reconciliation.is_empty() {
            self.presentation.project()
        } else {
            self.presentation.project_complete()
        })
        .map_err(|denial| UiRebindPreparationDenial::ContentMountedPreparation(Box::new(denial)))?;
        let completion = self
            .execute_framework_turn(|_| {})
            .map_err(|_| UiRebindPreparationDenial::FrameBoundaryUnavailable)?;
        let mut execution = completion
            .into_execution()
            .map_err(|_| UiRebindPreparationDenial::FrameBoundaryUnavailable)?;
        execution
            .prepare_mounted_theme_content(request, content, presentation, theme, reconciliation)
            .map_err(|denial| {
                UiRebindPreparationDenial::ContentMountedPreparation(Box::new(denial))
            })
    }
}

impl<'session> WorthUiPreparedMountedContentRebind<'session> {
    pub(in crate::facade::entry) fn prepare_theme(
        session: &'session mut WorthUiActiveApplicationSession,
        content: crate::mounting::UiMountedSemanticContentInput,
        mut theme: UiThemeSwitchChange,
        reconciliation: &[crate::mounting::UiMountedSurfaceReconciliationBinding],
    ) -> Result<Self, UiRebindPreparationDenial> {
        let frame = session.prepare_theme_content_frame(content, &mut theme, reconciliation)?;
        Ok(Self {
            session,
            frame,
            publication: WorthUiMountedContentPublication::ThemeSwitch {
                theme,
                reconciliation: reconciliation.to_vec().into_boxed_slice(),
            },
        })
    }
}
