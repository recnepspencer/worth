//! Appearance work admits the samples a frame reissues even when it changes
//! no appearance, and is no work only when it carries neither.

use super::{UiMountedAppearancePresentationWork, UiMountedAppearancePresentationWorkDenial};
use crate::*;

struct Attempt {
    frame: UiMountedFrameIdentity,
    presentation: UiMountedPresentationAttemptIdentity,
    requirement: UiMountedSurfaceBindingRequirement,
}

fn attempt() -> Attempt {
    Attempt {
        frame: UiMountedFrameIdentity::mint_unbound().unwrap(),
        presentation: UiMountedPresentationAttemptIdentity::mint_unbound().unwrap(),
        requirement: UiMountedSurfaceBindingRequirement::new(
            UiSemanticSurfaceIdentity::mint_unbound().unwrap(),
            UiHostSurfaceIdentity::mint_unbound().unwrap(),
            UiSurfaceBindingGeneration::mint_unbound().unwrap(),
            WorthUiHostCapabilityObservationGeneration::new(7),
            11,
            UiHostSurfacePresentationMode::RecordOnly,
        ),
    }
}

fn reissued(instance: UiMountedInstanceIdentity) -> UiMountedPresentationSampleChange {
    UiMountedPresentationSampleChange::from_runtime_sampling(
        UiMountedPaintCommandIdentity::appearance_surface(instance),
        None,
        UiMountedPresentationOpacity::from_runtime_composition(u16::MAX),
    )
}

fn sample_work(
    attempt: &Attempt,
    samples: Vec<UiMountedPresentationSampleChange>,
) -> Result<Option<UiMountedAppearancePresentationWork>, UiMountedAppearancePresentationWorkDenial>
{
    UiMountedAppearancePresentationWork::from_runtime_sample_overrides(
        attempt.frame,
        attempt.presentation,
        attempt.requirement,
        samples,
    )
}

#[test]
fn a_frame_that_reissues_no_samples_owes_no_appearance_work() {
    assert!(sample_work(&attempt(), Vec::new()).unwrap().is_none());
}

#[test]
fn a_frame_that_changes_no_appearance_still_carries_the_samples_it_reissues() {
    let attempt = attempt();
    let first = reissued(UiMountedInstanceIdentity::mint_unbound().unwrap());
    let second = reissued(UiMountedInstanceIdentity::mint_unbound().unwrap());
    let work = sample_work(&attempt, vec![first, second])
        .unwrap()
        .expect("reissued samples are work");
    assert!(work.fragments().is_empty());
    assert_eq!(work.sample_overrides(), &[first, second]);
    assert_eq!(work.frame(), attempt.frame);
    assert_eq!(work.presentation(), attempt.presentation);
    assert_eq!(work.requirement(), attempt.requirement);
}

#[test]
fn a_command_is_reissued_at_most_once() {
    let sample = reissued(UiMountedInstanceIdentity::mint_unbound().unwrap());
    assert_eq!(
        sample_work(&attempt(), vec![sample, sample]).err(),
        Some(UiMountedAppearancePresentationWorkDenial::PresentationMismatch)
    );
}
