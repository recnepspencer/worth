use super::{UiMountedAppearanceStateMembers, UiMountedAppearanceStateMembership};

impl UiMountedAppearanceStateMembers {
    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn membership_counts(
        &self,
    ) -> (usize, usize, usize) {
        self.primary
            .iter()
            .fold((0, 0, 0), |counts, (_, membership)| {
                let (retained, reserved, staged) = counts;
                match membership {
                    UiMountedAppearanceStateMembership::Retained(_) => {
                        (retained + 1, reserved, staged)
                    }
                    UiMountedAppearanceStateMembership::Reserved => {
                        (retained, reserved + 1, staged)
                    }
                    UiMountedAppearanceStateMembership::PhysicalOnly(_) => counts,
                    UiMountedAppearanceStateMembership::Staged { .. } => {
                        (retained, reserved, staged + 1)
                    }
                }
            })
    }

    #[cfg(test)]
    pub(in crate::mounting::projection::frame_storage) fn roots_shared_with(
        &self,
        other: &Self,
    ) -> bool {
        self.primary.root_is_shared_with(&other.primary)
            && self.reverse.root_is_shared_with(&other.reverse)
    }
}
