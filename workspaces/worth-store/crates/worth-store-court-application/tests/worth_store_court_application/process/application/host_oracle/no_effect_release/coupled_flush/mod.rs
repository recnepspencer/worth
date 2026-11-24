//! Interrupting the original coupled write strictly between member 1 and member 2.
pub mod fixture;
use fixture::*;
use std::path::Path;
use std::time::Duration;
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_decl::facade::authentication::WorthQueryExternalPrincipalIdentity;
use worth_query_host::facade::admission::authenticated_principal::{
    admit_authentication_adapter, WorthQueryAdmissionClock,
    WorthQueryAuthenticatedExternalPrincipal, WorthQueryAuthenticationAdapterAdmission,
    WorthQueryAuthenticationAudience, WorthQueryAuthenticationMethod,
    WorthQueryCancellationSource, WorthQueryLocalProcessPrincipal, WorthQueryRequestScope,
};
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationOutputDemandProgress,
    WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::application_installation::{
    WorthQueryApplicationLimits, WorthQueryApplicationProgramRoster,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_host::facade::{
    WorthQueryApplicationHome, WorthQueryApplicationHomeCheckpointDenial,
    WorthQueryApplicationHomeLocation, WorthQueryApplicationHomeRequest,
};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate;
use worth_store_testkit_application_oracle::facade::host_observation;

pub type Home = WorthQueryApplicationHome<Catalog, Program>;

pub fn home_request(location: &Path) -> WorthQueryApplicationHomeRequest<'static, Catalog, Program> {
    let declaration = Catalog::declaration().unwrap();
    let program = ApplicationProgramAuthoring::<Catalog, Program>::begin()
        .with_schema(&declaration)
        .validated_program()
        .unwrap();
    WorthQueryApplicationHomeRequest::new(
        WorthQueryApplicationHomeLocation::Directory(location.to_path_buf()),
        program,
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        ((),),
        super::super::opening::build(),
        WorthQueryApplicationLimits::ordinary_serial(WorthQueryAdmissionClock::system()),
        |_, bootstrap, installed| seed_principal(bootstrap, installed),
    )
}

fn scope() -> (WorthQueryRequestScope, WorthQueryCancellationSource) {
    let cancellation = WorthQueryCancellationSource::new();
    let scope = WorthQueryRequestScope::new(
        None,
        cancellation.token(),
        WorthQueryAdmissionClock::system(),
    );
    (scope, cancellation)
}

fn authenticate(
    home: &Home,
    scope: &WorthQueryRequestScope,
) -> WorthQueryAuthenticatedExternalPrincipal<Catalog> {
    let audience = WorthQueryAuthenticationAudience::new("coupled-flush").unwrap();
    let method = WorthQueryAuthenticationMethod::new("local-process").unwrap();
    let adapter = admit_authentication_adapter(
        home.runtime().runtime().installed_schema(),
        WorthQueryAuthenticationAdapterAdmission::new(
            audience.clone(),
            method.clone(),
            WorthQueryAdmissionClock::system(),
        ),
        WorthQueryLocalProcessPrincipal::new(
            "coupled-flush.local.v1",
            WorthQueryExternalPrincipalIdentity::new("coupled-flush", "reader").unwrap(),
            audience,
            method,
            Duration::from_secs(600),
        )
        .unwrap(),
    )
    .unwrap();
    adapter.authenticate_local(scope).unwrap()
}

