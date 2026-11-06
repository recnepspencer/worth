//! Real coupled writers read and publish Native content across a cold boundary.
use super::super::{
    correction_courts::{install_definition, reverse},
    coupled::{
        history::reopen,
        multi::{create, settle, TwoProgram},
    },
    lifecycle::*,
};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
type S = Model<43, heterogeneous::Absent, sheet_boundary::Absent, true>;
type Host = WorthQueryProgramApplicationRuntime<S, TwoProgram>;

fn retire(
    host: &Host,
    request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    id: u64,
    descendants: Vec<u64>,
    nonce: &str,
) -> WorthQueryApplicationCommitReceipt {
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 0,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id,
            descendants,
        })
        .without_source()
        .idempotency(&nonce.to_owned())
        .execute_in_program(host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, .. } = outcome else {
        panic!("the real content-bearing retirement must commit: {outcome:?}")
    };
    receipt
}
fn checkpoint(host: &Host) -> WorthQueryApplicationCheckpoint {
    host.with_host_advancement(|phase| {
        host.capture_application_checkpoint(
            &phase,
            WorthQueryCheckpointCapturePolicy::SystemAllocation,
        )
        .unwrap()
    })
    .unwrap()
}
#[test]
fn coupled_writers_read_and_publish_their_actual_content_across_checkpoint_and_correction() {
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20, 30]);
    create(&host, &request, 11, vec![]);
    settle(&host, &request, 10);
    settle(&host, &request, 11);
    coupled::producer::handler::reset_contacts();
    retire(&host, &request, 20, vec![], "content-first-stage");
    assert_eq!(coupled::producer::handler::contacts(), 2);
    for id in [103, 113] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    let captured = checkpoint(&host);
    drop(host);

    let host = reopen::<S>(captured);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    coupled::producer::handler::reset_contacts();
    // No preliminary demand: the producer itself decodes each prior writer's
    // actual bytes and compares them with independent logical word expectations.
    let retired = retire(&host, &request, 30, vec![], "content-cold-stage");
    assert_eq!(coupled::producer::handler::contacts(), 2);
    for id in [104, 114] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    reverse(&request, &retired, "content-stage-correction");
    assert_eq!(coupled::producer::handler::contacts(), 4);
    for id in [105, 115] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    settle(&host, &request, 10);
    settle(&host, &request, 11);
    assert_eq!(
        coupled::producer::handler::contacts(),
        4,
        "both content writers settled once"
    );
}

#[test]
fn a_closed_owned_publication_restores_content_and_retains_it_after_checkpoint() {
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let retired = retire(&host, &request, 10, vec![101, 102], "retire-content-owner");
    for id in [10, 101, 102] {
        assert!(request.query(query::Input { id }).execute().is_err());
    }
    // Corrections consume retained performed evidence in the current runtime;
    // checkpoint reopen does not promise to rehydrate that ephemeral authority.
    reverse(&request, &retired, "restore-content-owner");
    let captured = checkpoint(&host);
    drop(host);
    let host = reopen::<S>(captured);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    for id in [10, 101, 102] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
    // This ordinary demand follows real Native restoration, reads the restored
    // prior cell's content, and executes the existing Preserve successor.
    settle(&host, &request, 10);
    assert!(request.query(query::Input { id: 101 }).execute().is_ok());
}
