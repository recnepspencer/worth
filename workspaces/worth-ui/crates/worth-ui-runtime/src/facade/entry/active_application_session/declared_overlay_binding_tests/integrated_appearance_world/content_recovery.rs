use super::{authored, session::World};
use worth_ui_query_binding::WorthUiQueryWorkspaceExt;

const COMPONENT: &str = "platform.pulse.component.projected_status";
const PROJECTION: &str = "platform.pulse.status";

#[test]
fn authored_content_retry_keeps_product_text_and_recovered_occurrence_bindings() {
    let mut world = schema_mismatch_world();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let parent = world.open(0, "overlay.menu", None, 10);
    world.open(1, "overlay.menu", None, 11);
    world.open(2, "overlay.child", Some(parent), 12);
    let frame = world.prepare_surface_with_current_portals(world.surfaces[0]);
    world.publish(frame, 30, false);
    world.sample(world.surfaces[0], 1, 31, 0);
    world.sample(world.surfaces[0], 71, 33, 57_343);
    let predecessor = world.session.active_generation_identity();
    let predecessor_paint = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    world
        .session
        .admit_application_semantic_text(&[
            crate::native_platform::UiNativeComponentSemanticTextChange::successor(
                format!("component:{}", authored::COMPONENTS[0]),
                1,
                "CD",
            )
            .unwrap(),
        ])
        .unwrap();

    let plan = recovery_plan(&mut world, source("status"));
    assert!(matches!(
        plan.semantic_proof(),
        crate::runtime::rebind::UiRebindSemanticProof::AuthoredContent(_)
    ));
    for _ in world.surfaces {
        world.host.push_rejected();
    }
    let prepared = world
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(40),
        )
        .unwrap();
    let prepared_frame = prepared.prepared_frame().unwrap();
    assert!(
        prepared_frame
            .surfaces()
            .iter()
            .map(|surface| surface.projection())
            .any(|projection| projection
                .retained_paint_commands()
                .iter()
                .any(|command| matches!(
                    command,
                    worth_ui_host_contract::UiMountedPaintCommand::SemanticText { mechanic, .. }
                        if mechanic.mounted_instance() == world.instances[0]
                            && mechanic.text() == "CD"
                ))),
        "the prepared rebind carries the product text before the host answers"
    );
    let crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) =
        prepared.execute(40)
    else {
        panic!("scripted rejection must preserve the predecessor");
    };
    let retry = denial
        .detach_retry_for_native()
        .unwrap_or_else(|_| panic!("content retry detaches"));
    assert_eq!(world.session.active_generation_identity(), predecessor);
    assert_eq!(
        world
            .session
            .mounted
            .current_unpublished_appearance()
            .unwrap(),
        predecessor_paint.as_ref()
    );

    let surface = world.surfaces[0];
    let old_binding = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap()
        .binding();
    let before_view = world
        .session
        .mounted
        .current_projection_rc_for_test()
        .unwrap()
        .view_for(old_binding)
        .unwrap();
    let prior_commands = before_view
        .authored_paint_commands()
        .iter()
        .filter(|command| command.identity().mounted_instance() == world.instances[4])
        .cloned()
        .collect::<Vec<_>>();
    super::reconstruction::reconstruct_surface(&mut world);
    let new_binding = world
        .session
        .mounted
        .current_presentation_for_surface(surface)
        .unwrap()
        .binding();
    assert_ne!(
        old_binding, new_binding,
        "recovery must exercise actual host binding succession"
    );
    let recovered_view = world
        .session
        .mounted
        .current_projection_rc_for_test()
        .unwrap()
        .view_for(new_binding)
        .unwrap();
    for old in &prior_commands {
        let new = recovered_view
            .authored_paint_commands()
            .iter()
            .find(|command| command.identity() == old.identity())
            .unwrap_or_else(|| panic!("reconstruction must retain command {:?}", old.identity()));
        let (
            worth_ui_host_contract::UiMountedPaintCommand::SemanticText { mechanic: old, .. },
            worth_ui_host_contract::UiMountedPaintCommand::SemanticText { mechanic: new, .. },
        ) = (old, new)
        else {
            panic!("moving content is authored semantic text")
        };
        assert!(
            old.same_retained_paint_meaning_after_binding_replacement(
                new,
                old_binding,
                new_binding
            ),
            "unmodified moving content retains command meaning across recovery"
        );
    }
    for _ in world.surfaces {
        world.host.push_native_display_settled_without_effects();
    }
    assert!(matches!(
        retry
            .rebase_content_and_retry(&mut world.session, 401)
            .unwrap(),
        crate::runtime::rebind::UiRebindOutcome::Published(_)
    ));
    assert_ne!(world.session.active_generation_identity(), predecessor);
    assert_eq!(
        world
            .session
            .mounted
            .current_presentation_for_surface(surface)
            .unwrap()
            .binding(),
        new_binding
    );
    assert!(
        world
            .session
            .mounted
            .current_surface_viewport(surface)
            .is_some(),
        "accepted content succession must keep geometry on the recovered binding"
    );
    for instance in world.instances {
        assert!(world
            .session
            .mounted
            .has_current_occurrence_geometry(instance));
    }
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    // Transport is a delta: the already accepted neighbor need not resend text.
    // Inspect both exact accepted projection views used by host presentation.
    let accepted = world
        .session
        .mounted
        .current_projection_rc_for_test()
        .unwrap();
    for (surface, instance) in [
        (world.surfaces[0], world.instances[0]),
        (world.surfaces[1], world.instances[3]),
    ] {
        let binding = world
            .session
            .mounted
            .current_presentation_for_surface(surface)
            .unwrap()
            .binding();
        let view = accepted.view_for(binding).unwrap();
        assert!(
            view.semantic_text()
                .rows()
                .iter()
                .any(|text| text.mounted_instance() == instance && text.text() == "CD"),
            "accepted host projection must contain CD at {instance:?}"
        );
    }
    super::assert_portal_backdrop_order(output, &world);
    super::assert_always_region_backdrop(output, &world);
    let content = output
        .fragments()
        .iter()
        .find(|fragment| {
            matches!(fragment.identity(),
        worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
            successor: Some(receipt), ..
        } if receipt.mounted_instance() == world.instances[4])
        })
        .unwrap();
    let opacity = content
        .work()
        .successor()
        .mechanics()
        .iter()
        .map(|mechanic| match mechanic {
            worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(row) => {
                row.opacity().units()
            }
            worth_ui_host_contract::UiMountedAppearanceMechanic::Outline(row) => {
                row.opacity().units()
            }
            worth_ui_host_contract::UiMountedAppearanceMechanic::TextForeground(row) => {
                row.opacity().units()
            }
            _ => panic!("exactly the three content appearance families"),
        })
        .collect::<Vec<_>>();
    assert_eq!(
        opacity,
        vec![35_000; 3],
        "accepted 40000 * 57343 / 65535 composition survives recovery"
    );
    // The UI label is owned independently of the source-governed scalar. Recovery
    // may already publish that admitted label; retry must retain its current value.
    assert!(
        world
            .session
            .presentation
            .project()
            .unwrap()
            .content()
            .is_empty(),
        "acceptance consumes the exact product-text publication revisions"
    );
    let frame = world.prepare();
    assert_eq!(
        frame
            .appearance_selection_cost_report()
            .selected_instance_count(),
        0,
        "the next ordinary frame has no unresolved appearance dependency"
    );
    world.publish(frame, 402, false);
    let _ = world.session.shutdown();
}

