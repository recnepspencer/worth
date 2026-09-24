use super::*;
use crate::mounting::presentation::presented_surface_witness_for_certification;

#[test]
fn exact_physical_change_survives_acceptance_and_reconstruction_without_semantic_rederivation() {
    // The mounted producer boundary deliberately uses a physical translation
    // and clip that cannot be derived from this opacity-only semantic receipt.
    // Scroll group geometry/host pixels are exercised by the Scroll world.
    let world = MountedPresentationWorld::new();
    let frame = UiMountedFrameIdentity::mint_unbound().unwrap();
    let projection = world.projection(frame, [rect_spec(world.first_instance, 0.0)]);
    let current = UiMountedPresentationState::from_projection(&projection, world.requirement, None);
    let command = &projection.authored_paint_commands()[0];
    let identity = command.identity();
    let presentation = UiHostObservationPresentationBasis::new(
        world.requirement.host_surface(),
        frame,
        world.requirement.binding(),
        UiHostPresentationEpoch::issued_by_host(1),
    );
    let mut sampler = UiMountedMotionSampler::default();
    sampler
        .install(UiMotionCommitReceipt::for_sampling_test_transition(
            839,
            UiMotionTargetIdentity::from_mounted_owner(
                world.requirement.semantic_surface(),
                world.first_instance,
                839,
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
    let tick = sampler.prepare_tick(1, presentation).unwrap();
    let sample = tick.receipt().samples()[0];
    assert!(sample.geometry().is_none());
    let source = command.clip_bounds();
    let sampled = UiMountedCanonicalBox::canonicalize(UiMountedCanonicalBoxInput {
        x: source.x(),
        y: source.y() - 3.0,
        width: source.width(),
        height: source.height(),
        coordinate_space: source.coordinate_space(),
    })
    .unwrap();
    let change = UiMountedPresentationSampleChange::from_runtime_scroll_sampling(
        identity,
        UiMountedPresentationTransform::from_runtime_sampling(source, sampled).unwrap(),
        UiMountedPresentationOpacity::from_runtime_appearance_motion(
            UiMountedAppearanceOpacity::ONE,
            sample.opacity_units(),
        ),
        source,
    )
    .unwrap();
    let prepare = || {
        UiPreparedCommandMotionAcceptance::new(vec![
            current.prepare_command_motion_update_with_change(identity, sample, change)
        ])
    };
    drop(prepare());
    assert_eq!(
        current.accepted_motion_change(identity),
        None,
        "dropping rejected host work changes no physical evidence"
    );
    prepare()
        .accept(
            &current,
            &presented_surface_witness_for_certification(presentation),
        )
        .unwrap();
    assert_eq!(current.motion_for_command(identity), Some(Some(sample)));
    assert_eq!(current.accepted_motion_change(identity), Some(change));
    assert_eq!(
        current.appearance_motion_overrides(&[world.first_instance]),
        vec![change]
    );

    let mut rebuilt =
        UiMountedPresentationState::from_projection(&projection, world.requirement, Some(frame));
    rebuilt.inherit_reconstruction_motion(&current);
    assert_eq!(
        rebuilt.reconstruction_motion_overrides(),
        vec![change],
        "cold reconstruction preserves the accepted physical translation and post-transform clip"
    );

    let invalid_change = UiMountedPresentationSampleChange::from_runtime_sampling(
        UiMountedPaintCommandIdentity::appearance_surface(world.first_instance),
        None,
        UiMountedPresentationOpacity::from_runtime_composition(0),
    );
    let invalid = UiPreparedCommandMotionAcceptance::new(vec![
        current.prepare_command_motion_update_with_change(identity, sample, invalid_change)
    ]);
    assert!(matches!(
        invalid.accept(
            &current,
            &presented_surface_witness_for_certification(presentation)
        ),
        Err(UiCommandMotionAcceptanceDenial::SampleBasis)
    ));
    assert_eq!(current.accepted_motion_change(identity), Some(change));
    assert_eq!(
        rebuilt.reconstruction_motion_overrides(),
        vec![change],
        "malformed physical identity cannot change shared accepted evidence"
    );
}
