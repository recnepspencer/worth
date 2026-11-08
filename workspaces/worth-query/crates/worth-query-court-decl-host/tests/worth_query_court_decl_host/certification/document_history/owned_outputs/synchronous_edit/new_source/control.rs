//! C authentication expiry during a genuine D lookup keeps its typed cause.
use super::*;
use worth_query_host::facade::{
    admission::authenticated_principal::{
        WorthQueryAdmissionClock, WorthQueryAdmissionTimeController, WorthQueryCancellationSource,
        WorthQueryRequestScope,
    },
    runtime::ExecutionAllocationPolicy,
};
struct TimeSource(WorthQueryAdmissionClock);
impl WorthQueryRuntimeTimeSource for TimeSource {
    fn admission_clock(&self) -> WorthQueryAdmissionClock {
        self.0.clone()
    }
    fn current_time(&self) -> Result<std::time::SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(std::time::SystemTime::now())
    }
}
fn run(worker: bool) {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let controller = WorthQueryAdmissionTimeController::new().unwrap();
    controller.hold();
    let clock = controller.clock();
    let schema = Schema::declaration().unwrap();
    let host = in_memory_program_with_authorization_time_source(
        ApplicationProgramAuthoring::<Schema, program::Program>::begin()
            .with_schema(&schema)
            .validated_program()
            .unwrap(),
        schema,
        (world::Configuration::default(),),
        ApplicationBuildIdentity::parse("91830000-0000-0000-0000-000000005356").unwrap(),
        if worker {
            world::worker_limits_with_clock(clock.clone())
        } else {
            host_limits(clock.clone())
        },
        |_, graph, installed| world::seed(graph, installed, 1, None),
        TimeSource(clock.clone()),
    )
    .unwrap();
    let scope =
        WorthQueryRequestScope::new(None, WorthQueryCancellationSource::new().token(), clock);
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = courts::native_head(&host);
    let checkpoint = super::super::failure::checkpoint(&host);
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let control = host
        .runtime()
        .runtime_for_test()
        .primary_graph_for_test()
        .unwrap()
        .retain_entity_resolution_context()
        .advance_before_applied_lookup_for_test(controller, std::time::Duration::from_secs(7200));
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "fresh".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"expired-new-source".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        control.was_triggered_for_test(),
        "expiry occurs at the actual private D lookup"
    );
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("expired authentication cannot publish: {outcome:?}")
    };
    let producer = denial
        .coupled_producer_denial()
        .expect("the true source admission cause");
    assert_eq!(
        producer.kind(),
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            WorthQueryOperationAuthorizationRefusalKind::ExpiredAuthentication
        ),
        "{producer:?}"
    );
    assert!(
        producer.subject().contains("owned-outputs.first.initial"),
        "{producer:?}"
    );
    assert_eq!(host.producer_contacts_on_this_thread_for_test(), contacts);
    assert_eq!(courts::native_head(&host), before);
    super::super::failure::assert_model_unchanged(&host, &checkpoint);
}
#[test]
fn current_authentication_expiring_at_applied_lookup_refuses_without_effect() {
    run(false);
}
#[test]
fn worker_only_current_authentication_expiring_at_applied_lookup_refuses_without_effect() {
    run(true);
}
