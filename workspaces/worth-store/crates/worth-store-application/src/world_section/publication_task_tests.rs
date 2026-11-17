//! An original World publication cannot be displaced before a proven fate.
use super::{Denial, Endpoint, TaskState};

#[test]
fn unresolved_original_without_physical_handles_refuses_replacement() {
    let state = TaskState {
        endpoint: Some(Endpoint::Unresolved),
        ..TaskState::default()
    };
    assert!(matches!(
        state.replacement_refusal(false),
        Some(Denial::PublicationUnresolved)
    ));
}

#[test]
fn initial_and_driving_originals_remain_owned_until_proven_no_effect() {
    let mut state = TaskState::default();
    assert!(matches!(
        state.replacement_refusal(false),
        Some(Denial::PublicationBusy)
    ));
    state.endpoint = Some(Endpoint::NoEffect);
    assert!(matches!(
        state.replacement_refusal(true),
        Some(Denial::PublicationBusy)
    ));
    assert!(state.replacement_refusal(false).is_none());
    state.invalid = true;
    assert!(matches!(
        state.replacement_refusal(false),
        Some(Denial::PublicationUnresolved)
    ));
}
