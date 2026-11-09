//! A recorded creation inverse retires a producer while its retained body cleans up.
use super::super::{
    correction_courts::{install_definition, reverse},
    lifecycle::*,
    *,
};
use super::multi::{create, settle, TwoProgram};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
mod stale_source;
pub(in super::super) mod wire;
type S = Model<72, heterogeneous::Absent, wire::Wire>;

#[test]
fn a_recorded_wire_creation_inverse_publishes_without_the_deleted_producers_aggregates() {
    inverse(false);
}
#[test]
fn a_recorded_wire_creation_inverse_preserves_an_unrelated_live_producers_aggregates() {
    inverse(true);
}
fn inverse(with_neighbor: bool) {
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let body = create(&host, &request, 10, vec![]);
    let outcome = request
        .advanced()
        .mutate(Create {
            create_co: false,
            co_anchors: vec![],
            scope: 10,
            id: 20,
            depth: 1,
        })
        .without_source()
        .idempotency(&"create-wire-source".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed {
        receipt: created, ..
    } = outcome
    else {
        panic!("{outcome:?}")
    };
    settle(&host, &request, 10);
    settle(&host, &request, 20);
    let owner = |receipt: &WorthQueryApplicationCommitReceipt| {
        receipt
            .outputs_of::<CreatedOutputs>()
            .unwrap()
            .entity::<Created<S>>()
            .unwrap()
            .entity_id()
    };
    let memberships = |id| {
        host.runtime()
            .current_owned_aggregate_memberships_for_test(created.product_branch(), id, &scope)
            .unwrap()
    };
    assert_eq!(
        memberships(owner(&created)).len(),
        13,
        "the deleted producer really owns the complete wire roster"
    );
    assert_eq!(
        memberships(owner(&body)).len(),
        13,
        "the retained body has real coupled aggregate support"
    );
    let neighbor = if with_neighbor {
        let neighbor = create(&host, &request, 40, vec![50]);
        settle(&host, &request, 40);
        let before = memberships(owner(&neighbor));
        assert_eq!(before.len(), 13);
        Some((neighbor, before))
    } else {
        None
    };
    super::producer::handler::reset_contacts();
    reverse(&request, &created, "undo-wire-creation");
    assert_eq!(
        super::producer::handler::contacts(),
        1,
        "one retained body performs its own cleanup"
    );
    for id in [20, 201, 202, 102] {
        assert!(
            request.query(query::Input { id }).execute().is_err(),
            "retired record {id} survived"
        );
    }
    for id in [10, 101, 103] {
        assert!(
            request.query(query::Input { id }).execute().is_ok(),
            "retained body record {id} disappeared"
        );
    }
    assert!(
        memberships(owner(&created)).is_empty(),
        "no current aggregate memberships remain for the deleted producer"
    );
    assert!(
        memberships(owner(&body)).is_empty(),
        "the body retired its old wire roster atomically"
    );
    if let Some((neighbor, before)) = neighbor {
        assert_eq!(
            memberships(owner(&neighbor)),
            before,
            "unrelated aggregate membership rows retain their native IDs"
        );
        for id in [40, 50, 401, 402] {
            assert!(request.query(query::Input { id }).execute().is_ok());
        }
    }
}
