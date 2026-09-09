use super::{
    appearance_state_membership, UiMountedAppearanceFrameState, UiMountedAppearanceStateEntry,
};

impl UiMountedAppearanceFrameState {
    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn has_active_portal_for_test(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> bool {
        self.active_portal_instances
            .get(&surface)
            .is_some_and(|instances| instances.contains(&instance))
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn retain_overlay_sidecar_for_test(
        &mut self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) {
        self.overlay_sidecars.entry(surface).or_default();
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn has_overlay_sidecar_for_test(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> bool {
        self.overlay_sidecars.contains_key(&surface)
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn membership_counts(
        &self,
    ) -> (usize, usize, usize) {
        self.members.membership_counts()
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn membership_roots_shared_with(
        &self,
        other: &Self,
    ) -> bool {
        self.members.roots_shared_with(&other.members)
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn membership_work(
        &self,
    ) -> (usize, usize, usize) {
        let report = self.selection_cost_report();
        (
            report.membership_key_probes(),
            report.membership_copied_avl_nodes(),
            report.membership_traversed_entries(),
        )
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn retained_entry_for_test(
        &self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
    ) -> Option<&UiMountedAppearanceStateEntry> {
        let key = appearance_state_membership::state_key(context);
        self.members.retained_entry(&key)
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn retain_projection_for_test(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
        projection: crate::runtime::appearance::UiAppearanceProjection,
    ) {
        let key = appearance_state_membership::state_key(context);
        let (result, work) = self.members.insert_retained(UiMountedAppearanceStateEntry {
            key,
            context: context.clone(),
            projection,
            sidecar: Default::default(),
        });
        self.selection.record_membership_work(work);
        result.expect("test retention uses one exact mounted identity");
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn replace_sidecar_for_test(
        &mut self,
        context: &crate::runtime::appearance::UiAppearanceAttemptContext,
        sidecar: super::super::super::appearance::UiMountedAppearanceSidecar,
    ) {
        let key = appearance_state_membership::state_key(context);
        let entry = self
            .members
            .retained_entry(&key)
            .expect("test state entry should exist before sidecar replacement");
        let mut replacement = entry.clone();
        replacement.sidecar = sidecar;
        let (result, work) = self.members.insert_retained(replacement);
        self.selection.record_membership_work(work);
        result.expect("test sidecar replacement uses one exact mounted identity");
    }
}
