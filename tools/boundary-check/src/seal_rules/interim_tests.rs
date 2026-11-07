//! The interim door has one independently frozen external reach inventory.
use super::*;
#[test]
fn interim_restore_requires_its_frozen_caller_inventory() {
    let caller = "workspaces/worth-query/crates/worth-query-testkit-host/src/checkpoint.rs";
    let reference = "WorthQueryInterimCheckpointRestore::program(checkpoint);";
    let mut input = Inputs::fixture(&[(caller, reference)], "");
    input.files.insert(
        format!("{CONFIG}certification-construction-callers.txt"),
        format!("query-interim-checkpoint {caller}\n"),
    );
    assert_eq!(
        check(&input).len(),
        1,
        "another factory's caller file cannot admit the interim door"
    );
    input.files.insert(
        format!("{CONFIG}interim-checkpoint-callers.txt"),
        format!("query-interim-checkpoint {caller}\n"),
    );
    assert!(check(&input).is_empty());
    let foreign = "workspaces/worth-query/crates/worth-query-testkit-host/src/other.rs";
    input.files.insert(foreign.into(), reference.into());
    let denied = check(&input);
    assert_eq!(denied.len(), 1);
    assert_eq!(denied[0].subject, foreign);
}
