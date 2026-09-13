use super::{UiMountedProjectionChangeSnapshot, UiMountedProjectionChanges};
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiSemanticSurfaceIdentity};

impl UiMountedProjectionChangeSnapshot {
    pub(crate) fn for_surfaces(
        self,
        surfaces: &[UiSemanticSurfaceIdentity],
        all_bound_surfaces: bool,
        instance_surface: impl Fn(UiMountedInstanceIdentity) -> Option<UiSemanticSurfaceIdentity>,
    ) -> (Self, usize) {
        if all_bound_surfaces {
            return (self, 0);
        }
        let mut applied = UiMountedProjectionChanges::default();
        let mut work = 0;
        for instance in self.observed.changed_instances.iter().copied() {
            work += 1;
            if instance_surface(instance).is_some_and(|surface| surfaces.contains(&surface)) {
                applied.mark_changed_instance(instance);
            }
        }
        for instance in self
            .observed
            .appearance_input_changed_instances
            .iter()
            .copied()
        {
            work += 1;
            if instance_surface(instance).is_some_and(|surface| surfaces.contains(&surface)) {
                applied.mark_appearance_input_changed(instance);
            }
        }
        for (instance, surface) in self.observed.retired_instances.iter() {
            work += 1;
            if surfaces.contains(surface) {
                applied.mark_retired_instance(*instance, *surface);
            }
        }
        for surface in self.observed.changed_surfaces.iter().copied() {
            work += 1;
            if surfaces.contains(&surface) {
                applied.mark_changed_surface(surface);
            }
        }
        for surface in self.observed.removed_surfaces.iter().copied() {
            work += 1;
            if surfaces.contains(&surface) {
                applied.mark_removed_surface(surface);
            }
        }
        applied.order_changed = self.observed.order_changed;
        applied.coalesced = self.observed.coalesced;
        applied.overflowed = self.observed.overflowed;
        let mut remainder = self.observed.clone();
        remainder.apply(&applied);
        // A local frame applies its own order, but cannot consume another
        // surface's ordering transition from this global journal.
        remainder.order_changed = self.observed.order_changed;
        (
            Self {
                applied,
                remainder,
                ..self
            },
            work,
        )
    }
}
