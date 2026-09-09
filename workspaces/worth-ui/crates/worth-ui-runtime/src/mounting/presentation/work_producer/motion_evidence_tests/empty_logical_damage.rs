//! Logical allocation emptiness cannot deny an admitted command's Motion sample.
use super::*;

#[test]
fn opacity_sample_is_issued_for_text_with_only_overhanging_visible_coverage() {
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let layout = crate::mounting::qualified_text_test_support::UiQualifiedTextTestFixture::new()
        .layout("WORTH");
    let projection = world.text_projection(frame, &[world.first_instance], None, layout.view());
    let UiMountedPaintCommand::SemanticText {
        mechanic: original, ..
    } = &projection.authored_paint_commands()[0]
    else {
        panic!("text command");
    };
    let text = UiMountedSemanticTextMechanic::complete_from_runtime_mounting(
        UiMountedSemanticTextCompletionInput {
            content_generation: original.content_generation(),
            frame,
            surface: world.requirement.semantic_surface(),
            binding: world.requirement.binding(),
            mounted_instance: world.first_instance,
            node_receipt: original.node_receipt(),
            allocation_basis: original.allocation_basis(),
            bounds: box_at(0.0, 1.0),
            clip_bounds: box_at(10.0, 80.0),
            origin_x: 0.0,
            origin_y: 32.0,
            text: std::sync::Arc::from("WORTH"),
            layout: layout.view(),
            slot: UiSemanticTextSlot::Value,
            collection_row: None,
            foregrounds: original.foregrounds().to_vec().into(),
            profile: original.profile(),
            layer_semantic_order: original.layer_semantic_order(),
            capability_generation: original.capability_generation(),
            capability_profile_digest: original.capability_profile_digest(),
        },
    )
    .unwrap();
    let command = UiMountedPaintCommand::SemanticText {
        identity: UiMountedPaintCommandIdentity::semantic_text(&text),
        mechanic: text,
    };
    let id = command.identity();
    assert!(
        crate::mounting::presentation::work_producer::command_visible_bounds(&command).is_none()
    );
    let mut current =
        UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    current.commands_by_instance.insert(
        world.first_instance,
        UiMountedPresentationCommandBundle::from_commands(&[command]),
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
            832,
            UiMotionTargetIdentity::from_family_owner(
                world.requirement.semantic_surface(),
                world.first_instance,
                832,
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
    for (tick, units) in [(1, 65_535), (56, 8_192)] {
        let prepared = sampler.prepare_tick(tick, presentation).unwrap();
        let previous = current.motion_for_command(id);
        let (work, acceptance) = current
            .prepare_motion_sample(prepared.receipt(), presentation, &lease)
            .expect("actual native image damage is not bounded by logical allocation");
        let UiMountedPresentationWorkView::Sample(sample) = work.view() else {
            panic!("sample work");
        };
        assert_eq!(sample.changes().len(), 1);
        assert_eq!(sample.changes()[0].command(), id);
        assert_eq!(sample.changes()[0].opacity().units(), units);
        assert!(sample.changes()[0].transform().is_none());
        assert!(
            sample.damage().is_empty(),
            "no fabricated allocation rectangle"
        );
        assert_eq!(
            current.motion_for_command(id),
            previous,
            "preparation does not accept Motion"
        );
        acceptance.accept(&current, presentation).unwrap();
        assert_eq!(
            current
                .motion_for_command(id)
                .flatten()
                .unwrap()
                .opacity_units(),
            units
        );
        sampler.commit_prepared(prepared);
    }
}

fn box_at(x: f32, width: f32) -> UiMountedCanonicalBox {
    UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x,
        y: 32.0,
        width,
        height: 24.0,
        coordinate_space: UiMountedCoordinateSpace::HostSurface,
    })
    .unwrap()
}
