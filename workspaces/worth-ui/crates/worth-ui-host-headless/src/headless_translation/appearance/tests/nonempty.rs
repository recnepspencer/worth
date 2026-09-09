use worth_ui_host_contract::{
    UiAppearanceAllocationBounds, UiAppearanceBackdropExtent, UiAppearanceClip,
    UiAppearanceDamageRegion, UiMountedAppearanceColor, UiMountedAppearanceFrame,
    UiMountedAppearanceMechanic, UiMountedAppearanceMechanicChange,
    UiMountedAppearancePredecessorManifest, UiMountedAppearanceWork,
    UiMountedAppearanceWorkPosture, UiMountedBackdropAppearanceAttribution,
    UiMountedBackdropCompletionInput, UiMountedBackdropIdentity, UiMountedBackdropMechanic,
    UiMountedBackdropScope, UiMountedLayerProjection, UiMountedLayerReference,
    UiMountedNodeAppearanceAttribution, UiMountedNodeReceiptIssuer, UiMountedOverlayOrderMechanic,
    UiMountedPortalSurfaceAppearanceMechanic, UiMountedPresentationAttemptIdentity,
    UiMountedPresentationOpacity, UiMountedSurfaceAppearanceCompletionInput,
    UiMountedSurfaceAppearanceMechanic, UiMountedSurfacePaint, UiOverlayParticipantIdentity,
    UiOverlayPlacementReceipt, UiSemanticSurfaceIdentity,
};

struct FixtureIds {
    surface: UiSemanticSurfaceIdentity,
    presentation: UiMountedPresentationAttemptIdentity,
    base: worth_ui_host_contract::UiMountedInstanceIdentity,
    portal: worth_ui_host_contract::UiMountedInstanceIdentity,
    backdrop: UiMountedBackdropIdentity,
}

#[test]
fn initial_nonempty_translation_keeps_issued_overlay_order_and_oracle() {
    let ids = fixture_ids();
    let frame = frame_with(
        &ids,
        frame_identity(),
        [255, 0, 0, 128],
        0,
        valid_order(&ids),
    );
    let work = initial_work(&frame, damage(0, 0, 64, 48));

    let transcript = super::super::super::translate_unpublished_appearance_work(&work).unwrap();

    assert_eq!(
        transcript.posture(),
        UiMountedAppearanceWorkPosture::Initial
    );
    assert_eq!(transcript.successor().mechanics().len(), 3);
    assert_eq!(transcript.changes().len(), 3);
    assert_eq!(transcript.damage(), &[damage(0, 0, 64, 48)]);
    assert_eq!(
        transcript.successor().overlay_order().bottom_to_top(),
        frame.overlay_order().bottom_to_top()
    );
    assert_eq!(
        transcript.successor().reference_source_over(),
        color_oracle([255, 0, 0, 128], [0, 255, 0, 128])
    );
    assert!(transcript
        .successor()
        .mechanics()
        .iter()
        .zip(frame.mechanics())
        .all(|(translated, source)| translated.matches_mounted(source)));
}

#[test]
fn delta_and_reconstruction_preserve_exact_damage_and_mechanic_changes() {
    let ids = fixture_ids();
    let predecessor = frame_with(
        &ids,
        frame_identity(),
        [255, 0, 0, 128],
        0,
        valid_order(&ids),
    );
    let successor = frame_with(
        &ids,
        frame_identity(),
        [0, 0, 255, 128],
        0,
        valid_order(&ids),
    );
    let predecessor_backdrop = backdrop_mechanic(&predecessor);
    let successor_backdrop = backdrop_mechanic(&successor);
    let delta = mounted_change_work(
        UiMountedAppearanceWorkPosture::Delta,
        &predecessor,
        &successor,
        vec![UiMountedAppearanceMechanicChange::replacement(
            predecessor_backdrop.identity(),
            successor_backdrop,
        )
        .unwrap()],
        damage(8, 8, 32, 24),
    );
    let delta_transcript =
        super::super::super::translate_unpublished_appearance_work(&delta).unwrap();
    assert_eq!(
        delta_transcript.posture(),
        UiMountedAppearanceWorkPosture::Delta
    );
    assert_eq!(delta_transcript.predecessor(), Some(predecessor.frame()));
    assert!(!delta_transcript.order_changed());
    assert_eq!(delta_transcript.changes().len(), 1);
    assert_eq!(delta_transcript.damage(), &[damage(8, 8, 32, 24)]);
    assert_eq!(
        delta_transcript.successor().reference_source_over(),
        color_oracle([0, 0, 255, 128], [0, 255, 0, 128])
    );

    let reconstruction_changes = predecessor
        .mechanics()
        .iter()
        .zip(successor.mechanics())
        .map(|(before, after)| {
            UiMountedAppearanceMechanicChange::replacement(before.identity(), after.clone())
                .unwrap()
        })
        .collect::<Vec<_>>();
    let reconstruction = mounted_change_work(
        UiMountedAppearanceWorkPosture::Reconstruction,
        &predecessor,
        &successor,
        reconstruction_changes,
        damage(0, 0, 64, 48),
    );
    let reconstruction_transcript =
        super::super::super::translate_unpublished_appearance_work(&reconstruction).unwrap();
    assert_eq!(
        reconstruction_transcript.posture(),
        UiMountedAppearanceWorkPosture::Reconstruction
    );
    assert_eq!(reconstruction_transcript.changes().len(), 3);
    assert_eq!(reconstruction_transcript.damage(), &[damage(0, 0, 64, 48)]);
    assert_eq!(
        reconstruction_transcript
            .successor()
            .reference_source_over(),
        color_oracle([0, 0, 255, 128], [0, 255, 0, 128])
    );
}