fn source(field: &str) -> String {
    format!("{}\ncomponent {COMPONENT} {{ content projection {PROJECTION} }}\n\
        query_scalar {PROJECTION} {{ view {PROJECTION} field {field} require text lifecycle live }}",
        authored::source())
}

fn schema_mismatch_world() -> World {
    let workspace = worth_ui_query_binding::certification::scalar_projection_workspace(true);
    let registration = worth_ui_query_binding::UiScalarProjectionRegistration::text(
        workspace
            .worth_ui()
            .unwrap()
            .projection_view(PROJECTION)
            .unwrap(),
        worth_ui_query_binding::UiProjectionFieldRequirement::query_text_status(),
    );
    let component = crate::runtime::tests::source_ingress_boundary_test_support::
        source_backed_package_component(COMPONENT);
    World::launch_with_projection(
        false,
        false,
        source("revision"),
        Some((component, registration)),
    )
}

fn recovery_plan(
    world: &mut World,
    candidate_source: String,
) -> crate::runtime::rebind::UiRebindPlan {
    let submission =
        crate::runtime::tests::source_ingress_boundary_test_support::lower_file_submission(
            crate::runtime::WorthUiSourceProvider::in_memory("content-recovery")
                .with_file("app/main.wui", candidate_source),
            [crate::runtime::WorthUiWatcherEvent::provider_revision(
                "content-recovery",
            )],
            world.session.capabilities(),
        );
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(submission).unwrap();
    let admitted = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
        world.session.classify_observations(admitted).unwrap()
    else {
        panic!("source schema recovery changes authored meaning");
    };
    let lifecycle = world
        .session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    world
        .session
        .compile_rebind_plan(
            lifecycle,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap()
}

#[test]
fn schema_recovery_with_role_attachment_change_publishes_successor_appearance() {
    let mut world = schema_mismatch_world();
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let candidate = source("status")
        .replacen(
            "cell outside when hover = outside use token(overlay.content.background)",
            "cell outside when hover = outside use token(overlay.content.hovered)",
            1,
        )
        .replace(
            "appearance { role overlay.content }",
            "appearance { role overlay.successor }",
        )
        .replace(
            "appearance role overlay.content applies_to",
            "appearance role overlay.successor applies_to",
        );
    assert!(candidate.contains("appearance { role overlay.successor }"));
    assert!(!candidate.contains("appearance { role overlay.content }"));
    let plan = recovery_plan(&mut world, candidate);
    assert!(
        matches!(
            plan.semantic_proof(),
            crate::runtime::rebind::UiRebindSemanticProof::Changed(_)
        ),
        "content recovery must not erase an appearance attachment change"
    );
    for _ in world.surfaces {
        world.host.push_native_display_settled_without_effects();
    }
    assert!(matches!(
        world
            .session
            .prepare_rebind(
                plan,
                crate::runtime::rebind::UiRebindExecutionRequest::new(2),
            )
            .unwrap()
            .execute(2),
        crate::runtime::rebind::UiRebindOutcome::Published(_)
    ));
    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    for instance in [world.instances[0], world.instances[3]] {
        let fragment = output
            .fragments()
            .iter()
            .find(|fragment| {
                matches!(fragment.identity(),
            worth_ui_host_contract::UiUnpublishedAppearanceFragmentIdentity::NodeReceipt {
                successor: Some(receipt), ..
            } if receipt.mounted_instance() == instance)
            })
            .unwrap();
        assert!(fragment.work().successor().mechanics().iter().any(|mechanic| matches!(mechanic,
            worth_ui_host_contract::UiMountedAppearanceMechanic::Surface(surface)
                if matches!(surface.paint(), worth_ui_host_contract::UiMountedSurfacePaint::FillAndBorder { fill: worth_ui_host_contract::UiMountedSurfaceFill::Solid(fill), .. }
                    if fill.straight_srgba() == [192, 64, 32, 128]))));
    }
    let _ = world.session.shutdown();
}
