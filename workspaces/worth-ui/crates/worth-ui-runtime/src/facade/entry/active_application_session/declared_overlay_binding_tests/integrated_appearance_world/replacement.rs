use super::{authored, session::World};

#[path = "replacement/preserved_source.rs"]
mod preserved_source;

#[test]
fn rejected_source_replacement_retries_prepared_relational_overlays_and_motion() {
    replacement(true, 0);
}

#[test]
fn source_replacement_remaps_inserted_portals_surfaces_and_regions_before_retry() {
    replacement(true, 1);
}

#[test]
fn source_replacement_remaps_after_removing_an_unused_earlier_declaration() {
    replacement(false, -1);
}

fn earlier_declaration() -> &'static str {
    r#"
surface aaa.surface {}
portal aaa.earlier {
    surface aaa.surface
    anchor workspace.anchor
    layer modal
    dismiss anchor_gone
    focus first_enabled
    motion system_popover
}
backdrop aaa.scrim {
    scope per_portal_instance aaa.earlier
    extent presented_mosaic_region aaa.surface workspace.region.primary
    presence while portal aaa.earlier presented
    motion none
    place immediately_before portal aaa.earlier
    appearance { role overlay.scrim }
}
"#
}

fn replacement(reject_first: bool, declaration_shift: i8) {
    use worth_ui_host_contract::*;
    let mut world = if declaration_shift < 0 {
        World::launch_with_source(
            false,
            false,
            format!("{}{}", authored::source(), earlier_declaration()),
        )
    } else {
        World::launch()
    };
    let frame = world.prepare();
    world.publish(frame, 1, true);
    let parent = world.open(0, "overlay.menu", None, 10);
    let second = world.open(1, "overlay.menu", None, 11);
    let child = world.open(2, "overlay.child", Some(parent), 12);
    let frame = world.prepare_surface_with_current_portals(world.surfaces[0]);
    world.publish(frame, 30, false);
    world.sample(world.surfaces[0], 1, 31, 0);
    world.sample(world.surfaces[0], 71, 33, 57_343);
    if reject_first && declaration_shift == 0 {
        preserved_source::advance(&mut world);
    }
    let mut source = authored::source()
        .replacen(
            "cell outside when hover = outside use token(overlay.content.background)",
            "cell outside when hover = outside use token(overlay.content.hovered)",
            1,
        )
        .replace("role overlay.content ", "role overlay.successor ");
    if declaration_shift > 0 {
        source.push_str(earlier_declaration());
    }
    let predecessor_declarations = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings()
        .clone();
    // The precursor and replacement belong to consecutive revisions of one provider.
    let submission = crate::runtime::WorthUiReloadDebounce::default()
        .debounce(
            crate::runtime::WorthUiSourceProvider::in_memory("integrated-overlay")
                .with_file("app/main.wui", &source),
            &[crate::runtime::WorthUiWatcherEvent::provider_revision(
                "integrated-overlay",
            )],
            if reject_first && declaration_shift == 0 {
                2
            } else {
                1
            },
        )
        .unwrap()
        .attempt_candidate_for_certification(world.session.application.capabilities())
        .unwrap();
    let mut turn = world.session.begin_observation_turn().unwrap();
    turn.admit_source(submission).unwrap();
    let observations = turn.seal().unwrap();
    let crate::runtime::observation::UiChangeClassificationOutcome::Changed(changed) =
        world.session.classify_observations(observations).unwrap()
    else {
        panic!("an authored role attachment edit must change meaning");
    };
    let lifecycle = world
        .session
        .resolve_affected_scope(changed)
        .unwrap()
        .resolve_identity_lifecycle()
        .unwrap();
    let plan = world
        .session
        .compile_rebind_plan(
            lifecycle,
            crate::runtime::rebind::UiRebindExecutionPolicy::ordinary(),
        )
        .unwrap();
    let predecessor = world.session.active_generation_identity();
    let predecessor_paint = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .cloned();
    let predecessor_bindings = world.session.authored_overlay_binding_exports().unwrap();
    for _ in world.surfaces {
        if reject_first {
            world.host.push_rejected();
        } else {
            world.host.push_native_display_settled_without_effects();
        }
    }
    let retry = match world
        .session
        .prepare_rebind(
            plan,
            crate::runtime::rebind::UiRebindExecutionRequest::new(40),
        )
        .unwrap()
        .execute(40)
    {
        crate::runtime::rebind::UiRebindOutcome::Published(_) if !reject_first => None,
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) if reject_first => {
            assert_eq!(
                denial.cause(),
                crate::runtime::rebind::UiRebindDenialCause::HostRejectedBeforeEffects
            );
            Some(
                denial
                    .detach_retry_for_native()
                    .unwrap_or_else(|_| panic!("retain exact prepared retry")),
            )
        }
        crate::runtime::rebind::UiRebindOutcome::RejectedBeforeEffects(denial) => panic!(
            "replacement denied: {:?}, {:?}",
            denial.cause(),
            denial.host_rejections()
        ),
        _ => panic!("the real source edit must reach its scripted host outcome"),
    };
    if let Some(retry) = retry {
        assert_eq!(world.session.active_generation_identity(), predecessor);
        assert_eq!(
            world
                .session
                .mounted
                .current_unpublished_appearance()
                .unwrap(),
            predecessor_paint.as_ref()
        );
        assert_eq!(
            world.session.authored_overlay_binding_exports().unwrap(),
            predecessor_bindings
        );
        for _ in world.surfaces {
            world.host.push_native_display_settled_without_effects();
        }
        assert!(matches!(
            retry
                .rebase_content_and_retry(&mut world.session, 41)
                .unwrap(),
            crate::runtime::rebind::UiRebindOutcome::Published(_)
        ));
    }
    assert_ne!(world.session.active_generation_identity(), predecessor);
    let declarations = world
        .session
        .application
        .authored_overlay_material()
        .overlay_declaration_bindings();
    let surface = declarations
        .surface_named("workspace.surface.overlay")
        .unwrap();
    let secondary_surface = declarations
        .surface_named("workspace.surface.secondary")
        .unwrap();
    if declaration_shift != 0 {
        for name in ["workspace.surface.overlay", "workspace.surface.secondary"] {
            assert_ne!(
                declarations.region_named(name, "workspace.region.primary"),
                predecessor_declarations.region_named(name, "workspace.region.primary"),
                "the fixture must force real region declaration renumbering"
            );
        }
        assert_ne!(
            Some(surface),
            predecessor_declarations.surface_named("workspace.surface.overlay")
        );
        assert_ne!(
            declarations.portal_named("overlay.menu"),
            predecessor_declarations.portal_named("overlay.menu")
        );
    }
    for (declared, mounted, owner) in world.session.authored_overlay_bindings.bound_owners() {
        assert_eq!(
            declared,
            if mounted == world.surfaces[0] {
                surface
            } else {
                secondary_surface
            }
        );
        for (portal, binding) in owner.bindings() {
            let expected = if portal == child {
                "overlay.child"
            } else {
                assert!(portal == parent || portal == second);
                "overlay.menu"
            };
            assert_eq!(Some(binding), declarations.portal_named(expected));
        }
    }

    let output = world
        .session
        .mounted
        .current_unpublished_appearance()
        .unwrap()
        .unwrap();
    // Oracles describe the product relation and literal mounted region extent;
    // neither is obtained from the successor conversion being tested.
    super::assert_portal_backdrop_order(output, &world);
    super::assert_always_region_backdrop(output, &world);
    let content = output
        .fragments()
        .iter()
        .find(|fragment| {
            matches!(fragment.identity(),
        UiUnpublishedAppearanceFragmentIdentity::NodeReceipt { successor: Some(receipt), .. }
            if receipt.mounted_instance() == world.instances[4])
        })
        .unwrap();
    let opacity = content
        .work()
        .successor()
        .mechanics()
        .iter()
        .map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::Surface(row) => row.opacity().units(),
            UiMountedAppearanceMechanic::Outline(row) => row.opacity().units(),
            UiMountedAppearanceMechanic::TextForeground(row) => row.opacity().units(),
            _ => panic!("the content has exactly three appearance families"),
        })
        .collect::<Vec<_>>();
    // round-even(40000 * 57343 / 65535) = 35000. No u8 conversion,
    // and the source cutover must preserve the previously accepted sample.
    assert_eq!(opacity, vec![35_000; 3]);
    let portal = output
        .fragments()
        .iter()
        .flat_map(|fragment| fragment.work().successor().mechanics())
        .find_map(|mechanic| match mechanic {
            UiMountedAppearanceMechanic::PortalSurface(row)
                if row.surface().node_receipt().mounted_instance() == world.instances[0] =>
            {
                Some(row.surface())
            }
            _ => None,
        })
        .expect("the active Portal owns its surface paint");
    assert!(
        matches!(portal.paint(), UiMountedSurfacePaint::FillAndBorder { fill: worth_ui_host_contract::UiMountedSurfaceFill::Solid(fill), .. }
        if fill.straight_srgba() == [192, 64, 32, 128])
    );
    let _ = world.session.shutdown();
}

