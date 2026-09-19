use super::{commit_tick, UiMountedMotionSampler, UiPresentationReducedMotionPosture, World};

#[test]
fn retarget_starts_from_the_exact_committed_raw_motion_units() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    let initial = sampler.install(world.exit_receipt(100)).unwrap();
    assert_eq!(initial.sample().unwrap().opacity_units(), u16::MAX);
    commit_tick(&mut sampler, 1, world.presentation);

    // Halfway through the 110-tick cubic exit, one eighth remains.
    let mid = commit_tick(&mut sampler, 56, world.presentation);
    assert_eq!(mid.samples()[0].opacity_units(), 8_192);
    let appearance = worth_ui_host_contract::UiMountedAppearanceOpacity::from_units(40_000);
    let composed = crate::mounting::presentation::compose_opacity(
        appearance,
        mid.samples()[0].opacity_units(),
    );
    assert_eq!(composed.units(), 5_000);
    drop(sampler.prepare_tick(200, world.presentation).unwrap());

    let retarget = world.receipt(
        101,
        0.0,
        Some(
            crate::runtime::motion::UiMotionRetargetDisposition::Install {
                predecessor:
                    crate::runtime::motion::UiMotionRetargetPredecessor::CurrentPresentationSample,
            },
        ),
    );
    let installed = sampler.install(retarget).unwrap();
    assert_eq!(installed.sample().unwrap().opacity_units(), 8_192);
    let first = commit_tick(&mut sampler, 201, world.presentation);
    assert_eq!(first.samples()[0].opacity_units(), 8_192);

    // The next curve interpolates from 8192 units, not from a rounded u8 alpha
    // or an appearance-composed value: 8192 + (65535 - 8192) * 7/8.
    let mid = commit_tick(&mut sampler, 271, world.presentation);
    assert_eq!(mid.samples()[0].opacity_units(), 58_367);
    assert_eq!(
        crate::mounting::presentation::compose_opacity(
            appearance,
            mid.samples()[0].opacity_units(),
        )
        .units(),
        35_625,
    );
    let terminal = commit_tick(&mut sampler, 341, world.presentation);
    assert_eq!(terminal.samples()[0].opacity_units(), u16::MAX);
}

#[test]
fn reduced_motion_exit_publishes_exact_zero_without_a_float_conversion() {
    let world = World::new();
    let mut sampler = UiMountedMotionSampler::default();
    sampler.set_reduced_motion(UiPresentationReducedMotionPosture::Reduce);
    let terminal = sampler.install(world.exit_receipt(102)).unwrap();
    assert_eq!(terminal.sample().unwrap().opacity_units(), 0);
    assert!(terminal.terminal().is_some());
}
