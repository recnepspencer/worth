use std::{ops::Deref, sync::Arc};

use worth_ui_host_contract::{UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic};

use super::super::UiMountedProjectionDenial;

mod frame_affinity;

#[derive(Clone)]
pub(in crate::mounting::projection) struct UiMountedQualifiedSemanticText {
    mechanic: UiMountedSemanticTextMechanic,
    layout: UiMountedQualifiedLayoutState,
}

#[derive(Clone)]
enum UiMountedQualifiedLayoutState {
    Current(Arc<worth_ui_text::UiQualifiedTextLayout>),
    ReconstructionRequired(Arc<worth_ui_text::UiQualifiedTextReconstructionSource>),
}

pub(in crate::mounting::projection) struct UiMountedSemanticTextRepaintInput {
    pub(in crate::mounting::projection) content_generation:
        worth_ui_host_contract::UiMountedContentGeneration,
    pub(in crate::mounting::projection) frame: worth_ui_host_contract::UiMountedFrameIdentity,
    pub(in crate::mounting::projection) node_receipt:
        worth_ui_host_contract::UiMountedNodeReceiptIdentity,
    pub(in crate::mounting::projection) capability_generation:
        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration,
    pub(in crate::mounting::projection) capability_profile_digest: u64,
    pub(in crate::mounting::projection) foregrounds:
        Arc<[worth_ui_host_contract::UiMountedTextForegroundSpan]>,
}

impl UiMountedQualifiedSemanticText {
    pub(super) fn new(
        mechanic: UiMountedSemanticTextMechanic,
        layout: Arc<worth_ui_text::UiQualifiedTextLayout>,
    ) -> Self {
        debug_assert_eq!(mechanic.qualified_layout_identity(), layout.identity());
        Self {
            mechanic,
            layout: UiMountedQualifiedLayoutState::Current(layout),
        }
    }

    pub(in crate::mounting::projection) const fn mechanic(&self) -> &UiMountedSemanticTextMechanic {
        &self.mechanic
    }

    pub(in crate::mounting::projection) fn mechanic_clone(&self) -> UiMountedSemanticTextMechanic {
        self.mechanic.clone()
    }

    pub(in crate::mounting::projection) fn qualified_layout(
        &self,
    ) -> Option<&Arc<worth_ui_text::UiQualifiedTextLayout>> {
        match &self.layout {
            UiMountedQualifiedLayoutState::Current(layout) => Some(layout),
            UiMountedQualifiedLayoutState::ReconstructionRequired(_) => None,
        }
    }

    pub(in crate::mounting::projection) const fn layout_reconstruction_required(&self) -> bool {
        matches!(
            self.layout,
            UiMountedQualifiedLayoutState::ReconstructionRequired(_)
        )
    }

    pub(in crate::mounting::projection) fn require_layout_reconstruction(
        &mut self,
    ) -> Result<bool, UiMountedProjectionDenial> {
        let UiMountedQualifiedLayoutState::Current(layout) = &self.layout else {
            return Ok(false);
        };
        let source = layout
            .reconstruction_source()
            .cloned()
            .ok_or(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource)?;
        self.layout = UiMountedQualifiedLayoutState::ReconstructionRequired(source);
        Ok(true)
    }

    pub(in crate::mounting::projection) fn reconstruct_layout(
        &mut self,
    ) -> Result<bool, UiMountedProjectionDenial> {
        let UiMountedQualifiedLayoutState::ReconstructionRequired(source) = &self.layout else {
            return Ok(false);
        };
        let layout = source
            .reconstruct()
            .map_err(UiMountedProjectionDenial::SemanticTextReconstruction)?;
        if layout.identity() != self.mechanic.qualified_layout_identity()
            || layout.source() != self.mechanic.text()
        {
            return Err(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource);
        }
        self.layout = UiMountedQualifiedLayoutState::Current(Arc::new(layout));
        Ok(true)
    }

    pub(in crate::mounting::projection) fn font_collection_matches(
        &self,
        candidate: &Arc<worth_ui_text::UiGlobalFontCollection>,
    ) -> bool {
        match &self.layout {
            UiMountedQualifiedLayoutState::Current(layout) => {
                Arc::ptr_eq(layout.pinned_font_collection(), candidate)
            }
            UiMountedQualifiedLayoutState::ReconstructionRequired(source) => {
                source.matches_font_collection(candidate)
            }
        }
    }

