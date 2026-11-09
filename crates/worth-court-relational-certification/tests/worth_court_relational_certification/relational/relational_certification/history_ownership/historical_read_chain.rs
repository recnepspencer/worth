//! Original observations cross every authentic selection, including intermediate ABA.
use super::read_evidence_fixture::{edit, note, SelectedWorld};
use super::*;
use worth_relational::facade::branch::AdmittedRelationalBranchBasis as Basis;
use worth_relational::facade::runtime::RelationalHistoricalSelectionReadDenial as Denial;
use worth_testkit_relational::aspect_storage::aspect_key;

fn select_again(request: &ExecutionRequest<'_, '_>, world: &SelectedWorld) -> (Basis, Basis) {
    let selected = transfer_fixture::basis(&world.runtime);
    let result = transfer_fixture::select(request, &world.runtime, world.model, &selected).unwrap();
    let restored = transfer_fixture::basis(&world.runtime);
    release_test_commit_snapshot(&world.runtime, &result);
    (selected, restored)
}

#[test]
fn a_selection_chain_preserves_the_original_field_and_aspect_observations() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(request);
    let expected = world.revision(request, &world.selected, world.model, &note());
    let aspect = world.aspect(request, &world.selected, world.model);
    edit(
        request,
        &world.runtime,
        world.model,
        "other",
        "an unrelated new future",
    );
    let (selected, restored) = select_again(request, &world);
    let first = (world.selected.observation(), world.restored.observation());
    let second = (selected.observation(), restored.observation());
    let read = world.runtime.read_truth();
    let latest = read
        .read_historical_selection(request, &second.0, &second.1)
        .unwrap();
    assert!(!latest
        .matches_field_revision(request, &world.current(), world.model, &note(), expected)
        .unwrap());
    assert!(!latest
        .matches_aspect_revision(
            request,
            &world.current(),
            world.model,
            &aspect_key("note"),
            aspect
        )
        .unwrap());
    let chain = read
        .read_historical_selection_chain(request, &[(&first.0, &first.1), (&second.0, &second.1)])
        .unwrap();
    assert!(chain
        .matches_field_revision(request, &world.current(), world.model, &note(), expected)
        .unwrap());
    assert!(chain
        .matches_aspect_revision(
            request,
            &world.current(),
            world.model,
            &aspect_key("note"),
            aspect
        )
        .unwrap());
    // A caller may have observed this field after the first selection. Its
    // authentic revision needs only the newest link, not an invented rebasing.
    let intermediate = world.revision(request, &selected, world.model, &note());
    let intermediate_aspect = world.aspect(request, &selected, world.model);
    assert!(chain
        .matches_field_revision(
            request,
            &world.current(),
            world.model,
            &note(),
            intermediate
        )
        .unwrap());
    assert!(chain
        .matches_aspect_revision(
            request,
            &world.current(),
            world.model,
            &aspect_key("note"),
            intermediate_aspect
        )
        .unwrap());
    assert_eq!(chain.selection_commit(), second.1.commit_id().unwrap());
    edit(request, &world.runtime, world.model, "note", "later change");
    assert!(!chain
        .matches_field_revision(request, &world.current(), world.model, &note(), expected)
        .unwrap());
    assert!(!chain
        .matches_aspect_revision(
            request,
            &world.current(),
            world.model,
            &aspect_key("note"),
            aspect
        )
        .unwrap());
}

#[test]
fn an_intermediate_aba_is_not_erased_by_a_later_selection() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(request);
    let expected = world.revision(request, &world.selected, world.model, &note());
    let aspect = world.aspect(request, &world.selected, world.model);
    edit(request, &world.runtime, world.model, "note", "different");
    edit(request, &world.runtime, world.model, "note", "first");
    let (selected, restored) = select_again(request, &world);
    let first = (world.selected.observation(), world.restored.observation());
    let second = (selected.observation(), restored.observation());
    let chain = world
        .runtime
        .read_truth()
        .read_historical_selection_chain(request, &[(&first.0, &first.1), (&second.0, &second.1)])
        .unwrap();
    assert!(!chain
        .matches_field_revision(request, &world.current(), world.model, &note(), expected)
        .unwrap());
    assert!(!chain
        .matches_aspect_revision(
            request,
            &world.current(),
            world.model,
            &aspect_key("note"),
            aspect
        )
        .unwrap());
}

#[test]
fn a_read_chain_refuses_empty_reversed_and_mismatched_selection_links() {
    let host = serial_request();
    let request = &ExecutionRequest::serial(&host);
    let world = SelectedWorld::new(request);
    let (selected, restored) = select_again(request, &world);
    let first = (world.selected.observation(), world.restored.observation());
    let second = (selected.observation(), restored.observation());
    let read = world.runtime.read_truth();
    assert!(matches!(
        read.read_historical_selection_chain(request, &[]),
        Err(Denial::NotGraphSelection { commit: None })
    ));
    assert!(
        matches!(read.read_historical_selection_chain(request, &[(&second.0, &second.1), (&first.0, &first.1)]), Err(Denial::SelectedBasisMismatch { commit }) if commit == first.1.commit_id().unwrap())
    );
    assert!(
        matches!(read.read_historical_selection_chain(request, &[(&first.0, &first.1), (&first.0, &second.1)]), Err(Denial::SelectedBasisMismatch { commit }) if commit == second.1.commit_id().unwrap())
    );
}
