//! C4b.1: Real OS kills at every durable `SynchronizeFileState` I/O barrier of
//! a mixed Native + Signal + Bridge + World publication, verifying fresh-process
//! reopen, custody continuation, admitted-key retry, and MemoryOnly parity.
use super::super::opening;
use super::{
    assert_all_reopen_phases_killed, build_id, classify_barrier_kill, run_parallel_barriers,
    sync_barrier_placement, BarrierKill, ReopenPhase, RestorableHome, BUILD_PRIMARY,
    TAIL_VARIANCE_BOUND,
};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    application_contribution::with_bootstrap_advancement_from_request,
    application_installation::WorthQueryApplicationProgramRoster,
    primary_graph::{
        WorthQueryApplicationCommitOutcome as CommitOutcome,
        WorthQueryConditionalDefinitionPublicationOutcome,
    },
    product::WorthQueryProductBranch,
    runtime::RuntimeWorldCancellationSource,
    WorthQueryApplicationHome, WorthQueryApplicationHomeLocation as Location,
    WorthQueryApplicationHomeRequest,
};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::{
    contribution_installation::*, world::program_amendment::commit_definition_on_program_lane,
};
use worth_runtime_world::facade::WorldRecoveredContinuationOutcome;
use worth_store::physical_runtime::certification::MediaOperationRole;
use worth_store_testkit_application_oracle::facade::{
    host_interruption::opening::AdversarialPlacement, host_observation,
};

type SignalHome = WorthQueryApplicationHome<TemporalHostSchema, TemporalInstallationProgram>;

#[derive(Debug, Clone, PartialEq, Eq)]
struct StableNativeBasis {
    branch_id: worth_relational::facade::history::BranchId,
    truth_version: worth_relational::facade::branch::RelationalBranchVersion,
    schema_commitment: [u8; 32],
    posture: worth_relational::facade::branch::RelationalBranchBasisPosture,
}
#[derive(Debug, PartialEq, Eq)]
struct SignalSnapshot {
    intent: (String, u64, String, String),
    world_commit_ordinal: u64,
    native_basis: StableNativeBasis,
}

const CHILD: &str =
    "process::application::host_oracle::adversarial::signal_bridge_io_kills::signal_bridge_io_kill_child";
const ENV_HOME: &str = "WORTH_ADV_SIGNAL_HOME";
const ENV_MARKERS: &str = "WORTH_ADV_SIGNAL_MARKERS";
const ENV_ORDINAL: &str = "WORTH_ADV_SIGNAL_ORDINAL";

struct SignalOpen {
    location: Location,
    initialized: Arc<AtomicUsize>,
    contacts: ContactCounters,
    placement: Option<AdversarialPlacement>,
}

fn open_signal_home(cfg: SignalOpen, execution: &ExecutionRequest<'_, '_>) -> SignalHome {
    let declaration = TemporalHostSchema::declaration().unwrap();
    let (installation_predicate, _) = Predicate::controlled(cfg.contacts.clone());
    let (definition_predicate, _) = Predicate::controlled(cfg.contacts.clone());
    let (clock_source, clock_control) = ClockSource::due();
    let configuration = TemporalContributionConfiguration {
        installation_predicate,
        definition_predicate: Arc::new(definition_predicate),
        clock_source,
        clock_control,
        contacts: cfg.contacts,
        install_route: true,
        definition_only: true,
        multi_intent_targets: false,
    };
    let init = cfg.initialized;
    let request = WorthQueryApplicationHomeRequest::new(
        cfg.location,
        ApplicationProgramAuthoring::<TemporalHostSchema, TemporalInstallationProgram>::begin()
            .with_schema(&declaration)
            .validated_program()
            .unwrap(),
        WorthQueryApplicationProgramRoster::new(),
        declaration,
        (configuration,),
        build_id(BUILD_PRIMARY),
        publication_limit::limits(),
        move |_, graph, installed| {
            init.fetch_add(1, Ordering::SeqCst);
            let principal = installed
                .principal_binding(TemporalPrincipalBinding::reference())
                .unwrap();
            seed_graph(graph, &principal, "blocked", 0, 1, true);
            Ok(())
        },
    );
    let request = match cfg.placement {
        Some(p) => host_observation::opening::attach_placement(request, p).0,
        None => request,
    };
    (*with_bootstrap_advancement_from_request(*execution, |phase| {
        Box::new(request.open_in_advancement(&phase))
    })
    .unwrap())
    .unwrap_or_else(|e| panic!("Signal Home open: {e:?}"))
}