    pub(super) fn rebind(
        &self,
        replacement: super::super::super::UiSurfaceBindingIdentityView,
        allocation_basis: worth_ui_host_contract::UiMountedAllocationBasis,
    ) -> Result<Self, UiMountedProjectionDenial> {
        if replacement.semantic_surface_identity() != self.surface() {
            return Err(UiMountedProjectionDenial::MissingSurfaceBinding);
        }
        let layout = self
            .qualified_layout()
            .ok_or(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource)?;
        let mechanic =
            UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(
                UiMountedSemanticTextCompletionInput {
                    content_generation: self.content_generation(),
                    frame: self.frame(),
                    surface: self.surface(),
                    binding: replacement.binding_generation(),
                    mounted_instance: self.mounted_instance(),
                    portal_group: self.portal_group(),
                    node_receipt: self.node_receipt(),
                    allocation_basis,
                    bounds: self.bounds(),
                    clip_bounds: self.clip_bounds(),
                    origin_x: self.origin_x(),
                    origin_y: self.origin_y(),
                    text: Arc::from(self.text()),
                    layout: layout.view(),
                    slot: self.slot(),
                    collection_row: self.collection_row().cloned(),
                    foregrounds: Arc::from(self.foregrounds()),
                    profile: self.profile(),
                    layer_semantic_order: self.layer_semantic_order(),
                    capability_generation: self.capability_generation(),
                    capability_profile_digest: self.capability_profile_digest(),
                },
            )
            .map_err(UiMountedProjectionDenial::SemanticTextCompletion)?;
        Ok(Self::new(mechanic, Arc::clone(layout)))
    }

    pub(in crate::mounting::projection) fn repaint(
        &self,
        input: UiMountedSemanticTextRepaintInput,
    ) -> Result<Self, UiMountedProjectionDenial> {
        let layout = self
            .qualified_layout()
            .ok_or(UiMountedProjectionDenial::MissingSemanticTextReconstructionSource)?;
        let mechanic =
            UiMountedSemanticTextMechanic::complete_from_runtime_mounting_with_reused_layout(
                UiMountedSemanticTextCompletionInput {
                    content_generation: input.content_generation,
                    frame: input.frame,
                    surface: self.surface(),
                    binding: self.binding(),
                    mounted_instance: self.mounted_instance(),
                    portal_group: self.portal_group(),
                    node_receipt: input.node_receipt,
                    allocation_basis: self.allocation_basis(),
                    bounds: self.bounds(),
                    clip_bounds: self.clip_bounds(),
                    origin_x: self.origin_x(),
                    origin_y: self.origin_y(),
                    text: Arc::from(self.text()),
                    layout: layout.view(),
                    slot: self.slot(),
                    collection_row: self.collection_row().cloned(),
                    foregrounds: input.foregrounds,
                    profile: self.profile(),
                    layer_semantic_order: self.layer_semantic_order(),
                    capability_generation: input.capability_generation,
                    capability_profile_digest: input.capability_profile_digest,
                },
            )
            .map_err(UiMountedProjectionDenial::SemanticTextCompletion)?;
        Ok(Self::new(mechanic, Arc::clone(layout)))
    }
}

impl Deref for UiMountedQualifiedSemanticText {
    type Target = UiMountedSemanticTextMechanic;

    fn deref(&self) -> &Self::Target {
        &self.mechanic
    }
}

