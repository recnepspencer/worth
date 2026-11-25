//! C4b.1: Real OS kills at durable `SynchronizeFileState` I/O barriers of
//! Branch Program Adoption (P0 -> P1), verifying fresh-process reopen,
//! MemoryOnly reference parity, and zero duplicate transitions.
use super::{
    build_id, classify_barrier_kill, run_parallel_barriers, sync_barrier_placement, BarrierKill,
    RestorableHome, BUILD_PRIMARY, TAIL_VARIANCE_BOUND,
};
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_decl::facade::application_program::ApplicationProgramRevision;
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryAdmissionClock,
    application_contribution::with_bootstrap_advancement_from_request,
    application_entry::{
        WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
    },
    application_installation::{WorthQueryApplicationProgramRoster, WorthQueryProgramOwner},
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_document_retention::facade::document_retention_model::{
    actor_generation::validated_actor_program,
    host::{host_limits, seed_host},
    model_lifecycle::validated_model_lifecycle_program,
    operator_identity::{authenticate_operator, request_scope},
    programs::{
        validated_first_program, validated_removed_operation_program, validated_second_program,
        RetentionProgramP0, RetentionProgramP1,
    },
    readback::read_retention,
    schema::DocumentRetentionSchema as Schema,
};
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation,
};

type AdoptionHome = WorthQueryApplicationHome<Schema, RetentionProgramP0>;

#[derive(Debug, PartialEq, Eq)]
struct AdoptionState {
    revision: ApplicationProgramRevision,
    retention: u64,
}

const CHILD: &str =
    "process::application::host_oracle::adversarial::adoption_io_kills::adoption_io_kill_child";
const ENV_HOME: &str = "WORTH_ADV_ADOPT_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_ADOPT_MARKERS";
const ENV_ORDINAL: &str = "WORTH_ADV_ADOPT_ORDINAL";

fn open_adoption_home(
    location: Location,
    seed: bool,
    placement: Option<AdversarialPlacement>,
    execution: ExecutionRequest<'_, '_>,
) -> AdoptionHome {
    let request = WorthQueryApplicationHomeRequest::new(
        location,
        validated_first_program(),
        WorthQueryApplicationProgramRoster::new()
            .support(validated_second_program())
            .support(validated_removed_operation_program())
            .support(validated_actor_program())
            .support(validated_model_lifecycle_program()),
        Schema::declaration().unwrap(),
        ((),),
        build_id(BUILD_PRIMARY),
        host_limits(WorthQueryAdmissionClock::system()),
        move |phase, graph, schema| {
            assert!(seed, "reopen must never re-bootstrap");
            seed_host(phase, graph, schema)
        },
    );
    let request = match placement {
        Some(p) => host_observation::opening::attach_placement(request, p).0,
        None => request,
    };
    *with_bootstrap_advancement_from_request(execution, |phase| {
        Box::new(request.open_in_advancement(&phase).unwrap())
    })
    .unwrap()
}

fn observe_adoption(home: &AdoptionHome) -> AdoptionState {
    let revision = home
        .runtime()
        .with_host_advancement(|phase| {
            *home
                .runtime()
                .runtime()
                .on_branch(home.runtime().current_world())
                .select(&phase)
                .unwrap()
                .inspect_selected_program()
                .unwrap()
                .revision()
        })
        .unwrap();
    let retention = read_retention(home.runtime().runtime(), home.runtime().current_world());
    AdoptionState { revision, retention }
}

