//! Readable metadata identity across real atlas reservation, settlement and reset.
use super::image_observation::UiNativeTextAtlasImageObservationDenial as Denial;
use super::{
    UiNativeTextAtlas, UiNativeTextAtlasCommitOutcome as Outcome, UiNativeTextAtlasDenial,
    UiNativeTextAtlasExternalOutcome as External,
};

#[test]
fn image_observations_require_the_exact_readable_atlas_generation() {
    let atlas = UiNativeTextAtlas::new();
    let foreign = UiNativeTextAtlas::new();
    let initial = atlas.observe_images().unwrap();
    assert_eq!(atlas.validate_images(initial), Ok(()));
    assert_eq!(foreign.validate_images(initial), Err(Denial::Stale));
    let pending = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert_eq!(atlas.observe_images(), Err(Denial::Reserved));
    assert_eq!(atlas.validate_images(initial), Err(Denial::Reserved));
    drop(pending);
    assert_eq!(atlas.validate_images(initial), Ok(()));
    let rejected = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert!(matches!(
        atlas.settle(rejected, &[], External::Rejected),
        Outcome::Denied(_)
    ));
    assert_eq!(atlas.validate_images(initial), Ok(()));
    let committed = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert!(matches!(
        atlas.settle(committed, &[], External::Submitted),
        Outcome::Committed(_)
    ));
    assert_eq!(atlas.validate_images(initial), Err(Denial::Stale));
    let current = atlas.observe_images().unwrap();
    let uncertain = atlas.plan_demands(&[], &Default::default()).unwrap();
    let Outcome::EffectsIndeterminate(recovery) =
        atlas.settle(uncertain, &[], External::EffectsIndeterminate)
    else {
        panic!("indeterminate atlas must issue recovery");
    };
    assert_eq!(atlas.observe_images(), Err(Denial::Quarantined));
    assert_eq!(atlas.validate_images(current), Err(Denial::Quarantined));
    assert!(atlas.recover(&recovery));
    assert_eq!(atlas.validate_images(current), Err(Denial::Stale));
    assert!(atlas.observe_images().is_ok());
}

#[test]
fn clear_never_revives_an_observation_or_lets_an_old_plan_release_new_work() {
    let atlas = UiNativeTextAtlas::new();
    let initial = atlas.observe_images().unwrap();
    let old = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert!(atlas.clear());
    assert_eq!(atlas.validate_images(initial), Err(Denial::Stale));
    let successor = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert_eq!(
        atlas.settle(old, &[], External::Submitted),
        Outcome::Denied(UiNativeTextAtlasDenial::StalePlan)
    );
    assert_eq!(atlas.observe_images(), Err(Denial::Reserved));
    assert!(matches!(
        atlas.settle(successor, &[], External::Submitted),
        Outcome::Committed(_)
    ));
    let filled = atlas.observe_images().unwrap();
    assert!(atlas.clear());
    let refill = atlas.plan_demands(&[], &Default::default()).unwrap();
    assert!(matches!(
        atlas.settle(refill, &[], External::Submitted),
        Outcome::Committed(_)
    ));
    assert_eq!(atlas.validate_images(filled), Err(Denial::Stale));
}

#[test]
fn clear_generation_exhaustion_preserves_the_live_atlas() {
    let atlas = UiNativeTextAtlas::new();
    atlas.core.borrow_mut().generation = super::UiNativeTextAtlasGeneration::new(u64::MAX).unwrap();
    let before = atlas.snapshot();
    assert!(!atlas.clear());
    assert_eq!(atlas.snapshot(), before);
    #[cfg(feature = "certification-support")]
    assert!(!atlas.can_mutate_for_reconstruction());
}
