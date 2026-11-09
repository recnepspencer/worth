//! Closed co-anchors compose with surviving coupled writers and fresh restoration.
use super::super::{correction_courts::install_definition, lifecycle::*, *};
use super::multi::{create, settle, TwoProgram};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
fn retire_owner<const MODE: u8>() {
    type S<const MODE: u8> = Model<MODE>;
    let host = install_definition::<S<MODE>, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let created = create(&host, &request, 10, vec![20]);
    if MODE == 20 {
        create(&host, &request, 11, vec![]);
    }
    settle(&host, &request, 10);
    if MODE == 20 {
        settle(&host, &request, 11);
    }
    super::producer::handler::reset_contacts();
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![20],
            scope: 1,
            id: 10,
            descendants: vec![101, 102],
        })
        .without_source()
        .idempotency(&"retire-owner-and-stage".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: retired, ..
    } = outcome
    else {
        panic!("{outcome:?}")
    };
    assert_eq!(
        super::producer::handler::contacts(),
        usize::from(MODE == 20),
        "closed scopes do not execute a producer"
    );
    for id in [10, 20, 101, 102] {
        assert!(
            request.query(query::Input { id }).execute().is_err(),
            "retirement left {id}"
        );
    }
    if MODE == 20 {
        assert!(request.query(query::Input { id: 112 }).execute().is_err());
        for id in [11, 111, 113] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
        settle(&host, &request, 11);
        assert_eq!(super::producer::handler::contacts(), 1);
    }
    super::producer::handler::reset_contacts();
    let restored =
        super::super::correction_courts::reverse(&request, &retired, "restore-owner-and-stage");
    let restored_owner = restored
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S<MODE>>>()
        .unwrap()
        .entity_id();
    let original_owner = created
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S<MODE>>>()
        .unwrap()
        .entity_id();
    assert_ne!(restored_owner, original_owner);
    for id in [10, 20, 101, 102] {
        assert!(
            request.query(query::Input { id }).execute().is_ok(),
            "restore omitted {id}"
        );
    }
    assert_eq!(
        super::producer::handler::contacts(),
        usize::from(MODE == 20),
        "Native restores the closed publication; surviving writers re-realize once"
    );
    if MODE == 20 {
        assert!(request.query(query::Input { id: 113 }).execute().is_err());
        assert!(request.query(query::Input { id: 114 }).execute().is_ok());
        settle(&host, &request, 11);
        assert_eq!(super::producer::handler::contacts(), 1);
    }
    if MODE == 19 {
        let outcome = request
            .advanced()
            .mutate(Retire {
                relation_fault: 0,
                corrupt_restoration: false,
                co_anchors: vec![20],
                scope: 1,
                id: 10,
                descendants: vec![101, 102],
            })
            .without_source()
            .idempotency(&"retire-native-restored-owner".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("{outcome:?}")
        };
        super::super::correction_courts::reverse(
            &request,
            &receipt,
            "restore-native-restored-owner",
        );
        assert_eq!(
            super::producer::handler::contacts(),
            0,
            "Native corrections do not manufacture a producer run"
        );
    }
    // Native correction recreates the closed publication; the new source ID
    // subsequently follows ordinary producer demand and reuse rules.
    let before = super::producer::handler::contacts();
    settle(&host, &request, 10);
    let after = super::producer::handler::contacts();
    assert!(after <= before + 1, "fresh source is realized at most once");
    settle(&host, &request, 10);
    assert_eq!(
        super::producer::handler::contacts(),
        after,
        "a fresh realization is settled"
    );
    if MODE == 19 {
        // The next Native correction transports an actual Preserve antecedent,
        // including its retired role, rather than only an Initial/Create one.
        let outcome = request
            .advanced()
            .mutate(Retire {
                relation_fault: 0,
                corrupt_restoration: false,
                co_anchors: vec![20],
                scope: 1,
                id: 10,
                descendants: vec![101, 103],
            })
            .without_source()
            .idempotency(&"retire-realized-owner-and-stage".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
            panic!("{outcome:?}")
        };
        super::super::correction_courts::reverse(
            &request,
            &receipt,
            "restore-realized-owner-and-stage",
        );
        let before = super::producer::handler::contacts();
        settle(&host, &request, 10);
        assert_eq!(super::producer::handler::contacts(), before + 1);
        assert!(request.query(query::Input { id: 103 }).execute().is_err());
        assert!(request.query(query::Input { id: 104 }).execute().is_ok());
        settle(&host, &request, 10);
        assert_eq!(super::producer::handler::contacts(), before + 1);
    }
}
#[test]
fn a_wholly_retired_owner_uses_native_correction_without_running_declared_closed_roots() {
    retire_owner::<19>();
}
#[test]
fn restored_closed_roots_and_surviving_writers_share_one_real_correction() {
    retire_owner::<20>();
}