fn commit_mixed(home: &SignalHome, branch: WorthQueryProductBranch, ordinal: u8) -> CommitOutcome {
    let app = home.runtime();
    let cond = app.conditional::<TemporalConditional>().unwrap();
    let installed = cond.as_ref().as_ref().unwrap();
    let selected = app.with_host_advancement(|phase| app.on_branch(branch).select(&phase)).unwrap().unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    commit_definition_on_program_lane(
        app, &installed.clock, &installed.invariant, selected,
        &format!("adv-mixed-{ordinal}"), Arc::new(provider), ordinal,
    )
}

fn publish_definition_outcome(
    home: &SignalHome,
    branch: WorthQueryProductBranch,
) -> WorthQueryConditionalDefinitionPublicationOutcome {
    let app = home.runtime();
    let cond = app.conditional::<TemporalConditional>().unwrap();
    let installed = cond.as_ref().as_ref().unwrap();
    let selected = app.with_host_advancement(|phase| app.on_branch(branch).select(&phase)).unwrap().unwrap();
    let (provider, _) = Predicate::controlled(installed.contacts.clone());
    selected
        .publish_conditional_definition(&installed.clock, Arc::new(provider), &RuntimeWorldCancellationSource::new().token())
        .unwrap()
}

fn publish_definition(home: &SignalHome, branch: WorthQueryProductBranch) {
    assert!(matches!(
        publish_definition_outcome(home, branch),
        WorthQueryConditionalDefinitionPublicationOutcome::Performed(_)
    ));
}

fn observe_signal(home: &SignalHome) -> SignalSnapshot {
    let app = home.runtime();
    let branch = app.current_world();
    let row = publication_limit::read_intent_on_branch(app, branch);
    let (world_commit_ordinal, native_basis) = app
        .with_host_advancement(|phase| {
            let selected = app.on_branch(branch).select(&phase).unwrap();
            let prod = selected.product();
            let desc = prod.relational_basis_descriptor();
            (
                prod.selected_commit().ordinal(),
                StableNativeBasis {
                    branch_id: desc.branch_id().clone(),
                    truth_version: desc.truth_version(),
                    schema_commitment: desc.schema_commitment(),
                    posture: desc.posture(),
                },
            )
        })
        .unwrap();
    SignalSnapshot { intent: (row.identity, row.revision, row.lifecycle, row.input), world_commit_ordinal, native_basis }
}

fn assert_bridge_publication_healthy(home: &SignalHome, branch: WorthQueryProductBranch, before_ord: u64) {
    use WorthQueryConditionalDefinitionPublicationOutcome::Performed;
    let Performed(p1) = publish_definition_outcome(home, branch) else { panic!("Bridge def must publish") };
    assert_eq!(p1.product_commit().ordinal(), before_ord + 1, "Bridge publication advances World +1");
    let Performed(p2) = publish_definition_outcome(home, branch) else { panic!("repeat Bridge def must publish") };
    assert_eq!(p2.definition_generation(), p1.definition_generation() + 1, "Bridge def gen advances +1");
    assert_eq!(p2.product_commit().ordinal(), p1.product_commit().ordinal() + 1, "repeat Bridge advances +1");
}

