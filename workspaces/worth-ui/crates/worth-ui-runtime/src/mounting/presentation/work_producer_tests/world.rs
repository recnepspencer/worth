use worth_ui_host_contract::{
    UiHostSurfaceIdentity, UiHostSurfacePresentationMode, UiMountedAllocationBasis,
    UiMountedCanonicalBox, UiMountedCanonicalBoxInput, UiMountedClipTable,
    UiMountedContentGeneration, UiMountedCoordinateSpace, UiMountedFilledRectCompletionInput,
    UiMountedFilledRectMechanic, UiMountedFilledRectTable, UiMountedFrameIdentity,
    UiMountedHitTestTable, UiMountedInstanceIdentity, UiMountedLayerTable,
    UiMountedNodeReceiptIssuer, UiMountedPaintBatchTable, UiMountedProjectionView,
    UiMountedProjectionViewInput, UiMountedResourceTable, UiMountedRgba8,
    UiMountedSemanticTextTable, UiMountedSpatialBatchTable, UiMountedSurfaceBindingRequirement,
    UiMountedTransformProjection, UiSemanticSurfaceIdentity, UiSurfaceBindingGeneration,
    WorthUiHostCapabilityObservationGeneration,
};

use super::rect_node::rect_node;
use super::text_node::text_node;

pub(in crate::mounting::presentation) struct MountedPresentationWorld {
    surface: UiSemanticSurfaceIdentity,
    binding: UiSurfaceBindingGeneration,
    content: UiMountedContentGeneration,
    pub(in crate::mounting::presentation) first_instance: UiMountedInstanceIdentity,
    pub(in crate::mounting::presentation) second_instance: UiMountedInstanceIdentity,
    pub(in crate::mounting::presentation) requirement: UiMountedSurfaceBindingRequirement,
}

#[derive(Clone, Copy)]
pub(in crate::mounting::presentation) struct RectSpec {
    instance: UiMountedInstanceIdentity,
    pub(in crate::mounting::presentation) x: f32,
    pub(in crate::mounting::presentation) color: UiMountedRgba8,
    pub(in crate::mounting::presentation) clip_x: f32,
    clip_width: f32,
}

pub(in crate::mounting::presentation) fn rect_spec(
    instance: UiMountedInstanceIdentity,
    x: f32,
) -> RectSpec {
    RectSpec {
        instance,
        x,
        color: UiMountedRgba8::new(47, 129, 247, 255),
        clip_x: x,
        clip_width: 32.0,
    }
}

pub(in crate::mounting::presentation) fn rect_spec_with_clip(
    instance: UiMountedInstanceIdentity,
    x: f32,
    clip_x: f32,
    clip_width: f32,
) -> RectSpec {
    RectSpec {
        instance,
        x,
        color: UiMountedRgba8::new(47, 129, 247, 255),
        clip_x,
        clip_width,
    }
}

impl MountedPresentationWorld {
    pub(in crate::mounting::presentation) fn new() -> Self {
        let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
        let binding = UiSurfaceBindingGeneration::mint_unbound().unwrap();
        let generation = WorthUiHostCapabilityObservationGeneration::new(7);
        let requirement = UiMountedSurfaceBindingRequirement::new(
            surface,
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            binding,
            generation,
            11,
            UiHostSurfacePresentationMode::RecordOnly,
        );
        Self {
            surface,
            binding,
            content: UiMountedContentGeneration::mint_unbound().unwrap(),
            first_instance: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            second_instance: UiMountedInstanceIdentity::mint_unbound().unwrap(),
            requirement,
        }
    }

