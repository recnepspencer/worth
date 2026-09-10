use super::session::World;
use crate::mounting::{
    UiMountedFrameOutcome, UiMountedSurfaceReconciliationBinding,
    UiSurfaceBindingCoordinatePosture, UiSurfaceBindingProfile,
};
use worth_ui_host_contract::*;

pub(super) fn reject_retry_at_nonterminal_sample(
    world: &mut World,
    expected_opacity: u16,
    rejected_at: u64,
    accepted_at: u64,
) {
    let surface = world.surfaces[0];
    let current = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    let affected = current.binding();
    let accepted_frame = world.session.current_mounted_publication().unwrap().frame();
    let accepted_output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap()
        .clone();
    let accepted_owner = world
        .session
        .overlay_composition_owners
        .current_for_test(surface)
        .unwrap()
        .current()
        .unwrap()
        .clone();
    let accepted_samples = world
        .session
        .current_mounted_projection_rc_for_test()
        .unwrap()
        .view_for(affected)
        .unwrap()
        .retained_paint_commands()
        .iter()
        .filter_map(|command| {
            world
                .session
                .mounted
                .accepted_motion_for_command(current, command.identity())
                .unwrap()
                .map(|sample| (command.identity(), sample.opacity_units()))
        })
        .collect::<Vec<_>>();
    let moving_commands = accepted_samples
        .iter()
        .filter_map(|(command, opacity)| (*opacity == expected_opacity).then_some(*command))
        .collect::<Vec<_>>();
    assert!(
        !moving_commands.is_empty(),
        "the cold path starts mid-Motion"
    );
    let accepted_pixels = reference_pixels(&accepted_output, surface);
    let lost = world
        .session
        .mounted
        .require_current_layout_reconstruction(affected)
        .unwrap();
    assert!(
        lost > 0,
        "the nonterminal cold path loses real text layouts"
    );
    assert_eq!(
        world.session.mounted.reconstruct_current_layouts().unwrap(),
        lost
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
    let reconstruction_observation_start = world.host.reconstruction_sample_overrides().len();
    let rejected = prepare(world, &replacements);
    let bindings = rejected
        .surfaces()
        .iter()
        .map(|surface| surface.requirement().binding())
        .collect::<Vec<_>>();
    assert!(bindings.contains(&replacement), "bindings={bindings:?}");
    assert!(!bindings.contains(&affected), "bindings={bindings:?}");
    for _ in rejected.surfaces() {
        world.host.push_rejected();
    }
    let outcome = world
        .session
        .present_prepared_mounted_frame_for_reconciliation(
            rejected,
            &replacements,
            UiPresentationDeadline::at_tick(u64::MAX),
            rejected_at,
        )
        .unwrap();
    match outcome {
        UiMountedFrameOutcome::RejectedBeforeEffects(_) => {}
        UiMountedFrameOutcome::AdmissionDenied(denial) => {
            panic!("mid-Motion rejection admission: {:?}", denial.denial())
        }
        UiMountedFrameOutcome::PresentationIndeterminate(frame) => {
            panic!("mid-Motion rejection indeterminate: {:?}", frame.report())
        }
        other => panic!(
            "mid-Motion rejection outcome: {:?}",
            std::mem::discriminant(&other)
        ),
    }
    assert_eq!(world.host.pending_presentation_count(), 0);
    assert_eq!(
        world.session.current_mounted_publication().unwrap().frame(),
        accepted_frame,
        "rejection cannot replace the accepted mounted frame"
    );
    assert!(
        world
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap()
            .is_none(),
        "the deregistered binding cannot expose stale appearance while replacement work is rejected"
    );
    assert_eq!(
        world
            .session
            .overlay_composition_owners
            .current_for_test(surface)
            .unwrap()
            .current()
            .unwrap(),
        &accepted_owner,
        "reconciliation rejection discards staged overlay ownership"
    );

    let retry = prepare(world, &replacements);
    for _ in retry.surfaces() {
        world.host.push_native_display_presented();
    }
    let outcome = world
        .session
        .present_prepared_mounted_frame_for_reconciliation(
            retry,
            &replacements,
            UiPresentationDeadline::at_tick(u64::MAX),
            accepted_at,
        )
        .unwrap();
    assert!(matches!(outcome, UiMountedFrameOutcome::Reconciled(_)));
    let reconstruction_overrides = world.host.reconstruction_sample_overrides();
    let motion_payloads = reconstruction_overrides[reconstruction_observation_start..]
        .iter()
        .filter(|payload| !payload.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        motion_payloads.len(),
        2,
        "both rejected and accepted physical reconstruction attempts receive Motion overrides"
    );
    for payload in motion_payloads {
        assert_eq!(payload.len(), accepted_samples.len());
        for (command, opacity) in &accepted_samples {
            let override_change = payload
                .iter()
                .find(|change| change.command() == *command)
                .expect("every accepted retained command reaches the host reconstruction boundary");
            // The host receives authored appearance opacity (40_000) composed
            // with the accepted raw Motion sample. The odd denominator cannot
            // tie, so adding half before division gives independent nearest rounding.
            let composed = ((40_000_u64 * u64::from(*opacity) + 32_767) / 65_535) as u16;
            assert_eq!(
                override_change.opacity().units(),
                composed,
                "physical reconstruction receives once-composed appearance and Motion opacity"
            );
        }
    }

    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap();
    assert_eq!(presentation.binding(), replacement);
    for command in moving_commands {
        assert_eq!(
            world
                .session
                .mounted
                .accepted_motion_for_command(presentation, command)
                .unwrap()
                .map(|sample| sample.opacity_units()),
            Some(expected_opacity),
            "exact retained commands keep accepted nonterminal Motion after reconstruction"
        );
    }
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    assert!(output
        .fragments()
        .iter()
        .filter(|fragment| { fragment.surface_binding().semantic_surface() == surface })
        .all(|fragment| fragment.surface_binding().binding() == replacement));
    assert_eq!(reference_pixels(output, surface), accepted_pixels);
    let owner = world
        .session
        .overlay_composition_owners
        .current_for_test(surface)
        .unwrap()
        .current()
        .unwrap();
    assert_ne!(owner.presentation(), accepted_owner.presentation());
    assert_eq!(owner.participants(), accepted_owner.participants());
}

fn prepare(
    world: &mut World,
    replacements: &[UiMountedSurfaceReconciliationBinding],
) -> crate::mounting::UiPreparedMountedFrame {
    let request = world.session.mounted_frame_request();
    world
        .session
        .prepare_mounted_reconstruction_frame_with_application_presentation(
            request,
            replacements,
            |_| {},
        )
        .unwrap_or_else(|_| panic!("mid-Motion authored reconstruction must prepare"))
}

fn reference_pixels(
    output: &UiUnpublishedAppearanceFrameProjection,
    surface: UiSemanticSurfaceIdentity,
) -> ([u8; 4], [u8; 4]) {
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
