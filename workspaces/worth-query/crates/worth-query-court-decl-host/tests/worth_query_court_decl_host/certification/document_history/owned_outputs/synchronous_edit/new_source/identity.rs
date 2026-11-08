//! Descriptive D identity is bound to one actual producer-input qualification.
use super::*;
use worth_query_host::facade::primary_graph::{
    WorthQueryEntityResolutionDenialKind as Kind, WorthQueryEntityResolutionRefusalKind as Refusal,
    WorthQueryPrincipalResolutionMode,
};
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let graph = host
        .runtime()
        .runtime_for_test()
        .primary_graph_for_test()
        .unwrap();
    let installed = graph.retain_entity_resolution_context();
    let recorder = installed.observe_applied_reads_for_test();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "fresh".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"identity-proof".to_owned())
        .execute_in_program(
            &host,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(outcome.receipt().is_some());
    let reads = recorder
        .take_reads_for_test()
        .into_iter()
        .filter(|(entity, _, _)| entity == "Model")
        .collect::<Vec<_>>();
    drop(recorder);
    let (_, before, first) = reads
        .first()
        .expect("first producer resolves the real new Model");
    let (_, _, second) = reads
        .iter()
        .find(|(_, _, candidate)| {
            candidate.candidate_identity().qualification()
                != first.candidate_identity().qualification()
        })
        .expect("a later producer has its own actual input qualification");
    let head = courts::native_head(&host);
    host.with_host_advancement(|phase| {
        let custody = phase.test_host_request_for_test();
        let execution = custody.borrowed_request_for_test();
        let mut identity = installed
            .resolve_applied_field_for_test::<Schema, _>(
                &execution,
                before,
                first,
                ModelKey::at("ModelKey"),
                "fresh".into(),
            )
            .unwrap();
        assert_eq!(
            identity.identity_index_generation(),
            None,
            "an actual D has no committed index generation"
        );
        installed
            .validate_applied_identity_for_test(&execution, before, first, &identity)
            .unwrap();
        let at_second = installed
            .resolve_applied_field_for_test::<Schema, _>(
                &execution,
                before,
                second,
                ModelKey::at("ModelKey"),
                "fresh".into(),
            )
            .unwrap();
        assert_eq!(
            identity.entity_id_for_test(),
            at_second.entity_id_for_test(),
            "the wrong-candidate refusal cannot rely on the entity being absent"
        );
        let denied = installed
            .validate_applied_identity_for_test(&execution, before, second, &identity)
            .expect_err("a live entity at another D is not the selected D");
        assert_eq!(
            denied.kind(),
            Kind::Refused(Refusal::ForeignResolutionTruth)
        );

        let selected = host.on_branch(host.current_world()).select(&phase).unwrap();
        let (_, _, basis) = selected.into_parts_for_test();
        graph
            .integration_handle_for_test()
            .with_runtime_mut_for_test(&execution, |native| {
                let truth = installed
                    .at_snapshot_for_test(
                        native,
                        basis.snapshot_handle(),
                        WorthQueryPrincipalResolutionMode::Ordinary,
                    )
                    .unwrap();
                let denied = truth
                    .validate_entity_freshness_for_test(&execution, &identity)
                    .expect_err(
                        "committing the entity does not convert a D description into C truth",
                    );
                assert_eq!(
                    denied.kind(),
                    Kind::Refused(Refusal::ForeignResolutionTruth)
                );
            });
        let id = identity.entity_id_for_test();
        *identity.entity_id_mut_for_test() = worth_relational::facade::identity::EntityId::new(
            id.partition_id,
            id.local_slot_value(),
            id.generation_value() + 1,
        );
        let denied = installed
            .validate_applied_identity_for_test(&execution, before, first, &identity)
            .expect_err("the exact D must establish full-generation liveness");
        assert_eq!(
            denied.kind(),
            Kind::Refused(Refusal::ForeignResolutionTruth)
        );
    })
    .unwrap();
    assert_eq!(courts::native_head(&host), head);
}
#[test]
fn applied_identity_stays_at_its_exact_candidate_and_generation() {
    run(false);
}
#[test]
fn worker_only_applied_identity_stays_at_its_exact_candidate_and_generation() {
    run(true);
}
