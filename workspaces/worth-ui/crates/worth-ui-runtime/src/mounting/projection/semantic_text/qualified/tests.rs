use std::sync::Arc;

use worth_ui_host_contract::{UiMountedSemanticTextCompletionInput, UiMountedSemanticTextMechanic};

use super::{UiMountedQualifiedSemanticText, UiMountedSemanticTextRepaintInput};

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
                surface: worth_ui_host_contract::UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
                binding: worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound()
                    .unwrap(),
                mounted_instance: instance,
                portal_group: None,
                node_receipt: worth_ui_host_contract::UiMountedNodeReceiptIssuer::mint_for(frame)
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
    let successor_frame = worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap();
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
            worth_ui_host_contract::UiMountedTextPaintSpanIdentity::from_runtime_mounting([1; 32]),
        ),
    ])
}
