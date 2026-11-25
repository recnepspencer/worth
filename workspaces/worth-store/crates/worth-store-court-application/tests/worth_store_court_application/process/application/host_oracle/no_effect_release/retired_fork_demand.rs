//! Retiring a fork that owns a durably retained output demand, then killing the
//! process after World's `BranchRetired` WAL append and before Query discharges
//! the obligation journal, must reopen cleanly and reconcile the retired demand.
use super::coupled_flush::{authenticate, fixture, home_request, scope, Home};
use super::*;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use worth_query_decl::facade::application_query::ApplicationQueryParameterSet;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRequestExt,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationQueryAccessContext, WorthQueryOriginalOutputDemand,
    WorthQueryOutputDemandDenialKind, WorthQueryPrincipalResolutionMode,
    WorthQueryProductQueryControls,
};
use worth_query_host::facade::product::{
    WorthQueryProductBranchCreateError, WorthQueryProductBranchCreationContinuation,
    WorthQueryProductBranchRecoveryDenial,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_store::physical_runtime::certification::CertificationPhysicalMutationPauseGate;
use worth_store_testkit_application_oracle::facade::host_observation;

const LOCATION: &str = "WORTH_HOST_RETIRED_FORK_DEMAND_HOME";
const MARKERS: &str = "WORTH_HOST_RETIRED_FORK_DEMAND_MARKERS";
const CHILD: &str =
    "process::application::host_oracle::no_effect_release::retired_fork_with_retained_demand_child";

pub(super) fn child_env() -> Option<(PathBuf, PathBuf)> {
    let location = std::env::var_os(LOCATION)?;
    let markers = PathBuf::from(std::env::var_os(MARKERS).unwrap());
    Some((PathBuf::from(location), markers))
}

#[test]
fn retiring_fork_with_retained_output_demand_and_killing_before_discharge_reopens_cleanly() {
    let root = tempfile::tempdir().unwrap();
    let location = root.path().join("home");
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[(LOCATION, &location), (MARKERS, root.path())],
            &root.path().join("ready"),
            &root.path().join("finished"),
        ),
        crate::killed_child::ChildEnd::KilledAtPark
    );
    let recorded_bytes = std::fs::read(root.path().join("demand.bin")).unwrap();
    let original = WorthQueryOriginalOutputDemand::from_recorded_bytes(&recorded_bytes).unwrap();

    // Fresh process reopen: World durably retired the fork while the obligation
    // journal still held its retained demand. Reopen must succeed and discharge it.
    let serial = opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let reopened =
        host_observation::opening::open_request(home_request(&location), None, &execution);
    let (reopen_scope, _) = scope();
    let principal = authenticate(&reopened, &reopen_scope);

    // Live root branch still serves reads & mutations cleanly, while the retired
    // fork's original output demand was durably discharged by acquire position
    // (proven independently of branch name) and cannot be readmitted.
    assert!(
        !reopened
            .runtime()
            .runtime()
            .is_output_demand_retained(&original),
        "retired fork demand at acquire position {:?} must be durably discharged on reopen #1",
        original.acquire_position()
    );
    let main_branch = reopened.runtime().runtime().current_world();
    let readmit = reopened
        .runtime()
        .runtime()
        .with_application_advancement(&reopen_scope, |phase| {
            reopened
                .runtime()
                .runtime()
                .readmit_original_output_demand::<fixture::first::Family>(
                    &phase,
                    &principal,
                    &reopen_scope,
                    main_branch,
                    &original,
                )
        })
        .unwrap();
    assert_eq!(
        readmit.err().map(|denial| denial.kind()),
        Some(WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable),
        "retired fork demand must be discharged on reopen"
    );

    let committed = reopened
        .runtime()
        .runtime()
        .request(&principal, &reopen_scope)
        .advanced()
        .mutate(fixture::Edit { data: 9 })
        .without_source()
        .idempotency(&"after-retired-fork-reopen".to_owned())
        .execute_in_program(
            reopened.runtime(),
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(
        matches!(
            committed,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "live main branch must commit cleanly after reopen: {committed:?}"
    );
    drop((committed, principal));
    let closed = reopened
        .close(&execution)
        .unwrap_or_else(|refused| panic!("reopened Home closes cleanly: {refused:?}"));
    assert!(closed.is_closed());
    closed.reclaim();

    // Second reopen confirms durable discharge persists after graceful close.
    let final_home =
        host_observation::opening::open_request(home_request(&location), None, &execution);
    assert!(
        !final_home
            .runtime()
            .runtime()
            .is_output_demand_retained(&original),
        "retired fork demand at acquire position {:?} must remain durably discharged on reopen #2",
        original.acquire_position()
    );
    let closed = final_home.close(&execution).unwrap();
    assert!(closed.is_closed());
    closed.reclaim();
}

pub(super) fn run_child(
    home: Home,
    markers: &Path,
    arm: impl Fn() -> CertificationPhysicalMutationPauseGate,
) {
    let (work_scope, _) = scope();
    let principal = authenticate(&home, &work_scope);
    let app = home.runtime().runtime();

    // 1. Fork a branch from main with exact reuse of BOTH components so the
    //    fork shares its source composite commit with `main`.
    let fork = app
        .with_application_advancement(&work_scope, |phase| {
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
        .unwrap();

    // 2. Verify exact shared commit with `main`, then admit an output demand on
    //    the fork and durably retain it for reopen.
    let original = app
        .with_application_advancement(&work_scope, |phase| {
            let main_selected = app.on_branch(app.current_world()).select(&phase).unwrap();
            let main_commit = main_selected.product().selected_commit().clone();
            drop(main_selected);
            let selected = app.on_branch(fork).select(&phase).unwrap();
            assert_eq!(
                selected.product().selected_commit(),
                &main_commit,
                "reuse_exact fork must share its source composite commit with main"
            );
            let binding = app
                .installed_schema()
                .installed_query_binding::<fixture::SourceBinding>()
                .unwrap();
            let resolved_principal = selected
                .resolve_authenticated_principal(
                    &phase,
                    binding.principal_binding(),
                    &principal,
                    &work_scope,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap();
            let root = selected
                .resolve_entity(
                    &phase,
                    fixture::ModelKey::at("Key"),
                    1u64,
                    &work_scope,
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
                    WorthQueryProductQueryControls::new(NonZeroUsize::new(1).unwrap(), &work_scope),
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
        .unwrap();
    assert!(
        app.is_output_demand_retained(&original),
        "retained demand must be durably active at acquire position {:?} before fork retirement",
        original.acquire_position()
    );
    std::fs::write(
        markers.join("demand.bin"),
        original.to_recorded_bytes().unwrap(),
    )
    .unwrap();

    // 3. Retire the fork, let World's `BranchRetired` WAL append complete, then
    //    park strictly BEFORE Query's obligation discharge WAL append so OS kill
    //    leaves World's retired branch and Query's retained demand on disk.
    let mut gate = arm();
    std::thread::scope(|threads| {
        let worker = threads.spawn(|| -> Result<(), String> {
            app.with_application_advancement(&work_scope, |phase| {
                app.on_branch(fork)
                    .close(&phase)
                    .map(|_| ())
                    .map_err(|e| format!("{e:?}"))
            })
            .map_err(|e| format!("{e:?}"))?
        });
        let deadline = std::time::Instant::now() + crate::killed_child::PARK_DEADLINE;
        // Append #0 is World's durable `BranchRetired` record.
        assert!(crate::process_kill::await_mutation_seam(
            &gate, &worker, deadline
        ));
        let next = arm();
        gate.release();
        gate = next;
        // Append #1 is Query's obligation journal `Discharge` record: hold here and park for kill.
        assert!(crate::process_kill::await_mutation_seam(
            &gate, &worker, deadline
        ));
        crate::killed_child::announce(&markers.join("ready"), b"retired-fork-before-discharge");
        crate::killed_child::park();
    });
}