#[test]
fn signal_bridge_mixed_publication_io_boundary_kill_sweep() {
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let (expected_before, expected_after) = build_signal_reference(&execution);
    let saved = RestorableHome::prepare(|loc| {
        let home = open_signal_home(
            SignalOpen { location: Location::Directory(loc.to_path_buf()), initialized: Arc::new(AtomicUsize::new(0)), contacts: ContactCounters::default(), placement: None },
            &execution,
        );
        let main = home.runtime().current_world();
        assert!(commit_mixed(&home, main, 1).require_committed().is_ok());
        publish_definition(&home, main);
        home.close(&execution).unwrap().reclaim();
    });
    let m1 = measure_signal_syncs(&saved);
    let m2 = measure_signal_syncs(&saved);
    assert!(m1.abs_diff(m2) <= TAIL_VARIANCE_BOUND, "Signal/Bridge calibration spread {m1} vs {m2}");
    let total_syncs = m1.max(m2);
    let outcomes = run_parallel_barriers(
        "Signal/Bridge",
        total_syncs,
        || {
            let w_req = opening::serial();
            let w_exec = ExecutionRequest::serial(&w_req);
            RestorableHome::prepare(|loc| {
                let home = open_signal_home(
                    SignalOpen { location: Location::Directory(loc.to_path_buf()), initialized: Arc::new(AtomicUsize::new(0)), contacts: ContactCounters::default(), placement: None },
                    &w_exec,
                );
                let main = home.runtime().current_world();
                assert!(commit_mixed(&home, main, 1).require_committed().is_ok());
                publish_definition(&home, main);
                home.close(&w_exec).unwrap().reclaim();
            })
        },
        |worker_home, ordinal| {
            let w_req = opening::serial();
            let w_exec = ExecutionRequest::serial(&w_req);
            worker_home.restore();
            let kill = kill_signal_at(worker_home.location(), ordinal, total_syncs);
            let phase = verify_signal_reopen(worker_home.location(), (&expected_before, &expected_after), ordinal, &w_exec);
            if matches!(kill, BarrierKill::CompletedAtTail { .. }) {
                assert_eq!(phase, ReopenPhase::Committed, "Signal/Bridge sync#{ordinal}: completed tail run must reopen committed without custody");
            }
            (kill, phase)
        },
    );
    assert_all_reopen_phases_killed("Signal/Bridge", &outcomes);
}

fn build_signal_reference(execution: &ExecutionRequest<'_, '_>) -> (SignalSnapshot, SignalSnapshot) {
    let ref_home = open_signal_home(
        SignalOpen { location: Location::MemoryOnly, initialized: Arc::new(AtomicUsize::new(0)), contacts: ContactCounters::default(), placement: None },
        execution,
    );
    let ref_main = ref_home.runtime().current_world();
    assert!(commit_mixed(&ref_home, ref_main, 1).require_committed().is_ok());
    publish_definition(&ref_home, ref_main);
    let expected_before = observe_signal(&ref_home);
    assert!(commit_mixed(&ref_home, ref_main, 2).require_committed().is_ok());
    let expected_after = observe_signal(&ref_home);
    assert_ne!(expected_before.intent, expected_after.intent);
    assert_ne!(expected_before.native_basis, expected_after.native_basis);
    assert_eq!(expected_after.world_commit_ordinal, expected_before.world_commit_ordinal + 1);
    assert_bridge_publication_healthy(&ref_home, ref_main, expected_after.world_commit_ordinal);
    drop(ref_home.close(execution).err().unwrap().resume().unwrap());
    (expected_before, expected_after)
}

fn measure_signal_syncs(saved: &RestorableHome) -> u64 {
    saved.restore();
    let markers = tempfile::tempdir().unwrap();
    let env = [(ENV_HOME, saved.location()), (ENV_MARKERS, markers.path()), (ENV_ORDINAL, Path::new("count"))];
    let end = crate::killed_child::run_and_kill(CHILD, &env, &markers.path().join("ready"), &markers.path().join("finished"));
    assert_eq!(end, crate::killed_child::ChildEnd::Finished);
    std::fs::read_to_string(markers.path().join("finished")).unwrap().trim().parse().unwrap()
}

