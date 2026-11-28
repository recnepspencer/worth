//! The production Directory Home, actual original operation and current selected admission.
use super::*;
use worth_query_host::facade::runtime::time::{ControlledTime, system_time_sampler};

type Admission =
    WorthQueryAdmittedApplicationOperation<DispatchHomeSchema, RecoverableNotice, u64, Account>;
pub(super) struct Owners {
    pub location: std::path::PathBuf,
    pub handlers: Arc<AtomicUsize>,
    pub now: Arc<ControlledTime>,
}

impl Owners {
    pub fn new() -> Self {
        Self {
            location: dispatch::location(),
            handlers: Arc::new(AtomicUsize::new(0)),
            now: Arc::new(ControlledTime::at_unix_secs(
                system_time_sampler().wall().unwrap().unix_nanos() / 1_000_000_000,
            )),
        }
    }
    pub fn open(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        seed: bool,
        outcomes: &[Outcome],
    ) -> (Arc<Home>, Arc<ScriptedRail>) {
        let request = dispatch::request(
            self.location.clone(),
            ApplicationBuildIdentity::parse(env!("WORTH_IMPLEMENTATION_BUILD_ID")).unwrap(),
            self.handlers.clone(),
            seed,
        )
        .with_authorization_time_source(self.now.clone());
        let home = Arc::new(
            with_bootstrap_advancement_from_request(execution, |phase| {
                request.open_in_advancement(&phase)
            })
            .unwrap()
            .unwrap_or_else(|refused| {
                reopening::opening_failed("recovery claims Directory Home", &refused)
            }),
        );
        let rail = ScriptedRail::answering(outcomes);
        home.runtime()
            .runtime()
            .install_external_effect_transport(rail.clone())
            .unwrap();
        rail.home
            .set(Arc::downgrade(&home))
            .unwrap_or_else(|_| panic!("one actual Home per rail"));
        (home, rail)
    }
}
pub(super) fn emit(home: &Home, key: u64) -> WorthQueryApplicationCommitReceipt {
    let (_, scope) = pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    let outcome = runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(RecoverableNoticeIntent(key))
        .without_source()
        .idempotency(&key)
        .execute_in_program(
            runtime,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Committed { receipt, result } = outcome else {
        panic!("actual recoverable emission commits: {outcome:?}")
    };
    assert_eq!(result, key);
    receipt
}
pub(super) fn admitted_input<R>(
    home: &Home,
    key: u64,
    input: u64,
    body: impl for<'step> FnOnce(
        &WorthQueryAdvancementPhase<'step>,
        &Admission,
        WorthQueryApplicationIdempotencyBinding,
        &worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            worth_query_testkit_host::facade::host::dispatch_home::notify::BalanceValueBinding,
        >,
    ) -> R,
) -> R {
    let (_, scope) = pending_support::scope();
    let runtime = home.runtime();
    let principal = identity::inspection_authentication::authenticate(
        runtime.runtime().installed_schema(),
        &scope,
    );
    runtime
        .request(&principal, &scope)
        .advanced()
        .mutate(RecoverableNoticeIntent(input))
        .without_source()
        .idempotency(&key)
        .with_recovery_in_program(runtime, body)
        .unwrap()
}
pub(super) fn admitted<R>(
    home: &Home,
    key: u64,
    body: impl for<'step> FnOnce(
        &WorthQueryAdvancementPhase<'step>,
        &Admission,
        WorthQueryApplicationIdempotencyBinding,
        &worth_query_decl::facade::application_operation::ApplicationEncodedInput<
            worth_query_testkit_host::facade::host::dispatch_home::notify::BalanceValueBinding,
        >,
    ) -> R,
) -> R {
    admitted_input(home, key, key, body)
}
pub(super) fn status(home: &Home, key: u64) -> WorthQueryRecoveryClaimStatus {
    admitted(home, key, |phase, admission, binding, _| {
        home.runtime()
            .runtime()
            .resolve_admitted_recovery_claim_status(phase, admission, binding)
            .unwrap()
            .expect("actual keyed original")
    })
}
pub(super) fn recover(home: &Home, key: u64) -> WorthQueryRecoveryHandle {
    admitted(home, key, |phase, admission, binding, input| {
        home.runtime()
            .runtime()
            .mint_admitted_recovery_handle(phase, admission, binding, input)
    })
    .unwrap()
}
pub(super) fn mint(
    home: &Home,
    receipt: &WorthQueryApplicationCommitReceipt,
) -> WorthQueryRecoveryHandle {
    home.runtime()
        .runtime()
        .with_host_advancement(|phase| {
            home.runtime()
                .runtime()
                .mint_recovery_handle(&phase, receipt)
        })
        .unwrap()
        .unwrap()
}
pub(super) fn close(home: Arc<Home>, execution: &worth_execution::ExecutionRequest<'_, '_>) {
    Arc::into_inner(home)
        .expect("only the actual Home writer remains")
        .close(execution)
        .unwrap_or_else(|refused| panic!("real recovery custody must retire: {refused:?}"))
        .reclaim();
}
