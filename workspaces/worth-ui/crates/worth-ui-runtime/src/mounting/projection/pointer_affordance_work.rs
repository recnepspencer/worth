use super::UiMountedPointerAffordanceWork;

impl UiMountedPointerAffordanceWork {
    pub const fn observations_examined(self) -> usize {
        self.observations_examined
    }

    pub const fn targets_examined(self) -> usize {
        self.targets_examined
    }

    pub const fn membership_key_probes(self) -> usize {
        self.membership_key_probes
    }

    pub const fn surfaces_changed(self) -> usize {
        self.surfaces_changed
    }
}
