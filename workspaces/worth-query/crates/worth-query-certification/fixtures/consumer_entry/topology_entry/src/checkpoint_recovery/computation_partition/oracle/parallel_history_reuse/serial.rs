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
