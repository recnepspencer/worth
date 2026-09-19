use super::{
    UiMountedProjectionDenial, UiMountedQualifiedSemanticText, UiMountedSemanticMechanicRows,
    UiMountedSemanticMechanicSource,
};

impl UiMountedSemanticMechanicSource {
    pub(crate) fn qualified_layout_count(&self) -> usize {
        self.by_layout.len()
    }

    #[cfg(any(test, feature = "certification-support"))]
    pub(crate) fn require_layout_reconstruction(
        &mut self,
    ) -> Result<usize, UiMountedProjectionDenial> {
        let instances = self
            .by_instance
            .iter()
            .map(|(instance, _)| *instance)
            .collect::<Vec<_>>();
        let mut lost = 0_usize;
        for instance in instances {
            let mut rows = self
                .by_instance
                .get(&instance)
                .cloned()
                .expect("collected semantic mechanic instance remains present");
            lost = lost
                .checked_add(rows.require_layout_reconstruction()?)
                .ok_or(UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
            self.by_instance.insert(instance, rows);
        }
        self.rebuild_layout_index();
        Ok(lost)
    }

    pub(crate) fn reconstruct_layouts(&mut self) -> Result<usize, UiMountedProjectionDenial> {
        let instances = self
            .by_instance
            .iter()
            .map(|(instance, _)| *instance)
            .collect::<Vec<_>>();
        let mut reconstructed = 0_usize;
        for instance in instances {
            let mut rows = self
                .by_instance
                .get(&instance)
                .cloned()
                .expect("collected semantic mechanic instance remains present");
            reconstructed = reconstructed
                .checked_add(rows.reconstruct_layouts()?)
                .ok_or(UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
            self.by_instance.insert(instance, rows);
        }
        self.rebuild_layout_index();
        Ok(reconstructed)
    }

    pub(crate) fn layout_reconstruction_required(&self) -> bool {
        self.by_instance
            .iter()
            .any(|(_, rows)| rows.iter().any(|row| row.layout_reconstruction_required()))
    }
}

impl UiMountedSemanticMechanicRows {
    #[cfg(any(test, feature = "certification-support"))]
    fn require_layout_reconstruction(&mut self) -> Result<usize, UiMountedProjectionDenial> {
        self.update_layout_state(UiMountedQualifiedSemanticText::require_layout_reconstruction)
    }

    fn reconstruct_layouts(&mut self) -> Result<usize, UiMountedProjectionDenial> {
        self.update_layout_state(UiMountedQualifiedSemanticText::reconstruct_layout)
    }

    fn update_layout_state(
        &mut self,
        update: fn(&mut UiMountedQualifiedSemanticText) -> Result<bool, UiMountedProjectionDenial>,
    ) -> Result<usize, UiMountedProjectionDenial> {
        let keys = self.order.iter().copied().collect::<Vec<_>>();
        let mut updated = 0_usize;
        for key in keys {
            let mut row = self
                .rows
                .get(&key)
                .cloned()
                .expect("semantic mechanic order names an indexed row");
            updated = updated
                .checked_add(usize::from(update(&mut row)?))
                .ok_or(UiMountedProjectionDenial::SemanticTextCapacityExceeded)?;
            self.rows.insert(key, row);
        }
        Ok(updated)
    }
}
