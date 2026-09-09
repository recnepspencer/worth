use super::session::World;
use crate::mounting::{
    UiMountedFrameOutcome, UiMountedSurfaceReconciliationBinding,
    UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile,
};
use worth_ui_host_contract::*;

pub(super) fn cold_surface(world: &mut World) {
    assert_eq!(world.host.pending_presentation_count(), 0);
    let surface = world.surfaces[0];
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let affected = presentation.binding();
    let (inside_portal, outside_portals) = reference_output(world, surface);

    let lost = world
        .session
        .mounted
        .require_current_layout_reconstruction(affected)
        .unwrap();
    assert!(
        lost > 0,
        "cold reconstruction discards real qualified text layouts"
    );
    assert_eq!(
        world.session.mounted.reconstruct_current_layouts().unwrap(),
        lost,
        "the retained semantic sources rebuild every discarded layout"
    );
    let replacement = world
        .session
        .rebind_host_surface(
            affected,
            UiHostSurfacePresentationMode::NativeDisplay,
            UiSurfaceBindingProfile::new(
                1_000,
                UiSurfaceBindingCoordinatePosture::LogicalPoints,
                1,
            )
            .unwrap(),
        )
        .unwrap()
        .binding_generation();
    let replacements = [UiMountedSurfaceReconciliationBinding::new(
        affected,
        replacement,
    )];
    let frame = world
        .session
        .prepare_mounted_reconstruction_frame_with_application_presentation(
            crate::mounting::UiMountedFrameRequest::all_bound_surfaces(),
            &replacements,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("the cold authored world must prepare reconstruction"));
    for surface in frame.surfaces() {
        if surface.requirement().binding() == replacement {
            world.host.push_native_display_presented();
        } else {
            world.host.push_native_display_settled_without_effects();
        }
    }
    let outcome = world
        .session
        .present_prepared_mounted_frame_for_reconciliation(
            frame,
            &replacements,
            UiPresentationDeadline::at_tick(u64::MAX),
            400,
        )
        .unwrap();
    match outcome {
        UiMountedFrameOutcome::Reconciled(_) => {}
        UiMountedFrameOutcome::AdmissionDenied(denial) => {
            panic!("cold reconstruction admission: {:?}", denial.denial())
        }
        UiMountedFrameOutcome::RejectedBeforeEffects(rejected) => {
            panic!("cold reconstruction rejected: {:?}", rejected.rejections())
        }
        UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
            panic!("cold reconstruction indeterminate: {:?}", frame.report())
        }
        other => panic!(
            "cold reconstruction outcome: {:?}",
            std::mem::discriminant(&other)
        ),
    }

    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let affected_fragments = output
        .fragments()
        .iter()
        .filter(|fragment| fragment.surface_binding().semantic_surface() == surface)
        .collect::<Vec<_>>();
    assert!(!affected_fragments.is_empty());
    assert!(affected_fragments
        .iter()
        .all(|fragment| fragment.surface_binding().binding() == replacement));
    assert!(affected_fragments.iter().all(|fragment| {
        matches!(
            fragment.identity(),
            UiUnpublishedAppearanceFragmentIdentity::SurfacePointer { .. }
        ) || fragment.work().posture() == UiMountedAppearanceWorkPosture::Reconstruction
    }));
    assert_complete_reconstructed_mechanics(&affected_fragments);
    assert_original_text_ranges(&affected_fragments);
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let overlay = transcript
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
        })
        .unwrap();
    assert_eq!(
        overlay
            .work()
            .successor()
            .reference_overlay_at(310_000, 60_000)
            .straight_srgba(),
        inside_portal
    );
    assert_eq!(
        overlay
            .work()
            .successor()
            .reference_overlay_at(750_000, 550_000)
            .straight_srgba(),
        outside_portals
    );
}

fn reference_output(world: &World, surface: UiSemanticSurfaceIdentity) -> ([u8; 4], [u8; 4]) {
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    let transcript =
        worth_ui_host_headless::translate_unpublished_appearance_for_certification(output).unwrap();
    let overlay = transcript
        .fragments()
        .iter()
        .find(|fragment| {
            fragment.identity() == UiUnpublishedAppearanceFragmentIdentity::SurfaceOverlay(surface)
        })
        .unwrap();
    (
        overlay
            .work()
            .successor()
            .reference_overlay_at(310_000, 60_000)
            .straight_srgba(),
        overlay
            .work()
            .successor()
            .reference_overlay_at(750_000, 550_000)
            .straight_srgba(),
    )
}

fn assert_complete_reconstructed_mechanics(
    fragments: &[&worth_ui_host_contract::UiUnpublishedAppearanceFragment],
) {
    let mechanics = fragments
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .collect::<Vec<_>>();
    let identities = mechanics
        .iter()
        .map(|mechanic| mechanic.identity())
        .collect::<std::collections::HashSet<_>>();
    assert_eq!(
        identities.len(),
        mechanics.len(),
        "cold reconstruction must not duplicate a retained mechanic identity"
    );
    let counts = mechanics.iter().fold([0usize; 6], |mut counts, mechanic| {
        counts[match mechanic {
            UiMountedAppearanceMechanic::Surface(_) => 0,
            UiMountedAppearanceMechanic::Outline(_) => 1,
            UiMountedAppearanceMechanic::TextForeground(_) => 2,
            UiMountedAppearanceMechanic::PortalSurface(_) => 3,
            UiMountedAppearanceMechanic::Backdrop(_) => 4,
            UiMountedAppearanceMechanic::Pointer(_) => 5,
        }] += 1;
        counts
    });
    assert_eq!(
        counts,
        [2, 3, 3, 1, 2, 0],
        "the authored surviving world reconstructs mounted mechanics without stale rebound pointer evidence"
    );
}

fn assert_original_text_ranges(
    fragments: &[&worth_ui_host_contract::UiUnpublishedAppearanceFragment],
) {
    let text = fragments
        .iter()
        .flat_map(|fragment| fragment.text_candidates())
        .filter(|candidate| candidate.text() == "AB")
        .collect::<Vec<_>>();
    assert_eq!(
        text.len(),
        3,
        "terminal parent/child closure leaves three mounted text candidates"
    );
    for candidate in text {
        assert_eq!(
            candidate.foregrounds()[0].original_range(),
            UiTextOriginalRange::new(0, 1).unwrap()
        );
        assert_eq!(
            candidate.foregrounds()[1].original_range(),
            UiTextOriginalRange::new(1, 2).unwrap()
        );
    }
}
