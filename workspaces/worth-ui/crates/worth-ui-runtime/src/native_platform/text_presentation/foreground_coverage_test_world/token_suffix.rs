//! Build mixed adopted/token paint from qualified source ranges.
use super::{Arc, CoverageWorld};
use worth_ui_host_contract::*;

impl CoverageWorld {
    pub(in crate::native_platform::text_presentation) fn with_token_suffix(
        mut self,
        adopted_bytes: u32,
    ) -> Self {
        let ranges = [
            UiTextOriginalRange::new(0, adopted_bytes).unwrap(),
            UiTextOriginalRange::new(adopted_bytes, self.layout.view().source().len() as u32)
                .unwrap(),
        ];
        self.layout =
            crate::mounting::qualified_text_test_support::UiQualifiedTextTestFixture::new()
                .layout_with_ranges(self.layout.view().source(), &ranges);
        let candidates = self
            .fragment
            .text_candidates()
            .iter()
            .map(|original| {
                let foregrounds = Arc::from([
                    UiMountedTextForegroundSpan::from_runtime_mounting(
                        UiTextOriginalRange::new(0, adopted_bytes).unwrap(),
                        UiMountedRgba8::new(255, 255, 255, 255),
                        self.foreground.paint_span(),
                    ),
                    UiMountedTextForegroundSpan::from_runtime_mounting(
                        UiTextOriginalRange::new(adopted_bytes, original.text().len() as u32)
                            .unwrap(),
                        UiMountedRgba8::new(100, 150, 200, 255),
                        UiMountedTextPaintSpanIdentity::from_runtime_mounting([18; 32]),
                    ),
                ]);
                UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
                    UiMountedSemanticTextCompletionInput {
                        content_generation: original.content_generation(),
                        frame: original.frame(),
                        surface: original.surface(),
                        binding: original.binding(),
                        mounted_instance: original.mounted_instance(),
                        node_receipt: original.node_receipt(),
                        allocation_basis: original.allocation_basis(),
                        bounds: original.bounds(),
                        clip_bounds: original.clip_bounds(),
                        origin_x: original.origin_x(),
                        origin_y: original.origin_y(),
                        text: original.retained_text_for_runtime_mounting(),
                        layout: self.layout.view(),
                        slot: original.slot(),
                        collection_row: original.collection_row().copied(),
                        foregrounds,
                        profile: original.profile(),
                        layer_semantic_order: original.layer_semantic_order(),
                        capability_generation: original.capability_generation(),
                        capability_profile_digest: original.capability_profile_digest(),
                    },
                )
                .unwrap()
            })
            .collect::<Vec<_>>();
        self.fragment = UiUnpublishedAppearanceFragment::from_runtime_mounting(
            self.fragment.identity(),
            self.fragment.work().clone(),
            candidates,
            self.fragment.surface_binding(),
            self.fragment.presentation_affinity(),
        )
        .unwrap();
        self
    }
}
