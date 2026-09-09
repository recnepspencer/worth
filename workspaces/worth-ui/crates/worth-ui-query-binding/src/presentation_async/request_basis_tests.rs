use super::*;
use worth_ui_host_contract::{
    UiFontCollectionGeneration, UiFontCollectionLineageIdentity, UiGlyphRasterFractionalOrigin,
    UiGlyphRasterKey, UiGlyphRasterKeyInput, UiGlyphRasterPalette, UiGlyphRasterPinRequest,
    UiGlyphRasterSize, UiGlyphRasterSource, UiGlyphVariationCoordinates,
    UiQualifiedFontFaceIdentity, UiTextProfileGeneration,
};

#[test]
fn attempt_basis_is_mechanic_order_independent_and_retains_typed_content() {
    let first = mechanic_input(11, [1; 32], 0, 4, [10, 20, 30, 255]);
    let second = mechanic_input(12, [2; 32], 4, 8, [40, 50, 60, 255]);
    let input = basis_input(vec![second.clone(), first.clone()]);
    let reordered_input = WorthUiPresentationRequestBasisInput {
        mechanics: vec![first, second].into_boxed_slice(),
        ..input.clone()
    };
    let basis = WorthUiPresentationRequestBasis::from_runtime_correspondence(input).unwrap();
    let reordered =
        WorthUiPresentationRequestBasis::from_runtime_correspondence(reordered_input).unwrap();

    assert_eq!(basis.identity_parts(), reordered.identity_parts());
    assert_eq!(basis.mechanics()[0].content(), "text");
}

#[test]
fn nonadjacent_duplicate_paint_identities_are_denied() {
    let mounted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    let mechanic = WorthUiPresentationMechanicBasisInput {
        mounted_instance,
        mechanic: UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
            mounted_instance,
            0,
            None,
        ),
        content_generation: UiMountedContentGeneration::mint_unbound().unwrap(),
        content: std::sync::Arc::from("text"),
        layout: UiQualifiedTextLayoutIdentity::from_text_mechanics([7; 32]),
        layout_request: UiQualifiedTextLayoutRequestIdentity::from_text_mechanics([6; 32]),
        layout_width: UiQualifiedTextLayoutWidthBasis::new(80_000).unwrap(),
        paint_spans: vec![
            paint([1; 32], 0, 2, [1, 2, 3, 255]),
            paint([2; 32], 2, 4, [4, 5, 6, 255]),
            paint([1; 32], 4, 6, [7, 8, 9, 255]),
        ]
        .into_boxed_slice(),
        raster_keys: vec![raster_key(8)].into_boxed_slice(),
        text_scale: UiTextScaleGeneration::new(2).unwrap(),
    };

    assert_eq!(
        WorthUiPresentationRequestBasis::from_runtime_correspondence(basis_input(vec![mechanic]))
            .unwrap_err(),
        WorthUiPresentationRequestBasisDenial::DuplicatePaintSpan
    );
}

#[test]
fn whole_binding_pin_inventory_participates_in_query_request_identity() {
    let mechanic = mechanic_input(13, [3; 32], 0, 4, [1, 2, 3, 255]);
    let input = basis_input(vec![mechanic.clone()]);
    let without_pin =
        WorthUiPresentationRequestBasis::from_runtime_correspondence(input.clone()).unwrap();
    let pin = pin_basis(mechanic.layout);
    let with_pin = WorthUiPresentationRequestBasis::from_runtime_correspondence(
        WorthUiPresentationRequestBasisInput {
            binding_pins: vec![pin].into_boxed_slice(),
            ..input
        },
    )
    .unwrap();

    assert_ne!(without_pin.identity_parts(), with_pin.identity_parts());
    assert!(with_pin
        .identity_parts()
        .iter()
        .any(|part| part.key() == "binding-pins-fingerprint"));
}

#[test]
fn complete_empty_reconstruction_and_currentness_only_delta_are_meaningful() {
    let complete = basis_input(Vec::new());
    assert!(WorthUiPresentationRequestBasis::from_runtime_correspondence(complete.clone()).is_ok());

    let currentness = WorthUiPresentationRequestBasis::from_runtime_correspondence(
        WorthUiPresentationRequestBasisInput {
            complete: false,
            ..complete
        },
    )
    .unwrap();
    assert!(!currentness.complete());
    assert!(currentness.mechanics().is_empty());
}

#[test]
fn four_thousand_ninety_six_mechanics_have_a_bounded_drift_sensitive_identity() {
    let mechanics = (0..4_096)
        .map(|index| mechanic_input(index as u16, digest(index as u64), 0, 4, [1, 2, 3, 255]))
        .collect();
    let input = basis_input(mechanics);
    let mut changed_input = input.clone();
    changed_input.mechanics[4_095].content = std::sync::Arc::from("changed-last-mechanic");
    let basis = WorthUiPresentationRequestBasis::from_runtime_correspondence(input).unwrap();
    let changed =
        WorthUiPresentationRequestBasis::from_runtime_correspondence(changed_input).unwrap();

    assert_eq!(basis.mechanics().len(), 4_096);
    assert_eq!(basis.identity_parts().len(), 20);
    assert_ne!(basis.identity_parts(), changed.identity_parts());
    assert!(
        crate::presentation_async::WorthUiPresentationAsyncDeclaration::declare(&basis).is_ok()
    );
}

