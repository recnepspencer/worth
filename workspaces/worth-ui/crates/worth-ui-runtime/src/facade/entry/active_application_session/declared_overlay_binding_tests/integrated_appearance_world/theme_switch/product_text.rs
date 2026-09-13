use super::super::session::World;
use worth_ui_host_contract::{UiMountedInstanceIdentity, UiMountedPaintCommand};

pub(super) fn assert_current_ab(world: &World) {
    let presentation = world
        .session
        .mounted
        .current_presentation_for_surface(world.surfaces[0])
        .unwrap();
    let projection = world
        .session
        .mounted
        .current_projection_rc_for_test()
        .unwrap();
    let view = projection.view_for(presentation.binding()).unwrap();
    let commands = view.retained_paint_commands();
    assert_ab(&commands, world);
}

fn assert_ab(commands: &[UiMountedPaintCommand], world: &World) {
    for instance in target_instances(world) {
        assert!(
            commands.iter().any(|command| matches!(command,
                UiMountedPaintCommand::SemanticText { mechanic, .. }
                    if mechanic.mounted_instance() == instance && mechanic.text() == "AB"
            )),
            "the accepted A frame must include exact product text for {instance:?}"
        );
    }
}

fn target_instances(world: &World) -> [UiMountedInstanceIdentity; 3] {
    [world.instances[0], world.instances[1], world.instances[2]]
}