/// Runs a real 2-coupled-member `execute_in_program` write on a Directory Home,
/// lets World open, Native commit, World occurrence publish, and coupled
/// member 1's obligation write complete (appends 0..7), then cancels the
/// ORIGINAL request at `BeforeWalAppend` of coupled member 2's obligation write
/// (append #7). Member 1 is already durable and popped, while member 2 stays in
/// retryable owner custody until flushed, checkpointed, closed, and reopened.
pub fn run_original_write_interrupted_between_members(
    location: &Path,
    home: Home,
    arm_pause: impl Fn() -> CertificationPhysicalMutationPauseGate,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let (write_scope, cancellation) = scope();
    let principal = authenticate(&home, &write_scope);
    let mut gate = arm_pause();
    let outcome = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            let request = home.runtime().runtime().request(&principal, &write_scope);
            request
                .advanced()
                .mutate(Edit { data: 2 })
                .without_source()
                .idempotency(&"original-write-interrupted-between-members".to_owned())
                .execute_in_program(home.runtime(), ExecutionAllocationPolicy::SystemAllocation)
        });
        // Skip the 7 WAL appends before coupled member 2's obligation write
        // (World open, Native commit frames, World occurrence publish, and
        // coupled member 1's obligation write #6).
        for step in 0..7 {
            if !gate.await_arrival() {
                drop(gate);
                let early = worker.join().expect("worker did not panic");
                panic!(
                    "expected WAL append #{step} before member 2 obligation append; worker ended early with: {early:?}"
                );
            }
            let next = arm_pause();
            gate.release();
            gate = next;
        }
        // Pause strictly at coupled member 2's obligation WAL append (#7)
        // during the original request's own flush, then cancel before releasing
        // so member 1 has already persisted while member 2 stays pending.
        if !gate.await_arrival() {
            drop(gate);
            let early = worker.join().expect("worker did not panic");
            panic!(
                "expected coupled member 2 obligation WAL append (#7) on original write; worker ended early with: {early:?}"
            );
        }
        cancellation.cancel();
        gate.release();
        worker.join().expect("original coupled write completed")
    })
    .unwrap();
    assert!(
        matches!(outcome, WorthQueryApplicationMutationOutcome::Committed { .. }),
        "original coupled write committed in World before member 2 flush stopped: {outcome:?}"
    );
    drop((outcome, principal));

    // Member 2 remains pending in retryable owner custody, so cancelled
    // checkpoint and close must refuse.
    let cancelled = worth_execution::CancellationSource::new();
    cancelled.cancel();
    let cancelled_lease = worth_execution::ExecutionResourceLease::serial(
        worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(256 << 20),
                    None,
                ),
            ),
            cancellation: cancelled.token(),
            deadline: None,
        },
    )
    .unwrap();
    let cancelled_exec = worth_execution::ExecutionRequest::leased(&cancelled_lease);
    assert!(
        matches!(
            home.checkpoint(&cancelled_exec),
            Err(WorthQueryApplicationHomeCheckpointDenial::CoupledOutputRetention(_))
        ),
        "checkpoint must refuse while coupled member 2 remains pending"
    );
    let refused = home
        .close(&cancelled_exec)
        .err()
        .expect("cancelled close must refuse while coupled member 2 remains pending");
    let home = refused
        .resume()
        .unwrap_or_else(|r| panic!("pre-seal close refusal must resume cleanly: {r:?}"));

    // A 1-unit work ceiling flush completes cleanly, proving the ORIGINAL write
    // persisted member 1 and left EXACTLY member 2 (1 member) pending.
    let one_unit_lease = worth_execution::ExecutionResourceLease::serial(
        worth_execution::LeaseRequest {
            policy: worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    std::num::NonZeroUsize::MIN,
                    Some(256 << 20),
                    Some(1),
                ),
            ),
            cancellation: worth_execution::CancellationToken::new(),
            deadline: None,
        },
    )
    .unwrap();
    let one_unit_exec = worth_execution::ExecutionRequest::leased(&one_unit_lease);
    home.runtime()
        .flush_pending_coupled_outputs(&one_unit_exec)
        .unwrap_or_else(|denial| {
            panic!("exactly one remaining coupled member must fit in 1 work unit: {denial:?}")
        });

    home.checkpoint(execution)
        .unwrap_or_else(|d| panic!("checkpoint after flushing member 2 must succeed: {d:?}"));
    let closed = home
        .close(execution)
        .unwrap_or_else(|r| panic!("close after flushing member 2 must succeed: {r:?}"));
    assert!(closed.is_closed());
    closed.reclaim();

    // Reopen the Directory Home and verify BOTH coupled members settle with 0 producer contacts.
    let reopened = host_observation::opening::open_request(home_request(location), None, execution);
    let (reopen_scope, _) = scope();
    let reopen_principal = authenticate(&reopened, &reopen_scope);
    let request = reopened.runtime().runtime().request(&reopen_principal, &reopen_scope);
    let mut first_demand = request
        .demand(first::Demand)
        .start_in_program::<Program, FirstTree>(reopened.runtime())
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(first_settled) =
        first_demand.settle(&request).unwrap()
    else {
        panic!("reopened first coupled output must settle");
    };
    assert_eq!(first_settled.producer_contacts_in_this_demand(), 0);
    drop((first_settled, first_demand));
    let mut second_demand = request
        .demand(second::Demand)
        .start_in_program::<Program, SecondTree>(reopened.runtime())
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(second_settled) =
        second_demand.settle(&request).unwrap()
    else {
        panic!("reopened second coupled output must settle");
    };
    assert_eq!(second_settled.producer_contacts_in_this_demand(), 0);
    drop((second_settled, second_demand, reopen_principal));
    let closed = reopened.close(execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
}