fn adopt_to_p1(home: &AdoptionHome) {
    let host = home.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(host.runtime().installed_schema(), &scope);
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .unwrap()
        .owned_revision();
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(host.current_world())
        .programs();
    let req = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&req).unwrap().prepare(64).unwrap();
    assert!(matches!(
        prepared.publish(host.runtime(), &principal, &scope),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
}

fn build_adoption_reference(execution: ExecutionRequest<'_, '_>) -> (AdoptionState, AdoptionState) {
    let home = open_adoption_home(Location::MemoryOnly, true, None, execution);
    let before = observe_adoption(&home);
    adopt_to_p1(&home);
    let after = observe_adoption(&home);
    assert_ne!(before.revision, after.revision);
    drop(home.close(&execution).err().unwrap().resume().unwrap());
    (before, after)
}

#[test]
fn branch_adoption_io_boundary_kill_sweep() {
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = ExecutionRequest::leased(&lease);
    let (expected_before, expected_after) = build_adoption_reference(execution);
    let saved = RestorableHome::prepare(|loc| {
        let home = open_adoption_home(Location::Directory(loc.to_path_buf()), true, None, execution);
        home.close(&execution).unwrap().reclaim();
    });

    let m1 = measure_syncs(&saved);
    let m2 = measure_syncs(&saved);
    assert!(m1.abs_diff(m2) <= TAIL_VARIANCE_BOUND, "adoption calibration spread {m1} vs {m2}");
    let total = m1.max(m2);
    let outcomes = run_parallel_barriers(
        "adoption",
        total,
        || {
            let w_lease = checkpoint_capture::lease(256 << 20);
            let w_exec = ExecutionRequest::leased(&w_lease);
            RestorableHome::prepare(|loc| {
                let home = open_adoption_home(Location::Directory(loc.to_path_buf()), true, None, w_exec);
                home.close(&w_exec).unwrap().reclaim();
            })
        },
        |worker_home, ordinal| {
            let w_lease = checkpoint_capture::lease(256 << 20);
            let w_exec = ExecutionRequest::leased(&w_lease);
            worker_home.restore();
            let markers = tempfile::tempdir().unwrap();
            let ord = PathBuf::from(ordinal.to_string());
            let ready = markers.path().join("ready");
            let end = crate::killed_child::run_and_kill(
                CHILD,
                &[
                    (ENV_HOME, worker_home.location()),
                    (ENV_MARKERS, markers.path()),
                    (ENV_ORDINAL, &ord),
                ],
                &ready,
                &markers.path().join("finished"),
            );
            let kill = classify_barrier_kill(
                end,
                &ready,
                MediaOperationRole::SynchronizeFileState,
                ordinal,
                total,
                &format!("adoption sync#{ordinal} of {total}"),
            );
            let adopted = verify_adoption_reopen(
                worker_home.location(),
                (&expected_before, &expected_after),
                ordinal,
                w_exec,
            );
            if matches!(kill, BarrierKill::CompletedAtTail { .. }) {
                assert!(adopted, "adoption sync#{ordinal}: completed tail run must reopen adopted");
            }
            (kill, adopted)
        },
    );
    let saw_before = outcomes.iter().any(|(_, k, adopted)| *k == BarrierKill::Killed && !adopted);
    let saw_after = outcomes.iter().any(|(_, k, adopted)| *k == BarrierKill::Killed && *adopted);
    assert!(saw_before && saw_after, "adoption sweep must kill both pre-adopt ({saw_before}) and post-adopt ({saw_after})");
}

fn verify_adoption_reopen(
    location: &Path,
    expected: (&AdoptionState, &AdoptionState),
    ordinal: u64,
    execution: ExecutionRequest<'_, '_>,
) -> bool {
    let (expected_before, expected_after) = expected;
    let home = open_adoption_home(Location::Directory(location.to_path_buf()), false, None, execution);
    let handles: Vec<_> = home
        .runtime()
        .runtime()
        .product_publication_recovery_page(None, NonZeroUsize::new(8).unwrap())
        .unwrap()
        .rows()
        .iter()
        .map(|row| row.handle().clone())
        .collect();
    for handle in handles {
        let rec = home.runtime().runtime().readmit_product_publication_recovery(&handle).unwrap();
        loop {
            match rec.continue_original_in_request(&execution).unwrap() {
                WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
                WorldRecoveredContinuationOutcome::Published { .. } => break,
                other => panic!("adoption custody continue: {other:?}"),
            }
        }
    }
    let current = observe_adoption(&home);
    assert!(
        current == *expected_before || current == *expected_after,
        "adoption sync#{ordinal}: state must match MemoryOnly before/after (got {current:?})"
    );
    let already_adopted = current == *expected_after;
    if current == *expected_before {
        adopt_to_p1(&home);
    }
    assert_eq!(observe_adoption(&home), *expected_after, "adoption sync#{ordinal}: final parity");
    home.close(&execution).unwrap().reclaim();
    already_adopted
}

fn measure_syncs(saved: &RestorableHome) -> u64 {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    assert_eq!(
        crate::killed_child::run_and_kill(
            CHILD,
            &[
                (ENV_HOME, saved.location()),
                (ENV_MARKERS, markers.path()),
                (ENV_ORDINAL, Path::new("count")),
            ],
            &markers.path().join("ready"),
            &markers.path().join("finished"),
        ),
        crate::killed_child::ChildEnd::Finished
    );
    std::fs::read_to_string(markers.path().join("finished")).unwrap().trim().parse().unwrap()
}

#[test]
fn adoption_io_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let ord_str = std::env::var(ENV_ORDINAL).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let execution = ExecutionRequest::leased(&lease);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) = if ord_str == "count" {
        sync_barrier_placement(u64::MAX, None, None)
    } else {
        sync_barrier_placement(
            ord_str.parse().unwrap(),
            Some(markers.join("ready")),
            Some(done.clone()),
        )
    };
    let home = open_adoption_home(Location::Directory(location), false, Some(placement), execution);
    activation.arm().unwrap();
    adopt_to_p1(&home);
    done.store(true, Ordering::SeqCst);
    if let Some(h) = watcher {
        let _ = h.join();
        let actual = activation.matching_operation_count();
        crate::killed_child::announce(&markers.join("finished"), format!("missed:actual={actual}").as_bytes());
        return;
    }
    let count = activation.matching_operation_count();
    home.close(&execution).unwrap().reclaim();
    crate::killed_child::announce(&markers.join("finished"), count.to_string().as_bytes());
}
