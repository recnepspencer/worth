use std::sync::Arc;

impl super::UiMountedSemanticTextMechanic {
    /// Frame and receipt advancement do not replace an unchanged physical command.
    /// The presentation owner separately admits the successor's current authority.
    #[doc(hidden)]
    pub fn same_retained_paint_meaning(&self, other: &Self) -> bool {
        self.schema == other.schema
            && self.surface == other.surface
            && self.binding == other.binding
            && self.mounted_instance == other.mounted_instance
            && self.allocation_basis == other.allocation_basis
            && self.bounds == other.bounds
            && self.clip_bounds == other.clip_bounds
            && self.origin_x == other.origin_x
            && self.origin_y == other.origin_y
            && self.text == other.text
            && self.layout_identity == other.layout_identity
            && self.layout_request == other.layout_request
            && self.layout_profile == other.layout_profile
            && self.layout_fonts == other.layout_fonts
            && self.layout_scale == other.layout_scale
            && self.layout_width == other.layout_width
            && self.slot == other.slot
            && self.collection_row == other.collection_row
            && self.foregrounds == other.foregrounds
            && self.profile == other.profile
            && self.layer_semantic_order == other.layer_semantic_order
            && self.capability_generation == other.capability_generation
            && self.capability_profile_digest == other.capability_profile_digest
    }

    /// Reconciliation may move one unchanged command onto the admitted replacement binding.
    #[doc(hidden)]
    pub fn same_retained_paint_meaning_after_binding_replacement(
        &self,
        other: &Self,
        affected: crate::UiSurfaceBindingGeneration,
        replacement: crate::UiSurfaceBindingGeneration,
    ) -> bool {
        if self.binding != affected || other.binding != replacement {
            return false;
        }
        let mut rebound = self.clone();
        rebound.binding = replacement;
        // The replacement binding admits a fresh coordinate-ownership proof.
        // Keep the allocation receipt lineage and transform exact while comparing
        // the successor under that newly admitted ownership.
        rebound.allocation_basis = crate::UiMountedAllocationBasis::new(
            self.allocation_basis.receipt_identity(),
            self.allocation_basis.receipt_generation(),
            other.allocation_basis.coordinate_ownership(),
            self.allocation_basis.transform(),
        );
        rebound.same_retained_paint_meaning(other)
    }

    #[doc(hidden)]
    pub fn retained_text_for_runtime_mounting(&self) -> Arc<str> {
        Arc::clone(&self.text)
    }

    #[doc(hidden)]
    pub fn retained_foregrounds_for_runtime_mounting(
        &self,
    ) -> Arc<[super::UiMountedTextForegroundSpan]> {
        Arc::clone(&self.foregrounds)
    }
}
