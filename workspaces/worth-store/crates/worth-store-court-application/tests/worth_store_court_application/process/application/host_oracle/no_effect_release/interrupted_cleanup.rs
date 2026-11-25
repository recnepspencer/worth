//! Interrupting branch close before Query's obligation discharge WAL append
//! must return retryable `OwnerCleanupPending(ObligationDischarge)`, keep the
//! retired demand visible until retried, and never discharge a subsequently
//! forked live branch's retained demand when L1 cleanup retries.
use super::coupled_flush::{authenticate, fixture, home_request, scope, Home};
use std::num::NonZeroUsize;
use std::path::Path;
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryOriginalOutputDemand,
    WorthQueryPrincipalResolutionMode, WorthQueryProductQueryControls,
};
use worth_query_host::facade::product::{
    WorthQueryApplicationProductBranchCleanupDenial, WorthQueryApplicationProductBranchCloseDenial,
    WorthQueryProductBranch, WorthQueryProductBranchCreateError,
    WorthQueryProductBranchCreationContinuation, WorthQueryProductBranchRecoveryDenial,
};
use worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate;
use worth_store_testkit_application_oracle::facade::host_observation;

fn fork_reuse_exact(
    home: &Home,
    work_scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryProductBranch {
    let app = home.runtime().runtime();
    app.with_application_advancement(work_scope, |phase| {
        let mut recovery = match app
            .branches()
            .fork(app.current_world())
            .components(|components| {
                components
                    .reuse_exact_relational_basis()
                    .reuse_exact_signal_basis()
            })
            .create(&phase)
        {
            Ok(branch) => return branch,
            Err(WorthQueryProductBranchCreateError::ProductUnpublished(recovery)) => recovery,
            Err(cause) => panic!("durable fork creation refused: {cause:?}"),
        };
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
        loop {
            assert!(
                std::time::Instant::now() < deadline,
                "durable fork creation must settle"
            );
            match recovery.continue_in_advancement(&phase) {
                Ok(WorthQueryProductBranchCreationContinuation::Performed(branch)) => {
                    return branch;
                }
                Ok(other) => panic!("unexpected durable fork terminal: {other:?}"),
                Err(failure) => {
                    assert_eq!(
                        failure.denial(),
                        WorthQueryProductBranchRecoveryDenial::Busy
                    );
                    recovery = failure.into_recovery();
                    std::thread::yield_now();
                }
            }
        }
    })
    .unwrap()
}

fn retain_demand_on_branch(
    home: &Home,
    branch: WorthQueryProductBranch,
    principal: &worth_query_host::facade::admission::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<fixture::Catalog>,
    work_scope: &worth_query_host::facade::admission::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryOriginalOutputDemand {
    let app = home.runtime().runtime();
    app.with_application_advancement(work_scope, |phase| {
        let selected = app.on_branch(branch).select(&phase).unwrap();
        let binding = app
            .installed_schema()
            .installed_query_binding::<fixture::SourceBinding>()
            .unwrap();
        let resolved_principal = selected
            .resolve_authenticated_principal(
                &phase,
                binding.principal_binding(),
                principal,
                work_scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let root = selected
            .resolve_entity(
                &phase,
                fixture::ModelKey::at("Key"),
                1u64,
                work_scope,
                WorthQueryPrincipalResolutionMode::Ordinary,
            )
            .unwrap();
        let access = WorthQueryApplicationQueryAccessContext::new(&resolved_principal, &root);
        let plan = selected
            .admit_application_query(
                &phase,
                binding.query(),
                &access,
                ApplicationQueryParameterSet::new(),
                WorthQueryProductQueryControls::new(NonZeroUsize::new(1).unwrap(), work_scope),
            )
            .unwrap();
        let source = app
            .execute_application_query_one_shot(&phase, plan)
            .unwrap()
            .into_admitted_disclosed()
            .into_output_demand_source();
        let demand = app
            .admit_output_demand::<fixture::first::Family>(
                &phase,
                source,
                app.output_demand_resource_profile().limits(),
            )
            .unwrap();
        let retained = demand.retain_for_reopen(app, &phase).unwrap();
        drop(demand);
        retained
    })
    .unwrap()
}

pub(super) fn run_interrupted_cleanup_preserves_live_fork_demand(
    location: &Path,
    home: Home,
    arm: impl Fn() -> CertificationPhysicalMutationPauseGate,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
) {
    let (work_scope, _) = scope();
    let principal = authenticate(&home, &work_scope);
    let fork_1 = fork_reuse_exact(&home, &work_scope);
    let demand_1 = retain_demand_on_branch(&home, fork_1, &principal, &work_scope);
    assert!(home
        .runtime()
        .runtime()
        .is_output_demand_retained(&demand_1));

    // Close fork_1, let World's BranchRetired WAL append (#0) complete, then
    // cancel strictly at BeforeWalAppend of Query's obligation Discharge (#1).
    let (close_scope, close_cancel) = scope();
    let mut gate = arm();
    let failure = std::thread::scope(|threads| {
        let worker = threads.spawn(|| {
            home.runtime()
                .runtime()
                .with_application_advancement(&close_scope, |phase| {
                    home.runtime().runtime().on_branch(fork_1).close(&phase)
                })
                .unwrap()
        });
        assert!(
            gate.await_arrival(),
            "append #0 (BranchRetired) must arrive"
        );
        let next = arm();
        gate.release();
        gate = next;
        assert!(
            gate.await_arrival(),
            "append #1 (Query Discharge) must arrive"
        );
        close_cancel.cancel();
        gate.release();
        match worker.join().expect("close worker did not panic") {
            Err(WorthQueryApplicationProductBranchCloseDenial::OwnerCleanupPending(failure)) => {
                failure
            }
            other => panic!("expected retryable OwnerCleanupPending, got {other:?}"),
        }
    });
    assert!(
        matches!(
            failure.denial(),
            WorthQueryApplicationProductBranchCleanupDenial::ObligationDischarge(_)
        ),
        "interrupted discharge must surface ObligationDischarge: {:?}",
        failure.denial()
    );
    // Settle the cancelled pending WAL frame as ProvenNoEffect so the journal
    // has no unsettled frame while fork_1's demand_1 remains undischarge-pending.
    let (settle_scope, _) = scope();
    let failure = match home
        .runtime()
        .runtime()
        .with_application_advancement(&settle_scope, |phase| {
            failure.into_cleanup().retry_in_advancement(&phase)
        })
        .unwrap()
    {
        Err(next) => {
            assert!(
                matches!(
                    next.denial(),
                    WorthQueryApplicationProductBranchCleanupDenial::ObligationDischarge(_)
                ),
                "settling cancelled discharge must surface retryable ObligationDischarge: {:?}",
                next.denial()
            );
            next
        }
        Ok(receipt) => {
            panic!("cancelled BeforeWalAppend discharge unexpectedly committed: {receipt:?}")
        }
    };
    assert!(
        home.runtime()
            .runtime()
            .is_output_demand_retained(&demand_1),
        "interrupted cleanup leaves demand_1 active until retried"
    );

    // Fork a new live branch while fork_1's cleanup is pending and retain demand_2.
    let fork_2 = fork_reuse_exact(&home, &work_scope);
    let demand_2 = retain_demand_on_branch(&home, fork_2, &principal, &work_scope);
    assert!(home
        .runtime()
        .runtime()
        .is_output_demand_retained(&demand_2));

    // Retry fork_1's pending cleanup: discharges demand_1, leaves demand_2 intact.
    let (retry_scope, _) = scope();
    home.runtime()
        .runtime()
        .with_application_advancement(&retry_scope, |phase| {
            failure.into_cleanup().retry_in_advancement(&phase)
        })
        .unwrap()
        .expect("retried cleanup must finish cleanly");
    assert!(
        !home
            .runtime()
            .runtime()
            .is_output_demand_retained(&demand_1),
        "retried cleanup must durably discharge fork_1's demand"
    );
    assert!(
        home.runtime()
            .runtime()
            .is_output_demand_retained(&demand_2),
        "retried cleanup for fork_1 must not discharge live fork_2's demand"
    );

    drop(principal);
    let closed = home.close(execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();

    // Reopen confirms durable discharge of demand_1 and durable retention of demand_2.
    let reopened = host_observation::opening::open_request(home_request(location), None, execution);
    assert!(!reopened
        .runtime()
        .runtime()
        .is_output_demand_retained(&demand_1));
    assert!(reopened
        .runtime()
        .runtime()
        .is_output_demand_retained(&demand_2));
    let closed = reopened.close(execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
}