#[test]
fn malformed_order_and_backdrop_placement_are_denied() {
    let ids = fixture_ids();
    let foreign_portal = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    let malformed_order = vec![
        UiOverlayParticipantIdentity::Backdrop(ids.backdrop.clone()),
        UiOverlayParticipantIdentity::Portal(foreign_portal),
    ];
    let order_frame = frame_with(&ids, frame_identity(), [255, 0, 0, 128], 0, malformed_order);
    assert_eq!(
        super::super::super::translate_unpublished_appearance_work(&initial_work(
            &order_frame,
            damage(0, 0, 64, 48),
        )),
        Err(super::super::UiHeadlessAppearanceTranslationDenial::InvalidMechanic)
    );

    let misplaced_frame = frame_with(
        &ids,
        frame_identity(),
        [255, 0, 0, 128],
        1,
        valid_order(&ids),
    );
    assert_eq!(
        super::super::super::translate_unpublished_appearance_work(&initial_work(
            &misplaced_frame,
            damage(0, 0, 64, 48),
        )),
        Err(super::super::UiHeadlessAppearanceTranslationDenial::InvalidMechanic)
    );
}

fn fixture_ids() -> FixtureIds {
    let surface = UiSemanticSurfaceIdentity::mint_unbound().unwrap();
    let portal = worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap();
    FixtureIds {
        surface,
        presentation: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        base: worth_ui_host_contract::UiMountedInstanceIdentity::mint_unbound().unwrap(),
        portal,
        backdrop: UiMountedBackdropIdentity::from_runtime_mounting(
            "dialog.scrim",
            UiMountedBackdropScope::PerPortalInstance(portal),
            1,
        )
        .unwrap(),
    }
}

fn frame_identity() -> worth_ui_host_contract::UiMountedFrameIdentity {
    worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap()
}

fn valid_order(ids: &FixtureIds) -> Vec<UiOverlayParticipantIdentity> {
    vec![
        UiOverlayParticipantIdentity::Backdrop(ids.backdrop.clone()),
        UiOverlayParticipantIdentity::Portal(ids.portal),
    ]
}

fn frame_with(
    ids: &FixtureIds,
    frame: worth_ui_host_contract::UiMountedFrameIdentity,
    backdrop_color: [u8; 4],
    backdrop_ordinal: u32,
    order_participants: Vec<UiOverlayParticipantIdentity>,
) -> UiMountedAppearanceFrame {
    let issuer = UiMountedNodeReceiptIssuer::mint_for(frame).unwrap();
    let base_bounds = UiAppearanceAllocationBounds::new(0, 0, 64, 48).unwrap();
    let base = surface(
        issuer,
        ids.base,
        base_bounds,
        UiMountedAppearanceColor::from_straight_srgba([32, 48, 64, 255]),
    );
    let portal_bounds = UiAppearanceAllocationBounds::new(0, 0, 64, 48).unwrap();
    let portal_surface = surface(
        issuer,
        ids.portal,
        portal_bounds,
        UiMountedAppearanceColor::from_straight_srgba([0, 255, 0, 128]),
    );
    let portal = UiMountedPortalSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        ids.portal,
        portal_surface,
    )
    .unwrap();
    let placement =
        UiOverlayPlacementReceipt::from_runtime_overlay_order(7, backdrop_ordinal).unwrap();
    let backdrop = UiMountedBackdropMechanic::complete_from_runtime_mounting(
        UiMountedBackdropCompletionInput {
            identity: ids.backdrop.clone(),
            semantic_surface: ids.surface,
            placement,
            extent: UiAppearanceBackdropExtent::new(8, 8, 32, 24).unwrap(),
            clip: UiAppearanceClip::new(0, 0, 64, 48).unwrap(),
            background: UiMountedAppearanceColor::from_straight_srgba(backdrop_color),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            attribution: UiMountedBackdropAppearanceAttribution::from_runtime_transport(
                ids.surface,
                placement,
                11,
                3,
            )
            .unwrap(),
        },
    )
    .unwrap();
    let order = UiMountedOverlayOrderMechanic::complete_from_runtime_overlay_order(
        ids.surface,
        ids.presentation,
        7,
        11,
        order_participants,
    )
    .unwrap();
    UiMountedAppearanceFrame::from_runtime_mounting(
        frame,
        ids.surface,
        [
            UiMountedAppearanceMechanic::Surface(base),
            UiMountedAppearanceMechanic::PortalSurface(portal),
            UiMountedAppearanceMechanic::Backdrop(backdrop),
        ],
        order,
    )
    .unwrap()
}

