use super::WorthUiActiveApplicationSession;

impl WorthUiActiveApplicationSession {
    /// Commits an evidence-only successor authority and every owner its
    /// generation carries. The expression owner follows last, once the owners
    /// it reads hold the successor generation. Both a rebind and an authored
    /// successor publication commit through here.
    pub(in crate::facade::entry) fn commit_evidence_only_successor(
        &mut self,
        authority: crate::facade::prepared_application_authority::WorthUiPreparedApplicationAuthority,
        appearance_succession: super::super::UiPreparedAppearanceGenerationSuccession,
        owners: crate::runtime::appearance::UiPreparedRetainedAppearanceOwnerSuccession,
        overlay_bindings: crate::runtime::portal::UiPortalOverlayBindingLifecycle,
        pointer_succession:
            crate::runtime::pointer_affordance::UiPreparedPointerAffordanceGenerationSuccession,
        occurrence_geometry: crate::mounting::UiMountedOccurrenceGeometryState,
    ) -> (
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
        crate::facade::prepared_application_authority::WorthUiPreparedApplicationGenerationIdentity,
    ) {
        let generations = self.application.commit_evidence_only_rebind(authority);
        self.commit_retained_appearance_succession(appearance_succession, owners);
        self.authored_overlay_bindings = overlay_bindings;
        self.pointer_affordance_snapshot = pointer_succession.into_snapshot();
        self.mounted
            .commit_retained_geometry_succession(occurrence_geometry);
        self.follow_application_generation();
        generations
    }
}
