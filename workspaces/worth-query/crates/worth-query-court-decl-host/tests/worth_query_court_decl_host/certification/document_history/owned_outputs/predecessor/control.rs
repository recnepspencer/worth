//! Actual current authentication expiry and commit-lane contention on a Native loan.
use super::courts::{demand_denial, edit_model, reset, settle};
use super::*;
use family::predecessor_family as derived;
#[test]
fn authentication_expiring_during_old_read_prevents_delivering_its_value() {
    use worth_query_host::facade::admission::authenticated_principal::{
        WorthQueryAdmissionTimeController, WorthQueryCancellationSource, WorthQueryRequestScope,
    };
    let controller = std::sync::Arc::new(WorthQueryAdmissionTimeController::new().unwrap());
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
        host_limits(clock.clone()),
        |_, graph, installed| world::seed(graph, installed, 1, None),
        ClockSource(clock.clone()),
    )
    .unwrap();
    reset(false);
    let scope = WorthQueryRequestScope::new(
        None,
        WorthQueryCancellationSource::new().token(),
        clock.clone(),
    );
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    settle(&request, &host).unwrap();
    let document = edit_model(world::attach_with_scope(&host, &scope), &host, &request, 2);
    let selected = document.selection(&request).unwrap();
    let branch = selected.physical_endpoint().product_branch();
    let request = host.request(&principal, &scope).on_branch(branch);
    let advance = controller.clone();
    PROBE.with(|probe| {
        probe.borrow_mut().after_read = Some(Box::new(move || {
            advance
                .advance(std::time::Duration::from_secs(7200))
                .unwrap();
        }))
    });
    let mut demand = request
        .demand(derived::Demand("a".into()))
        .retain_program_outputs::<program::Program, program::Tree>(&host)
        .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
    let mut steps = 0;
    let denial = loop {
        steps += 1;
        assert!(steps <= 128, "expiry refusal must terminate");
        demand = match demand.advance(&request) {
            WorthQueryProgramOutputOperationAdvance::Pending(next) => next,
            WorthQueryProgramOutputOperationAdvance::Refused { denial, .. } => {
                break demand_denial(&denial)
            }
            WorthQueryProgramOutputOperationAdvance::Terminal(terminal) => {
                let denial = demand_denial(terminal.denial());
                let fresh = authenticate_operator(host.installed_schema(), &scope);
                let request = host.request(&fresh, &scope).on_branch(branch);
                terminal
                    .close(&request)
                    .unwrap_or_else(|cause| panic!("{:?}", cause.denial()));
                break denial;
            }
            WorthQueryProgramOutputOperationAdvance::Settled(_) => {
                panic!("expired authentication cannot publish")
            }
        };
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::RequestAuthorization(
            WorthQueryOperationAuthorizationRefusalKind::ExpiredAuthentication
        )
    );
    PROBE.with(|probe| {
        assert!(
            matches!(probe.borrow().values.last(), Some(Err(_))),
            "an old value cannot escape lost C control"
        )
    });
    let fresh = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&fresh, &scope).on_branch(branch);
    assert_eq!(document.selection(&request).unwrap(), selected);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

struct ClockSource(
    worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock,
);
impl WorthQueryRuntimeTimeSource for ClockSource {
    fn admission_clock(
        &self,
    ) -> worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock
    {
        self.0.clone()
    }
    fn current_time(&self) -> Result<std::time::SystemTime, WorthQueryRuntimeTimeSourceDenial> {
        Ok(std::time::SystemTime::now())
    }
}

fn busy(worker: bool) {
    let host = install(worker);
    super::courts::reset(false);
    let lane = host
        .with_host_advancement(|phase| {
            let selected = host.on_branch(host.current_world()).select(&phase).unwrap();
            let (_, product, _) = selected.into_parts_for_test();
            host.runtime()
                .runtime_for_test()
                .primary_provider_for_test()
                .application_branch_commit_lane_for_test(product.observation_for_test())
        })
        .unwrap();
    PROBE.with(|probe| {
        probe.borrow_mut().around_read = Some(Box::new(move |read| {
            let _held = lane.enter_for_test();
            read()
        }))
    });
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let before = host.current_world();
    let denial = settle(&request, &host)
        .expect_err("a borrowed Native read cannot wait for an earlier commit lane");
    assert_eq!(
        denial.kind(),
        WorthQueryOutputDemandDenialKind::CurrentControlBusy
    );
    assert_eq!(
        denial.recovery_posture(),
        WorthQueryOutputDemandRecoveryPosture::Retryable
    );
    assert_eq!(host.current_world(), before);
    settle(&request, &host).unwrap();
    assert_eq!(
        request
            .query(query::first_output::Read("predecessor:a".into()))
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        9001
    );
}
#[test]
fn borrowed_control_contention_is_typed_and_retryable() {
    busy(false);
}
#[test]
fn worker_only_borrowed_control_contention_is_typed_and_retryable() {
    busy(true);
}
