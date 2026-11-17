use super::*;
#[test]
fn missing_reuse_inputs_are_executed_and_observed() {
    for change in [
        Change::EmptyBefore,
        Change::EmptyAfter,
        Change::EmptyStable,
        Change::CodeIdentity,
    ] {
        case(change);
    }
}

#[test]
fn gather_facts() {
    case(Change::Gather)
}

#[test]
fn shared_facts() {
    case(Change::Shared)
}

#[test]
fn key_facts() {
    case(Change::Key)
}

#[test]
fn canonical_input() {
    case(Change::Input)
}

#[test]
fn faults() {
    case(Change::Fault)
}

#[test]
fn membership_join() {
    case(Change::Join)
}

#[test]
fn membership_leave() {
    case(Change::Leave)
}

#[test]
fn persisting_item_digest_swap() {
    case(Change::DigestSwap)
}

#[test]
fn membership_replacement() {
    case(Change::Replacement)
}

#[test]
fn authority_encoding_boundary_keeps_declared_compute_charge() {
    let _guard = checkpoint_recovery_test_guard();
    let model = facts();
    // 128 legitimate bootstrap issuances force the runtime's next authority
    // past the one-byte boundary even if earlier tests already advanced it.
    let application =
        execution::install_after_authority_issuances(|graph| seed(&model, graph), 128);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let expected = declaration::computed_work(&model);
    let (actual, charged) = only_run(demand(&request, &application));
    assert_eq!(actual.outcome.unwrap().0, model.expected_value().unwrap());
    assert_eq!(
        charged,
        Some(expected),
        "declared compute and tree charge is independent of authority encoding width"
    );
}

#[test]
fn ceiling_inside_carried_work() {
    case(Change::CarriedCeiling);
}

#[test]
fn synchronous_input_change_uses_current_workers_through_delivery() {
    case_at(Change::Input, 1, 2, true);
}

#[test]
fn same_input_source_edit_carries_unchanged_partitions_in_the_candidate() {
    let _guard = checkpoint_recovery_test_guard();
    let model = facts();
    let application = install(|graph| seed(&model, graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (first, charge) = only_run(demand(&request, &application));
    assert_eq!(charge, Some(declaration::computed_work(&model)));
    let (carried, report) = only_edit_run(execution::adjust_and_demand(
        &request,
        &application,
        EVEN_Y + 2,
        1,
    ));
    assert!(
        matches!(carried.runs.as_slice(), [Run::Incremental { .. }]),
        "expected one incremental run, got {:?}",
        carried.runs
    );
    assert_eq!(
        carried.calls.kernels, 0,
        "unchanged calls are carried, never recomputed"
    );
    assert!(
        !model.members().collect::<Vec<_>>().is_empty(),
        "the carried set is real"
    );
    assert_eq!(
        carried.outcome.as_ref().unwrap().0,
        model.expected_value().unwrap()
    );
    assert_eq!(
        carried.outcome.as_ref().unwrap().1,
        first.outcome.as_ref().unwrap().1,
        "carried work replays the real full-run charge"
    );
    assert_eq!(
        report, None,
        "an incremental run does not invent a full-run report"
    );
    super::super::assert_published_state(&[first], &[carried]);
}