pub(in crate::mounting::projection) fn rebind_semantic_text(
    rows: &mut [UiMountedQualifiedSemanticText],
    replacements: &[(
        worth_ui_host_contract::UiSurfaceBindingGeneration,
        super::super::super::UiSurfaceBindingIdentityView,
    )],
    semantic: &super::super::frame_storage::UiMountedSemanticProjection,
) -> Result<(), UiMountedProjectionDenial> {
    for row in rows {
        let Some((_, replacement)) = replacements
            .iter()
            .find(|(affected, _)| *affected == row.binding())
        else {
            continue;
        };
        let node = semantic
            .node(row.mounted_instance())
            .ok_or(UiMountedProjectionDenial::UnknownGraphNode)?;
        let allocation_basis = match node.presentation_allocation() {
            worth_ui_host_contract::UiMountedAllocationProjection::Known { basis, .. }
            | worth_ui_host_contract::UiMountedAllocationProjection::PortalAnchorObservation {
                basis,
                ..
            } => basis,
            worth_ui_host_contract::UiMountedAllocationProjection::Omitted(_) => {
                return Err(UiMountedProjectionDenial::MissingSemanticTextAllocation(
                    node.receipt().graph_node(),
                ));
            }
        };
        // Text retains its concrete layout allocation lineage. The mounted
        // owner supplies only the replacement surface's coordinate authority;
        // its regional allocation is not the text layout allocation.
        let text_basis = row.allocation_basis();
        let rebound_basis = worth_ui_host_contract::UiMountedAllocationBasis::new(
            text_basis.receipt_identity(),
            text_basis.receipt_generation(),
            allocation_basis.coordinate_ownership(),
            text_basis.transform(),
        );
        *row = row.rebind(*replacement, rebound_basis)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repaint_reuses_the_exact_qualified_layout_owner() {
        let (fonts, _) = worth_ui_text::UiGlobalFontCollection::admit_qualified_profile().unwrap();
        let fonts = Arc::new(fonts);
        let source: Arc<str> = Arc::from("WORTH");
        let constraints = worth_ui_text::UiTextParagraphConstraints::new(
            worth_ui_text::UiTextParagraphConstraintsInput {
                language: Arc::from("und"),
                base_direction: worth_ui_text::UiTextBaseDirection::Auto,
                wrap: worth_ui_text::UiTextWrap::UnicodeWord,
                alignment: worth_ui_text::UiTextAlignment::Start,
                overflow: worth_ui_text::UiTextOverflow::Clip,
                font_size_millipoints: 14_000,
                width_millipoints: 160_000,
                line_height_millipoints: 18_000,
                letter_spacing_millipoints: 0,
                word_spacing_millipoints: 0,
                tab_interval_millipoints: 56_000,
                maximum_lines: 1,
            },
        )
        .unwrap();
        let range = worth_ui_host_contract::UiTextOriginalRange::new(0, 5).unwrap();
        let style = worth_ui_text::UiTextStyleSpan::new(
            range,
            worth_ui_text::UiTextStyle::from_paragraph_constraints(&constraints),
        )
        .unwrap();
        let layout = Arc::new(
            worth_ui_text::qualify_text_layout(
                worth_ui_text::UiTextParagraphAdmissionInput {
                    source: Arc::clone(&source),
                    constraints,
                    profile_generation: worth_ui_host_contract::UiTextProfileGeneration::new(1)
                        .unwrap(),
                    font_collection_generation: fonts.generation(),
                    text_scale_generation: worth_ui_host_contract::UiTextScaleGeneration::new(1)
                        .unwrap(),
                    styles: Box::new([style]),
                },
                fonts,
            )
            .unwrap(),
        );
        let frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let instance = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
        let bounds = worth_ui_host_contract::UiMountedCanonicalBox::canonicalize(
            worth_ui_host_contract::UiMountedCanonicalBoxInput {
                x: 0.0,
                y: 0.0,
                width: 160.0,
                height: 96.0,
                coordinate_space: worth_ui_host_contract::UiMountedCoordinateSpace::Viewport,
            },
        )
        .unwrap();
        let row = UiMountedQualifiedSemanticText::new(
            UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
                UiMountedSemanticTextCompletionInput {
                    content_generation:
                        worth_ui_host_contract::UiMountedContentGeneration::mint_unbound().unwrap(),
                    frame,
                    surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound()
                        .unwrap(),
                    binding: worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
                        .unwrap(),
                    mounted_instance: instance,
                    portal_group: None,
                    node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(
                        frame,
                    )
                    .unwrap()
                    .receipt_for(instance),
                    allocation_basis: worth_ui_host_contract::UiMountedAllocationBasis::new(
                        1,
                        2,
                        3,
                        worth_ui_host_contract::UiMountedTransformProjection::Identity,
                    ),
                    bounds,
                    clip_bounds: bounds,
                    origin_x: 0.0,
                    origin_y: 0.0,
                    text: source,
                    layout: layout.view(),
                    slot: worth_ui_host_contract::UiSemanticTextSlot::Value,
                    collection_row: None,
                    foregrounds: foreground(range, [255, 255, 255, 255]),
                    profile: worth_ui_host_contract::UiSemanticTextProfile::BodyDefault,
                    layer_semantic_order: 1,
                    capability_generation:
                        worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration::new(1),
                    capability_profile_digest: 1,
                },
            )
            .unwrap(),
            Arc::clone(&layout),
        );
        let successor_frame =
            worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
        let successor_content =
            worth_ui_host_contract::UiMountedContentGeneration::mint_unbound().unwrap();
        let successor_receipt =
            worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(successor_frame)
                .unwrap()
                .receipt_for(instance);
        let repainted = row
            .repaint(UiMountedSemanticTextRepaintInput {
                content_generation: successor_content,
                frame: successor_frame,
                node_receipt: successor_receipt,
                capability_generation:
                    worth_ui_host_contract::WorthUiHostCapabilityObservationGeneration::new(2),
                capability_profile_digest: 2,
                foregrounds: foreground(range, [247, 129, 47, 255]),
            })
            .unwrap();
        assert!(Arc::ptr_eq(
            row.qualified_layout().unwrap(),
            repainted.qualified_layout().unwrap()
        ));
        assert_eq!(repainted.frame(), successor_frame);
        assert_eq!(repainted.content_generation(), successor_content);
        assert_eq!(repainted.node_receipt(), successor_receipt);
        assert_ne!(row.semantic_digest(), repainted.semantic_digest());
        assert_eq!(
            repainted.foregrounds()[0].color().channels(),
            [247, 129, 47, 255]
        );
    }

    fn foreground(
        range: worth_ui_host_contract::UiTextOriginalRange,
        color: [u8; 4],
    ) -> Arc<[worth_ui_host_contract::UiMountedTextForegroundSpan]> {
        Arc::from([
            worth_ui_host_contract::UiMountedTextForegroundSpan::from_runtime_mounting(
                range,
                worth_ui_host_contract::UiMountedRgba8::new(color[0], color[1], color[2], color[3]),
                worth_ui_host_contract::UiMountedTextPaintSpanIdentity::from_runtime_mounting(
                    [1; 32],
                ),
            ),
        ])
    }
}
