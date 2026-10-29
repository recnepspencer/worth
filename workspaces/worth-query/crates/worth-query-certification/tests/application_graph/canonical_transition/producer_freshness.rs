//! Query-owned source freshness across genuine intervening cell publication.
use super::{host, model, output, producer, program};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_query_host::facade::application_entry::*;
use worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind;

#[test]
fn aba_requires_current_producer_receipt_and_full_generation_output_lineage() {
    let a = model::authored_value();
    let mut b = a.clone();
    b.title = "intervening".into();
    let application = host::install(a.clone()).unwrap();
    let source_id = host::source_id(&application);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.runtime().request(&principal, &scope);
    let mut initial_demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(initial) =
        initial_demand.settle(&request).unwrap()
    else {
        panic!("the genuine initial producer must settle");
    };
    assert_eq!(initial.producer_contacts_in_this_demand(), 1);
    let initial_receipt = initial.application_commit_receipt().unwrap().clone();
    let output_id = initial
        .outputs_of::<output::DerivedOutputs>()
        .unwrap()
        .entity::<output::CreatedOutput>()
        .unwrap()
        .entity_id();
    assert_ne!(output_id.generation_value(), 0);
    let at_initial = request.retain_current_read().unwrap();
    request
        .at(&at_initial)
        .require_current_output_demand(&initial)
        .unwrap();
    drop(initial_demand);

    // The performed source edit owns required-output custody and settles B's
    // Preserve producer before returning. This is actual cell publication.
    let b_receipt = host::edit_source(&application, b.clone(), 481);
    assert_eq!(host::read(&application), b);
    assert_eq!(
        host::read_derived(&application),
        output::DerivedV1 {
            caption: "INTERVENING".into(),
            rank_sum: 12,
            tags: vec!["left".into(), "right".into()],
        }
    );
    assert!(!b_receipt.is_same_authoritative_commit(&initial_receipt));
    assert_ne!(
        b_receipt.installed_operation(),
        initial_receipt.installed_operation()
    );
    assert_eq!(
        b_receipt
            .outputs_of::<output::PreservedOutputs>()
            .unwrap()
            .entity::<output::PreservedOutput>()
            .unwrap()
            .entity_id(),
        output_id
    );
    let mut b_demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(at_b) =
        b_demand.settle(&request).unwrap()
    else {
        panic!("the already settled B output must be reusable");
    };
    assert_eq!(at_b.producer_contacts_in_this_demand(), 0);
    assert!(at_b
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&b_receipt));
    let b_read = request.retain_current_read().unwrap();
    request
        .at(&b_read)
        .require_current_output_demand(&at_b)
        .unwrap();
    assert!(
        matches!(request.at(&b_read).require_current_output_demand(&initial),
        Err(WorthQueryOutputCurrentnessDenial::Output(cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
    );
    drop(b_demand);

    // Returning to A's parameters must not resurrect A's old producer receipt.
    let a_receipt = host::edit_source(&application, a.clone(), 482);
    assert_eq!(host::source_id(&application), source_id);
    assert_eq!(host::read(&application), a);
    assert_eq!(
        host::read_derived(&application),
        output::DerivedV1 {
            caption: "ASSEMBLY \u{0392}".into(),
            rank_sum: 12,
            tags: vec!["left".into(), "right".into()],
        }
    );
    assert!(!a_receipt.is_same_authoritative_commit(&initial_receipt));
    assert!(!a_receipt.is_same_authoritative_commit(&b_receipt));
    assert_eq!(
        a_receipt.installed_operation(),
        b_receipt.installed_operation()
    );
    assert_eq!(
        a_receipt
            .outputs_of::<output::PreservedOutputs>()
            .unwrap()
            .entity::<output::PreservedOutput>()
            .unwrap()
            .entity_id(),
        output_id
    );
    let mut a_demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(current_a) =
        a_demand.settle(&request).unwrap()
    else {
        panic!("the returned A output must be current");
    };
    assert_eq!(current_a.producer_contacts_in_this_demand(), 0);
    assert!(current_a
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&a_receipt));
    let a_read = request.retain_current_read().unwrap();
    request
        .at(&a_read)
        .require_current_output_demand(&current_a)
        .unwrap();
    for stale in [&initial, &at_b] {
        assert!(
            matches!(request.at(&a_read).require_current_output_demand(stale),
            Err(WorthQueryOutputCurrentnessDenial::Output(cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
        );
    }
    drop(a_demand);

    // Repeat A/B/A with Preserve for ALL three publications. The earlier A
    // above is now the first A of this journey, not the initial Create(A).
    let second_b_receipt = host::edit_source(&application, b.clone(), 483);
    assert_eq!(
        second_b_receipt.installed_operation(),
        a_receipt.installed_operation()
    );
    for earlier in [&initial_receipt, &b_receipt, &a_receipt] {
        assert!(!second_b_receipt.is_same_authoritative_commit(earlier));
    }
    assert_eq!(host::read(&application), b);
    assert_eq!(
        host::read_derived(&application),
        output::DerivedV1 {
            caption: "INTERVENING".into(),
            rank_sum: 12,
            tags: vec!["left".into(), "right".into()],
        }
    );
    assert_eq!(
        second_b_receipt
            .outputs_of::<output::PreservedOutputs>()
            .unwrap()
            .entity::<output::PreservedOutput>()
            .unwrap()
            .entity_id(),
        output_id
    );
    let mut second_b_demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(second_b) =
        second_b_demand.settle(&request).unwrap()
    else {
        panic!("the second B must retain its actual Preserve publication");
    };
    assert_eq!(second_b.producer_contacts_in_this_demand(), 0);
    assert!(second_b
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&second_b_receipt));
    let second_b_read = request.retain_current_read().unwrap();
    request
        .at(&second_b_read)
        .require_current_output_demand(&second_b)
        .unwrap();
    for stale in [&initial, &at_b, &current_a] {
        assert!(
            matches!(request.at(&second_b_read).require_current_output_demand(stale),
            Err(WorthQueryOutputCurrentnessDenial::Output(cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
        );
    }
    drop(second_b_demand);

    let final_receipt = host::edit_source(&application, a.clone(), 484);
    assert_eq!(
        final_receipt.installed_operation(),
        a_receipt.installed_operation()
    );
    assert_eq!(
        final_receipt.installed_operation(),
        second_b_receipt.installed_operation()
    );
    for earlier in [&initial_receipt, &b_receipt, &a_receipt, &second_b_receipt] {
        assert!(!final_receipt.is_same_authoritative_commit(earlier));
    }
    assert_eq!(host::source_id(&application), source_id);
    assert_eq!(host::read(&application), a);
    assert_eq!(
        host::read_derived(&application),
        output::DerivedV1 {
            caption: "ASSEMBLY \u{0392}".into(),
            rank_sum: 12,
            tags: vec!["left".into(), "right".into()],
        }
    );
    assert_eq!(
        final_receipt
            .outputs_of::<output::PreservedOutputs>()
            .unwrap()
            .entity::<output::PreservedOutput>()
            .unwrap()
            .entity_id(),
        output_id
    );
    let mut final_demand = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(final_a) =
        final_demand.settle(&request).unwrap()
    else {
        panic!("the final A must retain a fresh Preserve publication");
    };
    assert_eq!(final_a.producer_contacts_in_this_demand(), 0);
    assert!(final_a
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&final_receipt));
    let final_read = request.retain_current_read().unwrap();
    request
        .at(&final_read)
        .require_current_output_demand(&final_a)
        .unwrap();
    for stale in [&initial, &at_b, &current_a, &second_b] {
        assert!(
            matches!(request.at(&final_read).require_current_output_demand(stale),
            Err(WorthQueryOutputCurrentnessDenial::Output(cause)) if cause.kind() == WorthQueryOutputDemandDenialKind::Superseded)
        );
    }
    drop(final_demand);

    let mut retry = request
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(unchanged) =
        retry.settle(&request).unwrap()
    else {
        panic!("an unchanged retry must settle");
    };
    assert_eq!(unchanged.producer_contacts_in_this_demand(), 0);
    assert!(unchanged
        .application_commit_receipt()
        .unwrap()
        .is_same_authoritative_commit(&final_receipt));
    assert_eq!(
        unchanged
            .outputs_of::<output::PreservedOutputs>()
            .unwrap()
            .entity::<output::PreservedOutput>()
            .unwrap()
            .entity_id(),
        output_id
    );
    let retry_read = request.retain_current_read().unwrap();
    request
        .at(&retry_read)
        .require_current_output_demand(&unchanged)
        .unwrap();
    request
        .at(&retry_read)
        .require_current_output_demand(&final_a)
        .unwrap();
}
