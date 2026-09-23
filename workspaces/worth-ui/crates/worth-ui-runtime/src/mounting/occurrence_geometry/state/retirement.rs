use super::*;

impl UiMountedOccurrenceGeometryState {
    pub(crate) fn retire_instance(
        &mut self,
        instance: UiMountedInstanceIdentity,
    ) -> Box<[UiMountedInstanceIdentity]> {
        let mut affected = BTreeSet::new();
        for surface in self.surfaces.values_mut() {
            if !surface.occurrences.contains_key(&instance) {
                continue;
            }
            let mut pending = vec![instance];
            while let Some(candidate) = pending.pop() {
                let Some(row) = surface.occurrences.remove(&candidate) else {
                    continue;
                };
                affected.insert(candidate);
                surface.scroll_poses.remove(&candidate);
                if let Some(parent) = row
                    .parent
                    .and_then(|parent| surface.children.get_mut(&parent))
                {
                    parent.retain(|child| *child != candidate);
                }
                if let Some(descendants) = surface.children.remove(&candidate) {
                    pending.extend(descendants);
                }
            }
            surface.regions.retain(|_, rows| {
                rows.retain(|(owner, _, _)| surface.occurrences.contains_key(owner));
                !rows.is_empty()
            });
            surface.scroll_index =
                std::sync::Arc::new(scroll_index::UiScrollGeometryIndex::build(surface));
        }
        affected.into_iter().collect::<Vec<_>>().into_boxed_slice()
    }

    pub(crate) fn retire_surface(&mut self, surface: UiSemanticSurfaceIdentity) {
        self.surfaces.remove(&surface);
    }
}
