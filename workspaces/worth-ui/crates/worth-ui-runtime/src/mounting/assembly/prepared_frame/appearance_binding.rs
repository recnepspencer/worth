use super::UiPreparedMountedFrame;

impl UiPreparedMountedFrame {
    pub(in crate::mounting) fn appearance_raw_opacity_for_instance(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    ) -> Option<worth_ui_host_contract::UiMountedAppearanceOpacity> {
        self.candidate
            .owner
            .appearance_raw_opacity_for_instance(instance)
    }

    pub(in crate::mounting) fn appearance_changed_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.candidate.owner.appearance_changed_instances()
    }

    pub(in crate::mounting) fn retired_appearance_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.candidate.owner.retired_appearance_instances()
    }

    pub(crate) fn prepare_appearance_reconstruction(
        &mut self,
    ) -> Result<(), crate::mounting::projection::UiMountedProjectionDenial> {
        self.candidate
            .prepare_appearance_reconstruction_for(self.manifest.surfaces())
    }
}