fn surface(
    issuer: UiMountedNodeReceiptIssuer,
    instance: worth_ui_host_contract::UiMountedInstanceIdentity,
    bounds: UiAppearanceAllocationBounds,
    color: UiMountedAppearanceColor,
) -> UiMountedSurfaceAppearanceMechanic {
    UiMountedSurfaceAppearanceMechanic::complete_from_runtime_mounting(
        UiMountedSurfaceAppearanceCompletionInput {
            issuer,
            node_receipt: issuer.receipt_for(instance),
            bounds,
            clip: UiAppearanceClip::new(bounds.x(), bounds.y(), bounds.width(), bounds.height())
                .unwrap(),
            surface_paint_order: 0,
            radii: worth_ui_host_contract::UiAppearanceNormalizedLogicalRadii::normalize(
                bounds,
                [worth_ui_host_contract::UiAppearanceLogicalLength::ZERO; 4],
            ),
            border_edges: worth_ui_host_contract::UiMountedSurfaceBorderEdges::ALL,
            border_omissions: Box::new([]),
            paint: UiMountedSurfacePaint::Fill(color),
            opacity: UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
            projection: UiMountedNodeAppearanceAttribution::from_runtime_mounting(issuer, 1, 1)
                .unwrap(),
        },
    )
    .unwrap()
}

fn initial_work(
    frame: &UiMountedAppearanceFrame,
    damage: UiAppearanceDamageRegion,
) -> UiMountedAppearanceWork {
    UiMountedAppearanceWork::from_runtime_mounting(
        UiMountedAppearanceWorkPosture::Initial,
        None,
        None,
        frame.clone(),
        frame
            .mechanics()
            .iter()
            .cloned()
            .map(UiMountedAppearanceMechanicChange::Insert)
            .collect::<Vec<_>>(),
        [damage],
        true,
    )
    .unwrap()
}

fn mounted_change_work(
    posture: UiMountedAppearanceWorkPosture,
    predecessor: &UiMountedAppearanceFrame,
    successor: &UiMountedAppearanceFrame,
    changes: Vec<UiMountedAppearanceMechanicChange>,
    damage: UiAppearanceDamageRegion,
) -> UiMountedAppearanceWork {
    let manifest = UiMountedAppearancePredecessorManifest::from_runtime_mounting(
        predecessor
            .mechanics()
            .iter()
            .map(|mechanic| mechanic.identity()),
        predecessor.overlay_order().bottom_to_top().iter().cloned(),
    )
    .unwrap();
    UiMountedAppearanceWork::from_runtime_mounting(
        posture,
        Some(predecessor.frame()),
        Some(manifest),
        successor.clone(),
        changes,
        [damage],
        false,
    )
    .unwrap()
}

fn backdrop_mechanic(frame: &UiMountedAppearanceFrame) -> UiMountedAppearanceMechanic {
    frame
        .mechanics()
        .iter()
        .find(|mechanic| matches!(mechanic, UiMountedAppearanceMechanic::Backdrop(_)))
        .cloned()
        .unwrap()
}

fn damage(x: i32, y: i32, width: u32, height: u32) -> UiAppearanceDamageRegion {
    UiAppearanceDamageRegion::new(x, y, width, height).unwrap()
}

fn color_oracle(bottom: [u8; 4], top: [u8; 4]) -> UiMountedAppearanceColor {
    let bottom_alpha = f64::from(bottom[3]) / 255.0;
    let top_alpha = f64::from(top[3]) / 255.0;
    let output_alpha = top_alpha + bottom_alpha * (1.0 - top_alpha);
    let channel = |index: usize| {
        let linear = (srgb_to_linear(top[index]) * top_alpha
            + srgb_to_linear(bottom[index]) * bottom_alpha * (1.0 - top_alpha))
            / output_alpha;
        linear_to_srgb(linear)
    };
    UiMountedAppearanceColor::from_straight_srgba([
        channel(0),
        channel(1),
        channel(2),
        (output_alpha * 255.0).round() as u8,
    ])
}

fn srgb_to_linear(channel: u8) -> f64 {
    let encoded = f64::from(channel) / 255.0;
    if encoded <= 0.04045 {
        encoded / 12.92
    } else {
        ((encoded + 0.055) / 1.055).powf(2.4)
    }
}

fn linear_to_srgb(linear: f64) -> u8 {
    let encoded = if linear <= 0.0031308 {
        linear * 12.92
    } else {
        1.055 * linear.powf(1.0 / 2.4) - 0.055
    };
    (encoded.clamp(0.0, 1.0) * 255.0).round() as u8
}