pub(super) fn successor_role() -> worth_ui_dsl::UiAppearanceRoleDeclaration {
    use worth_ui_dsl::*;
    let previous = authored::role(0);
    let mut background = UiAppearancePartitionAuthoring::new([UiAppearanceAxisDomain::complete(
        UiAppearanceStateAxis::Hover,
    )]);
    for (name, class) in [
        ("outside", UiAppearanceAxisClass::HoverOutside),
        ("inside", UiAppearanceAxisClass::Hovered),
    ] {
        background = background.with_cell(
            UiAppearanceCell::named(name)
                .when([UiAppearanceAxisPredicate::exact(class)])
                .uses_slot(
                    UiThemeSlotIdentity::new("overlay.content.hovered").unwrap(),
                    UiThemeValueKind::Color,
                ),
        );
    }
    let background = background.compile(UiAppearanceAspect::Background).unwrap();
    UiAppearanceRoleDeclaration::admit(
        UiAppearanceRoleIdentity::new("overlay.successor").unwrap(),
        previous.revision(),
        previous.applicability().clone(),
        previous.aspect_contract(),
        previous.partitions().iter().map(|(aspect, partition)| {
            (
                *aspect,
                if *aspect == UiAppearanceAspect::Background {
                    background.clone()
                } else {
                    partition.clone()
                },
            )
        }),
    )
    .unwrap()
}
