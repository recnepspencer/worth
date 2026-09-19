use super::*;
use crate::mounting::UiMountedSurfaceReconciliationBinding;
use crate::runtime::rebind::{UiRebindReconciliationRequest, UiRebindRecoveryOutcome};

pub(super) fn finish_motion(world: &mut World) {
    assert_accepted_motion(world);
    let surface = world.surfaces[0];
    world.sample(surface, 141, 613, 65_535);
    let final_sample = world.prepare_surface_with_current_portals(surface);
    world.publish(final_sample, 614, false);
}

fn assert_accepted_motion(world: &mut World) {
    let frame = world.prepare_surface_with_current_portals(world.surfaces[0]);
    let commands = frame
        .surfaces()
        .iter()
        .flat_map(|surface| surface.projection().retained_paint_commands().to_vec())
        .filter(|command| command.identity().mounted_instance() == world.instances[4])
        .collect::<Vec<_>>();
    assert!(!commands.is_empty());
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    for command in commands {
        assert_eq!(
            world
                .session
                .mounted
                .accepted_motion_for_command(presentation, command.identity())
                .unwrap()
                .map(|sample| sample.opacity_units()),
            Some(57_343)
        );
    }
}

pub(super) fn recover_predecessor(
    outcome: UiThemeSwitchOutcome<'_>,
    host: &crate::certification_support::ScriptedPresentationHost,
) {
    let UiThemeSwitchOutcome::Indeterminate(recovery) = outcome else {
        panic!("host truth must require recovery")
    };
    let mut reconciliation = recovery.begin_reconciliation();
    let affected = reconciliation.affected_bindings().to_vec();
    assert_eq!(affected.len(), 1);
    let replacements = affected
        .into_iter()
        .map(|binding| {
            let replacement = reconciliation
                .rebind_surface(
                    binding,
                    UiHostSurfacePresentationMode::NativeDisplay,
                    crate::mounting::UiSurfaceBindingProfile::new(
                        1_000,
                        crate::mounting::UiSurfaceBindingCoordinatePosture::LogicalPoints,
                        1,
                    )
                    .unwrap(),
                )
                .unwrap();
            UiMountedSurfaceReconciliationBinding::new(binding, replacement.binding_generation())
        })
        .collect::<Vec<_>>();
    host.push_native_display_presented();
    match reconciliation.present_current(
        UiRebindReconciliationRequest::new(
            replacements.into_boxed_slice(),
            UiPresentationDeadline::at_tick(u64::MAX),
        ),
        611,
    ) {
        UiRebindRecoveryOutcome::Recovered(receipt) => {
            assert!(receipt.predecessor_remains_current())
        }
        UiRebindRecoveryOutcome::RejectedBeforeEffects(denial) => {
            panic!("theme recovery denied: {:?}", denial.cause())
        }
        _ => panic!("theme host recovery must restore accepted predecessor"),
    }
}