    pub(in crate::mounting::presentation) fn projection(
        &self,
        frame: UiMountedFrameIdentity,
        specs: impl IntoIterator<Item = RectSpec>,
    ) -> UiMountedProjectionView {
        let rows = specs
            .into_iter()
            .map(|spec| self.rect(frame, spec))
            .collect::<Vec<_>>();
        let nodes = rows
            .iter()
            .enumerate()
            .map(|(index, row)| rect_node(index, row))
            .collect::<Vec<_>>();
        let (authored_paint_commands, authored_paint_order) =
            crate::mounting::compile_presentation_sources(&nodes, &rows, &[], &[]);
        UiMountedProjectionView::new(UiMountedProjectionViewInput {
            frame,
            surface: self.surface,
            binding: self.binding,
            content_generation: self.content,
            nodes,
            clips: UiMountedClipTable::produced(Vec::new()),
            layers: UiMountedLayerTable::produced(Vec::new()),
            filled_rects: UiMountedFilledRectTable::from_runtime_mounting(rows).unwrap(),
            portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
            semantic_text: UiMountedSemanticTextTable::empty(),
            hit_tests: UiMountedHitTestTable::empty(),
            paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
            spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
            realtime_batches: worth_ui_host_contract::UiMountedRealtimeBatchTable::new(Vec::new()),
            resources: UiMountedResourceTable::new(Vec::new()),
            authored_paint_commands,
            authored_paint_order,
        })
    }

    pub(in crate::mounting::presentation) fn mixed_projection(
        &self,
        frame: UiMountedFrameIdentity,
        instances: &[UiMountedInstanceIdentity],
        changed_index: Option<usize>,
        text_layout: worth_ui_host_contract::UiQualifiedTextLayoutView<'_>,
    ) -> UiMountedProjectionView {
        let split = instances.len() / 2;
        let rects = instances[..split]
            .iter()
            .enumerate()
            .map(|(index, instance)| {
                let mut spec = rect_spec(*instance, index as f32 * 40.0);
                if changed_index == Some(index) {
                    spec.color = UiMountedRgba8::new(242, 204, 96, 255);
                }
                self.rect(frame, spec)
            })
            .collect::<Vec<_>>();
        let texts = instances[split..]
            .iter()
            .enumerate()
            .map(|(index, instance)| {
                self.text(
                    frame,
                    *instance,
                    index,
                    text_layout,
                    changed_index == Some(index + split),
                )
            })
            .collect::<Vec<_>>();
        let nodes = rects
            .iter()
            .enumerate()
            .map(|(index, row)| rect_node(index, row))
            .chain(
                texts
                    .iter()
                    .enumerate()
                    .map(|(index, row)| text_node(index, row)),
            )
            .collect::<Vec<_>>();
        let (authored_paint_commands, authored_paint_order) =
            crate::mounting::compile_presentation_sources(&nodes, &rects, &[], &texts);
        UiMountedProjectionView::new(UiMountedProjectionViewInput {
            frame,
            surface: self.surface,
            binding: self.binding,
            content_generation: self.content,
            nodes,
            clips: UiMountedClipTable::produced(Vec::new()),
            layers: UiMountedLayerTable::produced(Vec::new()),
            filled_rects: UiMountedFilledRectTable::from_runtime_mounting(rects).unwrap(),
            portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
            semantic_text: UiMountedSemanticTextTable::from_runtime_mounting(texts).unwrap(),
            hit_tests: UiMountedHitTestTable::empty(),
            paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
            spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
            realtime_batches: worth_ui_host_contract::UiMountedRealtimeBatchTable::new(Vec::new()),
            resources: UiMountedResourceTable::new(Vec::new()),
            authored_paint_commands,
            authored_paint_order,
        })
    }

    pub(in crate::mounting::presentation) fn text_projection(
        &self,
        frame: UiMountedFrameIdentity,
        instances: &[UiMountedInstanceIdentity],
        changed_index: Option<usize>,
        layout: worth_ui_host_contract::UiQualifiedTextLayoutView<'_>,
    ) -> UiMountedProjectionView {
        let texts = instances
            .iter()
            .enumerate()
            .map(|(index, instance)| {
                self.text(
                    frame,
                    *instance,
                    index,
                    layout,
                    changed_index == Some(index),
                )
            })
            .collect::<Vec<_>>();
        let nodes = texts
            .iter()
            .enumerate()
            .map(|(index, row)| text_node(index, row))
            .collect::<Vec<_>>();
        let (authored_paint_commands, authored_paint_order) =
            crate::mounting::compile_presentation_sources(&nodes, &[], &[], &texts);
        UiMountedProjectionView::new(UiMountedProjectionViewInput {
            frame,
            surface: self.surface,
            binding: self.binding,
            content_generation: self.content,
            nodes,
            clips: UiMountedClipTable::produced(Vec::new()),
            layers: UiMountedLayerTable::produced(Vec::new()),
            filled_rects: UiMountedFilledRectTable::empty(),
            portal_overlays: worth_ui_host_contract::UiMountedPortalOverlayTable::empty(),
            semantic_text: UiMountedSemanticTextTable::from_runtime_mounting(texts).unwrap(),
            hit_tests: UiMountedHitTestTable::empty(),
            paint_batches: UiMountedPaintBatchTable::new(Vec::new()),
            spatial_batches: UiMountedSpatialBatchTable::new(Vec::new()),
            realtime_batches: worth_ui_host_contract::UiMountedRealtimeBatchTable::new(Vec::new()),
            resources: UiMountedResourceTable::new(Vec::new()),
            authored_paint_commands,
            authored_paint_order,
        })
    }

