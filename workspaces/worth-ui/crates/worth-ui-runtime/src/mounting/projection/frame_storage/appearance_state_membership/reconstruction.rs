use super::*;

impl UiMountedAppearanceStateMembers {
    pub(in crate::mounting::projection::frame_storage) fn keys_for_reconstruction(
        &self,
    ) -> (
        Vec<UiMountedAppearanceLocalNodeKey>,
        UiMountedAppearanceMembershipWork,
    ) {
        let mut work = UiMountedAppearanceMembershipWork::default();
        work.add_traversal(self.primary.len());
        let mut keys = self
            .primary
            .iter()
            .filter_map(|(local_key, membership)| match membership {
                UiMountedAppearanceStateMembership::Retained(_)
                | UiMountedAppearanceStateMembership::PhysicalOnly(_) => Some(local_key.clone()),
                UiMountedAppearanceStateMembership::Reserved
                | UiMountedAppearanceStateMembership::Staged { .. } => None,
            })
            .collect::<Vec<_>>();
        keys.sort();
        (keys, work)
    }

    pub(in crate::mounting::projection::frame_storage) fn physical_only_keys_for_reconstruction(
        &self,
    ) -> (
        Vec<UiMountedAppearanceLocalNodeKey>,
        UiMountedAppearanceMembershipWork,
    ) {
        let mut work = UiMountedAppearanceMembershipWork::default();
        work.add_traversal(self.primary.len());
        let keys = self
            .primary
            .iter()
            .filter_map(|(key, membership)| {
                matches!(
                    membership,
                    UiMountedAppearanceStateMembership::PhysicalOnly(_)
                )
                .then_some(key.clone())
            })
            .collect();
        (keys, work)
    }
}
