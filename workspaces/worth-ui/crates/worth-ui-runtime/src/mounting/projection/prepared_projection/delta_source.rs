use super::super::frame_storage::UiMountedProjectionFrame;

#[derive(Clone, Copy)]
pub(crate) struct UiMountedPresentationDeltaSource<'a> {
    pub(super) frame: &'a UiMountedProjectionFrame,
    pub(super) predecessor: Option<worth_ui_host_contract::UiMountedFrameIdentity>,
    pub(super) changed_instances: &'a [worth_ui_host_contract::UiMountedInstanceIdentity],
    pub(super) node_changed_instances: &'a [worth_ui_host_contract::UiMountedInstanceIdentity],
    pub(super) changes: &'a super::super::super::UiMountedProjectionChangeSnapshot,
}

impl UiMountedPresentationDeltaSource<'_> {
    pub(in crate::mounting) const fn frame(&self) -> &UiMountedProjectionFrame {
        self.frame
    }

    pub(in crate::mounting) const fn predecessor(
        &self,
    ) -> Option<worth_ui_host_contract::UiMountedFrameIdentity> {
        self.predecessor
    }

    pub(in crate::mounting) const fn changed_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.changed_instances
    }

    pub(in crate::mounting) const fn node_changed_instances(
        &self,
    ) -> &[worth_ui_host_contract::UiMountedInstanceIdentity] {
        self.node_changed_instances
    }

    pub(in crate::mounting) fn surface_changed(
        &self,
        surface: worth_ui_host_contract::UiSemanticSurfaceIdentity,
    ) -> bool {
        self.changes.affects_surface(surface)
    }
}
