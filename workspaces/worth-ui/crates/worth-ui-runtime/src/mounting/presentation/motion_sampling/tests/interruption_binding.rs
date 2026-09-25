//! An interrupted sample carries into its successor only on its own binding.
use super::*;

#[test]
fn a_retarget_departs_from_an_interrupted_sample_only_on_its_binding() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.install(world.receipt(1, 0.0, None)).unwrap();
    commit_tick(&mut sampler, 70, world.presentation);
    let rebound = World {
        presentation: worth_ui_host_contract::UiHostObservationPresentationBasis::new(
            world.presentation.host_surface(),
            worth_ui_host_contract::UiMountedFrameIdentity::mint_unbound().unwrap(),
            worth_ui_host_contract::UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            worth_ui_host_contract::UiHostPresentationEpoch::issued_by_host(2),
        ),
        ..world
    };
    let retarget = Some(
        crate::runtime::motion::UiMotionRetargetDisposition::Install {
            predecessor:
                crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
        },
    );
    let tracks = |sampler: &UiMountedMotionSampler| {
        let (running, held, _, current, ..) = sampler.certification_observation();
        (running, held, current)
    };
    let before = tracks(&sampler);
    assert!(
        matches!(
            sampler.install(rebound.receipt(2, 100.0, retarget)),
            Err(UiPresentationMotionSamplingDenial::InvalidSampleGeometry(
                UiPresentationGeometrySamplingDenial::PresentationBindingChanged
            ))
        ),
        "a sample drawn on the old binding is not carried onto a new one unannounced"
    );
    assert_eq!(
        tracks(&sampler),
        before,
        "the refused retarget leaves the track running on its current sample"
    );

    sampler.rebind_published_presentation(rebound.target.semantic_surface(), rebound.presentation);
    let installed = sampler
        .install(rebound.receipt(3, 100.0, retarget))
        .expect("once the publication rebinds the track, its sample carries");
    assert_eq!(
        installed.sample().geometry().unwrap().presentation_basis(),
        rebound.presentation
    );
}