fn kill_signal_at(location: &Path, ordinal: u64, total_syncs: u64) -> BarrierKill {
    let markers = tempfile::tempdir().unwrap();
    let ord_path = PathBuf::from(ordinal.to_string());
    let ready = markers.path().join("ready");
    let env = [(ENV_HOME, location), (ENV_MARKERS, markers.path()), (ENV_ORDINAL, &ord_path)];
    let end = crate::killed_child::run_and_kill(CHILD, &env, &ready, &markers.path().join("finished"));
    classify_barrier_kill(end, &ready, MediaOperationRole::SynchronizeFileState, ordinal, total_syncs, &format!("Signal/Bridge sync#{ordinal}/{total_syncs}"))
}

fn verify_signal_reopen(
    location: &Path,
    expected: (&SignalSnapshot, &SignalSnapshot),
    ordinal: u64,
    execution: &ExecutionRequest<'_, '_>,
) -> ReopenPhase {
    let (expected_before, expected_after) = expected;
    let init = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let home = open_signal_home(
        SignalOpen { location: Location::Directory(location.to_path_buf()), initialized: init.clone(), contacts: contacts.clone(), placement: None },
        execution,
    );
    assert_eq!(init.load(Ordering::SeqCst), 0, "sync#{ordinal}: no re-init");
    assert_eq!(contacts.snapshot(), (0, 0, 0, 0), "sync#{ordinal}: no reopen provider calls");
    let main = home.runtime().current_world();
    let handles: Vec<_> = home
        .runtime().product_publication_recovery_page(None, std::num::NonZeroUsize::new(16).unwrap()).unwrap()
        .rows().iter().map(|row| row.handle().clone()).collect();
    let state = observe_signal(&home);
    let phase = if !handles.is_empty() {
        verify_signal_custody((&home, main), &handles, (&state, expected_before, expected_after), (ordinal, &contacts, execution));
        ReopenPhase::Custody
    } else if state.intent == expected_before.intent {
        assert_eq!(state, *expected_before, "sync#{ordinal}: pre-op snapshot must match MemoryOnly");
        assert!(commit_mixed(&home, main, 2).require_committed().is_ok());
        let after = observe_signal(&home);
        assert_signal_advanced(&after, expected_before, expected_after, (ordinal, true));
        assert!(matches!(commit_mixed(&home, main, 2), CommitOutcome::Committed(_) | CommitOutcome::RecoveredCommitted(_)));
        assert_eq!(observe_signal(&home), after, "sync#{ordinal}: duplicate retry must not advance");
        ReopenPhase::PreOp
    } else {
        assert_signal_advanced(&state, expected_before, expected_after, (ordinal, false));
        assert!(matches!(commit_mixed(&home, main, 2), CommitOutcome::RecoveredCommitted(_)));
        assert_eq!(observe_signal(&home), state, "sync#{ordinal}: retry must not advance");
        assert_eq!(contacts.snapshot(), (0, 0, 0, 0), "sync#{ordinal}: zero replay on committed");
        ReopenPhase::Committed
    };
    let settled = observe_signal(&home);
    assert_eq!(settled.intent, expected_after.intent, "sync#{ordinal}: settled intent parity");
    assert_eq!(settled.native_basis, expected_after.native_basis, "sync#{ordinal}: settled native basis parity");
    assert_bridge_publication_healthy(&home, main, settled.world_commit_ordinal);
    home.close(execution).unwrap_or_else(|e| panic!("sync#{ordinal}: Signal close: {e:?}")).reclaim();
    phase
}