    fn rect(&self, frame: UiMountedFrameIdentity, spec: RectSpec) -> UiMountedFilledRectMechanic {
        let bounds = canonical_box(spec.x, 0.0, 32.0, 24.0);
        UiMountedFilledRectMechanic::complete_from_runtime_mounting(
            UiMountedFilledRectCompletionInput {
                frame,
                surface: self.surface,
                binding: self.binding,
                mounted_instance: spec.instance,
                node_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                    .unwrap()
                    .receipt_for(spec.instance),
                allocation_basis: UiMountedAllocationBasis::new(
                    1,
                    2,
                    3,
                    UiMountedTransformProjection::Identity,
                ),
                bounds,
                color: spec.color,
                layer_semantic_order: (spec.x as u32) / 40,
                clip_bounds: canonical_box(spec.clip_x, 0.0, spec.clip_width, 24.0),
            },
        )
        .unwrap()
    }

    fn text(
        &self,
        frame: UiMountedFrameIdentity,
        instance: UiMountedInstanceIdentity,
        index: usize,
        layout: worth_ui_host_contract::UiQualifiedTextLayoutView<'_>,
        changed: bool,
    ) -> worth_ui_host_contract::UiMountedSemanticTextMechanic {
        use std::sync::Arc;
        let bounds = canonical_box(index as f32 * 40.0, 32.0, 32.0, 24.0);
        worth_ui_host_contract::UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
            worth_ui_host_contract::UiMountedSemanticTextCompletionInput {
                content_generation: self.content,
                frame,
                surface: self.surface,
                binding: self.binding,
                mounted_instance: instance,
                node_receipt: UiMountedNodeReceiptIssuer::mint_for(frame)
                    .unwrap()
                    .receipt_for(instance),
                allocation_basis: UiMountedAllocationBasis::new(
                    1,
                    2,
                    3,
                    UiMountedTransformProjection::Identity,
                ),
                bounds,
                clip_bounds: bounds,
                origin_x: bounds.x(),
                origin_y: bounds.y(),
                text: Arc::from("WORTH"),
                layout,
                slot: worth_ui_host_contract::UiSemanticTextSlot::Value,
                collection_row: None,
                foregrounds: Arc::from([
                    worth_ui_host_contract::UiMountedTextForegroundSpan::from_runtime_mounting(
                        worth_ui_host_contract::UiTextOriginalRange::from_text_mechanics(0, 5)
                            .unwrap(),
                        if changed {
                            UiMountedRgba8::new(242, 204, 96, 255)
                        } else {
                            UiMountedRgba8::new(255, 255, 255, 255)
                        },
                        worth_ui_host_contract::UiMountedTextPaintSpanIdentity::from_runtime_mounting(
                            [1; 32],
                        ),
                    ),
                ]),
                profile: worth_ui_host_contract::UiSemanticTextProfile::BodyDefault,
                layer_semantic_order: u32::try_from(index + 2_048).unwrap(),
                capability_generation: WorthUiHostCapabilityObservationGeneration::new(7),
                capability_profile_digest: 11,
            },
        )
        .unwrap()
    }
}

fn canonical_box(x: f32, y: f32, width: f32, height: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y,
        width,
        height,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
