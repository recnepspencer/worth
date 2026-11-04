//! Genuine per-writer Native history survives Query checkpoint reopening.
use super::super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
};
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;

#[test]
fn coupled_stage_retirement_reuses_native_writer_custody_after_checkpoint_reopen() {
    type S = Model<30>;
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20, 30]);
    settle(&host, &request, 10);
    let retire = |id, nonce: &str| {
        let outcome = request
            .advanced()
            .mutate(Retire {
                relation_fault: 30,
                corrupt_restoration: false,
                co_anchors: vec![],
                scope: 1,
                id,
                descendants: vec![],
            })
            .without_source()
            .idempotency(&nonce.to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationMutationOutcome::Committed { .. }
            ),
            "{outcome:?}"
        );
    };
    retire(20, "retire-first-stage-before-checkpoint");
    assert!(request.query(query::Input { id: 103 }).execute().is_ok());
    let checkpoint = host
        .with_host_advancement(|phase| {
            host.capture_application_checkpoint(
                &phase,
                WorthQueryCheckpointCapturePolicy::SystemAllocation,
            )
            .unwrap()
        })
        .unwrap();
    drop(request);
    drop(host);
    let host = super::super::coupled::history::reopen::<S>(checkpoint);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    super::super::coupled::producer::handler::reset_contacts();
    // Checkpoint locators retain Native predecessor custody. Optional cached
    // decision facts do not promise cold Ready; the next authored edit must
    // drive the actual cold Preserve directly, without a preliminary demand.
    assert!(request.query(query::Input { id: 103 }).execute().is_ok());
    let outcome = request
        .advanced()
        .mutate(Retire {
            relation_fault: 30,
            corrupt_restoration: false,
            co_anchors: vec![],
            scope: 1,
            id: 30,
            descendants: vec![],
        })
        .without_source()
        .idempotency(&"retire-second-stage-after-checkpoint".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{outcome:?}"
    );
    assert_eq!(
        super::super::coupled::producer::handler::contacts(),
        1,
        "the cold body's genuine Preserve participates"
    );
    for id in [20, 30, 103] {
        assert!(
            request.query(query::Input { id }).execute().is_err(),
            "retired {id}"
        );
    }
    for id in [10, 101, 104] {
        assert!(
            request.query(query::Input { id }).execute().is_ok(),
            "actual surviving body {id}"
        );
    }
    settle(&host, &request, 10);
    assert_eq!(super::super::coupled::producer::handler::contacts(), 1);
}