fn assert_signal_advanced(
    actual: &SignalSnapshot,
    before: &SignalSnapshot,
    after: &SignalSnapshot,
    opts: (u64, bool),
) {
    let (ordinal, allow_released_gap) = opts;
    assert_eq!(actual.intent, after.intent, "sync#{ordinal}: intent parity");
    assert_eq!(actual.native_basis, after.native_basis, "sync#{ordinal}: stable Native basis parity");
    let expected_next = before.world_commit_ordinal + 1;
    if allow_released_gap {
        assert!(
            actual.world_commit_ordinal == expected_next
                || actual.world_commit_ordinal == expected_next + 1,
            "sync#{ordinal}: post-retry ordinal {} must be {expected_next} or {}",
            actual.world_commit_ordinal,
            expected_next + 1
        );
    } else {
        assert_eq!(
            actual.world_commit_ordinal, expected_next,
            "sync#{ordinal}: committed/continued ordinal must be exact predecessor + 1"
        );
    }
}

fn verify_signal_custody(
    target: (&SignalHome, WorthQueryProductBranch),
    handles: &[worth_runtime_world::facade::ProductUnpublishedRecoveryHandle],
    states: (&SignalSnapshot, &SignalSnapshot, &SignalSnapshot),
    run: (u64, &ContactCounters, &ExecutionRequest<'_, '_>),
) {
    let (home, main) = target;
    let (ordinal, contacts, execution) = run;
    let (state, expected_before, expected_after) = states;
    assert_eq!(handles.len(), 1, "sync#{ordinal}: at most one unpublished custody");
    assert_eq!(state, expected_before, "sync#{ordinal}: head and Native basis stayed at predecessor");
    assert!(matches!(commit_mixed(home, main, 2), CommitOutcome::ProductUnpublished(_)));
    let recovery = home.runtime().readmit_product_publication_recovery(&handles[0]).unwrap();
    loop {
        match recovery.continue_original_in_request(execution).unwrap() {
            WorldRecoveredContinuationOutcome::Pending(c) => drop(c),
            WorldRecoveredContinuationOutcome::Published { .. } => break,
            other => panic!("sync#{ordinal}: Signal continuation failed: {other:?}"),
        }
    }
    let continued = observe_signal(home);
    assert_signal_advanced(&continued, expected_before, expected_after, (ordinal, false));
    assert_eq!(contacts.snapshot(), (0, 0, 0, 0), "sync#{ordinal}: zero replay on continuation");
    assert!(matches!(commit_mixed(home, main, 2), CommitOutcome::RecoveredCommitted(_)));
    assert_eq!(observe_signal(home), continued, "sync#{ordinal}: retry must not advance");
}

#[test]
fn signal_bridge_io_kill_child() {
    let Some(location) = std::env::var_os(ENV_HOME).map(PathBuf::from) else { return; };
    let markers = PathBuf::from(std::env::var_os(ENV_MARKERS).unwrap());
    let ord_str = std::env::var(ENV_ORDINAL).unwrap();
    let req = opening::serial();
    let execution = ExecutionRequest::serial(&req);
    let done = Arc::new(AtomicBool::new(false));
    let (placement, activation, watcher) = if ord_str == "count" {
        sync_barrier_placement(u64::MAX, None, None)
    } else {
        sync_barrier_placement(ord_str.parse().unwrap(), Some(markers.join("ready")), Some(done.clone()))
    };
    let home = open_signal_home(
        SignalOpen { location: Location::Directory(location), initialized: Arc::new(AtomicUsize::new(0)), contacts: ContactCounters::default(), placement: Some(placement) },
        &execution,
    );
    activation.arm().unwrap();
    let main = home.runtime().current_world();
    let _ = commit_mixed(&home, main, 2);
    done.store(true, Ordering::SeqCst);
    if let Some(handle) = watcher {
        let _ = handle.join();
        let actual = activation.matching_operation_count();
        crate::killed_child::announce(&markers.join("finished"), format!("missed:actual={actual}").as_bytes());
        return;
    }
    let count = activation.matching_operation_count();
    home.close(&execution).unwrap().reclaim();
    crate::killed_child::announce(&markers.join("finished"), count.to_string().as_bytes());
}