#[test]
fn length_delimited_content_cannot_shift_across_mechanic_boundaries() {
    let mut input = basis_input(vec![
        mechanic_input(21, [21; 32], 0, 1, [1, 2, 3, 255]),
        mechanic_input(22, [22; 32], 0, 1, [1, 2, 3, 255]),
    ]);
    input.mechanics[0].content = std::sync::Arc::from("ab");
    input.mechanics[1].content = std::sync::Arc::from("c");
    let mut shifted_input = input.clone();
    shifted_input.mechanics[0].content = std::sync::Arc::from("a");
    shifted_input.mechanics[1].content = std::sync::Arc::from("bc");

    let basis = WorthUiPresentationRequestBasis::from_runtime_correspondence(input).unwrap();
    let shifted =
        WorthUiPresentationRequestBasis::from_runtime_correspondence(shifted_input).unwrap();
    assert_ne!(basis.identity_parts(), shifted.identity_parts());
}

fn basis_input(
    mechanics: Vec<WorthUiPresentationMechanicBasisInput>,
) -> WorthUiPresentationRequestBasisInput {
    WorthUiPresentationRequestBasisInput {
        mounted_frame: UiMountedFrameIdentity::mint_unbound().unwrap(),
        semantic_surface: UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
        host_surface: UiHostSurfaceIdentity::mint_unbound().unwrap(),
        binding: UiSurfaceBindingGeneration::mint_unbound().unwrap(),
        complete: true,
        mechanics: mechanics.into_boxed_slice(),
        removed_mechanics: Box::new([]),
        binding_pins: Box::new([]),
        pin_additions: Box::new([]),
        pin_releases: Box::new([]),
        dpi_milli: 1_250,
        attempt: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        predecessor: Some(UiMountedFrameIdentity::mint_unbound().unwrap()),
        host_lineage: UiHostPresentationLineageIdentity::from_certification_host_session(9)
            .unwrap(),
    }
}

fn mechanic_input(
    slot: u16,
    identity: [u8; 32],
    start: u32,
    end: u32,
    foreground: [u8; 4],
) -> WorthUiPresentationMechanicBasisInput {
    let mounted_instance = UiMountedInstanceIdentity::mint_unbound().unwrap();
    WorthUiPresentationMechanicBasisInput {
        mounted_instance,
        mechanic: UiMountedPaintCommandIdentity::semantic_text_from_correspondence(
            mounted_instance,
            slot,
            None,
        ),
        content_generation: UiMountedContentGeneration::mint_unbound().unwrap(),
        content: std::sync::Arc::from("text"),
        layout: UiQualifiedTextLayoutIdentity::from_text_mechanics(digest(u64::from(slot))),
        layout_request: UiQualifiedTextLayoutRequestIdentity::from_text_mechanics(digest(
            (1_u64 << 32) | u64::from(slot),
        )),
        layout_width: UiQualifiedTextLayoutWidthBasis::new(80_000).unwrap(),
        paint_spans: vec![paint(identity, start, end, foreground)].into_boxed_slice(),
        raster_keys: vec![raster_key(u32::from(slot))].into_boxed_slice(),
        text_scale: UiTextScaleGeneration::new(2).unwrap(),
    }
}

fn paint(
    identity: [u8; 32],
    start: u32,
    end: u32,
    foreground: [u8; 4],
) -> WorthUiPresentationPaintSpanBasis {
    WorthUiPresentationPaintSpanBasis {
        identity,
        original_range: UiTextOriginalRange::new(start, end).unwrap(),
        foreground,
    }
}

fn pin_basis(layout: UiQualifiedTextLayoutIdentity) -> WorthUiPresentationPinBasis {
    WorthUiPresentationPinBasis::from_runtime(UiGlyphRasterPinRequest::from_text_mechanics(
        layout,
        raster_key(1),
    ))
}

fn raster_key(glyph_id: u32) -> UiGlyphRasterKey {
    UiGlyphRasterKey::from_text_mechanics(UiGlyphRasterKeyInput {
        font_collection: UiFontCollectionGeneration::new(1).unwrap(),
        font_collection_lineage: UiFontCollectionLineageIdentity::from_text_mechanics([4; 32]),
        profile: UiTextProfileGeneration::new(1).unwrap(),
        face: UiQualifiedFontFaceIdentity::from_text_mechanics([1; 32], 0),
        glyph_id,
        variations: UiGlyphVariationCoordinates::empty(),
        palette: UiGlyphRasterPalette::new(0),
        size: UiGlyphRasterSize::from_millipoints(12_000).unwrap(),
        source: UiGlyphRasterSource::AlphaOutline,
        dpi_milli: 1_250,
        origin: UiGlyphRasterFractionalOrigin::from_sixty_fourths(0, 0),
    })
    .unwrap()
}

fn digest(seed: u64) -> [u8; 32] {
    let mut bytes = [0; 32];
    bytes[..8].copy_from_slice(&seed.to_le_bytes());
    bytes[8..16].copy_from_slice(&seed.rotate_left(17).to_le_bytes());
    bytes
}
