use super::*;

#[test]
fn selective_text_replacement_denies_ambiguous_node_surface_motion() {
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let layout = crate::mounting::qualified_text_test_support::UiQualifiedTextTestFixture::new()
        .layout("WORTH");
    let projection = world.text_projection(frame, &[world.first_instance], None, layout.view());
    let first = projection.authored_paint_commands()[0].clone();
    let second = text_command(
        &first,
        layout.view(),
        UiSemanticTextSlot::Posture,
        [180, 190, 200, 255],
    );
    let commands = [first.clone(), second.clone()];
    let mut accepted =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    accepted.commands_by_instance.insert(
        world.first_instance,
        UiMountedPresentationCommandBundle::from_commands(&commands),
    );
    let presentation = UiHostObservationPresentationBasis::new(
        world.requirement.host_surface(),
        frame,
        world.requirement.binding(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(UiMotionCommitReceipt::for_sampling_test_transition(
            851,
            UiMotionTargetIdentity::from_family_owner(
                world.requirement.semantic_surface(),
                world.first_instance,
                851,
            ),
            presentation,
            None,
            true,
            None,
            false,
            UiMotionDeclaration::portal_exit(),
            None,
        ))
        .unwrap();
    let lease = UiMountedPresentationLeaseGate::default().claim().unwrap();
    let tick = sampler.prepare_tick(56, presentation).unwrap();
    let (_, acceptance) = accepted
        .prepare_motion_sample(tick.receipt(), presentation, &lease)
        .unwrap();
    acceptance.accept(&accepted, presentation).unwrap();
    let retained_opacity = accepted
        .motion_for_command(first.identity())
        .flatten()
        .unwrap()
        .opacity_units();

    let mut successor_bundle = UiMountedPresentationCommandBundle::from_commands(&commands);
    successor_bundle.inherit_unchanged_motion(
        accepted
            .commands_by_instance
            .get(&world.first_instance)
            .unwrap(),
    );
    let replacement = text_command(
        &second,
        layout.view(),
        UiSemanticTextSlot::Posture,
        [1, 2, 3, 255],
    );
    assert!(successor_bundle.replace(replacement.clone()));
    let mut successor =
        UiMountedPresentationState::from_projection(&projection, world.requirement, Some(frame));
    successor
        .commands_by_instance
        .insert(world.first_instance, successor_bundle);
    let candidates = crate::mounting::presentation::coordinator::UiPreparedFrameCandidates::from_single_state_for_test(successor);
    let motion = candidates.accepted_appearance_motion(&[(
        world.requirement.semantic_surface(),
        world.first_instance,
    )]);

    assert_eq!(
        motion.opacity_for(first.identity()).flatten(),
        Some(retained_opacity),
        "the unchanged text command retains its exact accepted sample"
    );
    assert_eq!(motion.opacity_for(replacement.identity()), Some(None));
    assert_eq!(
        motion.opacity_for_instance(world.first_instance, false),
        Err(()),
        "a node-level surface cannot silently choose between divergent text samples"
    );
    assert_eq!(motion.commands_visited(), 2);
}

fn text_command(
    source: &UiMountedPaintCommand,
    layout: UiQualifiedTextLayoutView<'_>,
    slot: UiSemanticTextSlot,
    color: [u8; 4],
) -> UiMountedPaintCommand {
    let UiMountedPaintCommand::SemanticText { mechanic, .. } = source else {
        panic!("text fixture command");
    };
    let foreground = UiMountedTextForegroundSpan::from_runtime_mounting(
        UiTextOriginalRange::from_text_mechanics(0, 5).unwrap(),
        UiMountedRgba8::new(color[0], color[1], color[2], color[3]),
        UiMountedTextPaintSpanIdentity::from_runtime_mounting([slot_ordinal(slot) as u8; 32]),
    );
    let replacement = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: mechanic.content_generation(),
            frame: mechanic.frame(),
            surface: mechanic.surface(),
            binding: mechanic.binding(),
            mounted_instance: mechanic.mounted_instance(),
            node_receipt: mechanic.node_receipt(),
            allocation_basis: mechanic.allocation_basis(),
            bounds: mechanic.bounds(),
            clip_bounds: mechanic.clip_bounds(),
            origin_x: mechanic.origin_x(),
            origin_y: mechanic.origin_y(),
            text: std::sync::Arc::from("WORTH"),
            layout,
            slot,
            collection_row: None,
            foregrounds: std::sync::Arc::from([foreground]),
            profile: mechanic.profile(),
            layer_semantic_order: mechanic.layer_semantic_order(),
            capability_generation: mechanic.capability_generation(),
            capability_profile_digest: mechanic.capability_profile_digest(),
        },
    )
    .unwrap();
    UiMountedPaintCommand::SemanticText {
        identity: UiMountedPaintCommandIdentity::semantic_text(&replacement),
        mechanic: replacement,
    }
}

fn slot_ordinal(slot: UiSemanticTextSlot) -> u16 {
    match slot {
        UiSemanticTextSlot::Value => 1,
        UiSemanticTextSlot::CollectionValue {
            selected_field_ordinal,
        } => selected_field_ordinal.saturating_add(2),
        UiSemanticTextSlot::Posture => u16::MAX,
    }
}
