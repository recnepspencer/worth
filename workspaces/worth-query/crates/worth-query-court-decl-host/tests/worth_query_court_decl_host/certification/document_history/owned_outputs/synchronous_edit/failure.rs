//! A later synchronous producer refuses the source and the earlier writer too.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

pub(super) fn checkpoint<P: ApplicationProgramDefinition<Schema>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
) -> WorthQueryApplicationCheckpoint {
    host.with_host_advancement(|phase| {
        host.capture_application_checkpoint(
            &phase,
            WorthQueryCheckpointCapturePolicy::SystemAllocation,
        )
        .unwrap()
    })
    .unwrap()
}

fn second_refuses(value: u64) -> u64 {
    if value == 2 {
        u64::MAX
    } else {
        second::derived(value)
    }
}

fn refuses_without_effect(worker: bool) {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::install_configured::<program::Program>(
        limits,
        1,
        None,
        world::Configuration {
            reconciliation_indices: false,
            reconciliation_peer_caps: false,
            reconciliation_emission: false,
            second_derivation: Some(second_refuses),
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let original = (
        world::model(&host, host.current_world()),
        world::outputs(&host, host.current_world()),
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let before = checkpoint(&host);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"refused-synchronous-edit".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("the complete edit must refuse: {outcome:?}")
    };
    let producer = denial
        .coupled_producer_denial()
        .expect("the true producer's typed refusal");
    assert_eq!(
        producer.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert!(
        producer.subject().contains("owned-outputs.second.preserve"),
        "{producer:?}"
    );
    assert_eq!(producer.domain_reason(), Some("DerivedValueOutOfRange"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "failure follows one actual earlier writer"
    );
    assert_eq!(
        (
            world::model(&host, host.current_world()),
            world::outputs(&host, host.current_world())
        ),
        original
    );
    assert_model_unchanged(&host, &before);
}

#[test]
fn a_later_producer_refusal_keeps_the_complete_model_byte_identical() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_without_effect(false);
}
#[test]
fn worker_only_later_producer_refusal_keeps_the_complete_model_byte_identical() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_without_effect(true);
}

pub(super) fn assert_model_unchanged<P: ApplicationProgramDefinition<Schema>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    before: &WorthQueryApplicationCheckpoint,
) {
    let after = checkpoint(host);
    host.with_host_advancement(|phase| {
        let before = host
            .runtime()
            .inspect_application_checkpoint(&phase, before.bytes())
            .unwrap();
        let after = host
            .runtime()
            .inspect_application_checkpoint(&phase, after.bytes())
            .unwrap();
        assert_eq!(
            after.model, before.model,
            "all model and publication value bytes are unchanged"
        );
        assert_eq!(after.live_records, before.live_records);
        assert_eq!(
            after.current_publication_records,
            before.current_publication_records
        );
    })
    .unwrap();
}
