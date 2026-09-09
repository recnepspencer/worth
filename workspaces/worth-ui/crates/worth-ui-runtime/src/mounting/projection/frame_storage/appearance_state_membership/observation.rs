use super::{UiMountedAppearanceStateMembers, UiMountedAppearanceStateMembership};

impl UiMountedAppearanceStateMembers {
    pub(in crate::mounting::projection::frame_storage) fn matches_geometry_input(
        &self,
        instance: worth_ui_host_contract::UiMountedInstanceIdentity,
        input: &crate::mounting::UiMountedAppearanceGeometryInput,
    ) -> (bool, usize) {
        let (key, probes) = self.reverse.get_with_probes(&instance);
        let Some(key) = key else {
            return (false, probes);
        };
        let (membership, member_probes) = self.primary.get_with_probes(key);
        let matches = match membership {
            Some(UiMountedAppearanceStateMembership::Retained(entry)) => {
                entry.sidecar.matches_geometry_input(input)
            }
            Some(UiMountedAppearanceStateMembership::PhysicalOnly(previous)) => {
                previous.sidecar.matches_geometry_input(input)
            }
            _ => false,
        };
        (matches, probes + member_probes)
    }

    pub(in crate::mounting::projection::frame_storage) fn has_current_retained(
        &self,
        key: &super::UiMountedAppearanceStateKey,
    ) -> (bool, super::UiMountedAppearanceMembershipWork) {
        let (membership, probes) = self.primary.get_with_probes(key.local_node());
        let mut work = super::UiMountedAppearanceMembershipWork::default();
        work.add_lookup(probes);
        let current = matches!(membership,
            Some(UiMountedAppearanceStateMembership::Retained(entry)) if entry.key == *key);
        (current, work)
    }
}
