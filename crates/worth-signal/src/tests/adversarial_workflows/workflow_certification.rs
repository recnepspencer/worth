use crate::facade::{compare_lineage_records, compare_replay_slices, SignalRuntimePolicy};

use crate::facade::{NodeEvaluationResult, SignalTransaction};
use crate::tests::support::{version_ab, ASPECT_A};

use super::fintech_session::fintech_session;
use super::geometry_session::geometry_session;
use super::geometry_world::{
    build_geometry_fixture, geometry_checked_evaluator, seed_geometry_baseline,
};
use super::workflow_truth::{AdversarialWorkflow, WorkflowDomain, WorkflowSeed};
use crate::tests::leased_execution::support::{authority, request};

use super::workflow_truth::trace_adv;
use super::workflow_truth::ReferenceModel;

type GeometryTransaction<'a> = SignalTransaction<'a, (), (), (), (), ()>;

#[test]
fn geometry_kernel_adversarial_seed_matrix_keeps_invariants() {
    for (seed, workflow) in [
        (
            WorkflowSeed(7),
            AdversarialWorkflow::FeatureEditRewireRestoreChurn,
        ),
        (
            WorkflowSeed(19),
            AdversarialWorkflow::PartitionScopeCliffSession,
        ),
    ] {
        let _ = geometry_session(
            seed,
            workflow,
            SignalRuntimePolicy::kernel().with_history_limit(8),
            None,
        );
    }
}

#[test]
fn fintech_adversarial_seed_matrix_keeps_invariants() {
    for (seed, workflow) in [
        (
            WorkflowSeed(11),
            AdversarialWorkflow::LateTickCorrectionWithBranchReplay,
        ),
        (
            WorkflowSeed(23),
            AdversarialWorkflow::RiskAlertFlapUnderMemoChurn,
        ),
    ] {
        let _ = fintech_session(
            seed,
            workflow,
            SignalRuntimePolicy::fintech().with_history_limit(8),
            None,
        );
    }
}

#[test]
fn policy_overlap_for_generated_workflows_matches_guaranteed_truth() {
    for (domain, workflow, seed) in [
        (
            WorkflowDomain::GeometryKernel,
            AdversarialWorkflow::FeatureEditRewireRestoreChurn,
            WorkflowSeed(31),
        ),
        (
            WorkflowDomain::Fintech,
            AdversarialWorkflow::LateTickCorrectionWithBranchReplay,
            WorkflowSeed(37),
        ),
    ] {
        let runs = [
            (
                "operational",
                match domain {
                    WorkflowDomain::GeometryKernel => geometry_session(
                        seed,
                        workflow,
                        SignalRuntimePolicy::operational().with_history_limit(4),
                        None,
                    ),
                    WorkflowDomain::Fintech => fintech_session(
                        seed,
                        workflow,
                        SignalRuntimePolicy::operational().with_history_limit(4),
                        None,
                    ),
                },
            ),
            (
                "development",
                match domain {
                    WorkflowDomain::GeometryKernel => geometry_session(
                        seed,
                        workflow,
                        SignalRuntimePolicy::development().with_history_limit(6),
                        None,
                    ),
                    WorkflowDomain::Fintech => fintech_session(
                        seed,
                        workflow,
                        SignalRuntimePolicy::development().with_history_limit(6),
                        None,
                    ),
                },
            ),
            (
                "forensic",
                match domain {
                    WorkflowDomain::GeometryKernel => geometry_session(
                        seed,
                        workflow,
                        SignalRuntimePolicy::forensic().with_history_limit(8),
                        None,
                    ),
                    WorkflowDomain::Fintech => fintech_session(
                        seed,
                        workflow,
                        SignalRuntimePolicy::forensic().with_history_limit(8),
                        None,
                    ),
                },
            ),
        ];

        for pair in runs.windows(2) {
            let (_, (h1, replay1, lineage1)) = &pair[0];
            let (name2, (h2, replay2, lineage2)) = &pair[1];
            let replay_diff = compare_replay_slices(replay1, replay2);
            let lineage_diff = compare_lineage_records(lineage1, lineage2);
            if !replay_diff.is_empty() {
                h2.panic_diff(
                    &SignalRuntimePolicy::development(),
                    "serial",
                    format!("policy overlap drift against {name2}"),
                    replay_diff.mismatches.len(),
                    lineage_diff.mismatches.len(),
                );
            }
            let _ = h1;
        }
    }
}

#[test]
fn parallel_geometry_hostile_session_matches_serial_truth() {
    trace_adv("[parallel-test] geometry:start");
    let workflow = AdversarialWorkflow::FeatureEditRewireRestoreChurn;
    let seed = WorkflowSeed(41);
    let serial = geometry_session(
        seed,
        workflow,
        SignalRuntimePolicy::development().with_history_limit(8),
        None,
    );
    trace_adv("[parallel-test] geometry:serial-finished");
    for workers in [1, 2, 4] {
        let leased = geometry_session(
            seed,
            workflow,
            SignalRuntimePolicy::development().with_history_limit(8),
            Some(workers),
        );
        trace_adv(format!("[parallel-test] geometry:lease-{workers}-finished"));
        let replay_diff = compare_replay_slices(&serial.1, &leased.1);
        let lineage_diff = compare_lineage_records(&serial.2, &leased.2);
        if !replay_diff.is_empty() || !lineage_diff.is_empty() {
            leased.0.panic_diff(
                &SignalRuntimePolicy::development(),
                &format!("serial-vs-lease-{workers}"),
                "geometry execution differential drift",
                replay_diff.mismatches.len(),
                lineage_diff.mismatches.len(),
            );
        }
    }
}

#[test]
fn parallel_fintech_hostile_session_matches_serial_truth() {
    trace_adv("[parallel-test] fintech:start");
    let workflow = AdversarialWorkflow::RiskAlertFlapUnderMemoChurn;
    let seed = WorkflowSeed(53);
    let serial = fintech_session(
        seed,
        workflow,
        SignalRuntimePolicy::development().with_history_limit(8),
        None,
    );
    trace_adv("[parallel-test] fintech:serial-finished");
    for workers in [1, 2, 4] {
        let leased = fintech_session(
            seed,
            workflow,
            SignalRuntimePolicy::development().with_history_limit(8),
            Some(workers),
        );
        trace_adv(format!("[parallel-test] fintech:lease-{workers}-finished"));
        let replay_diff = compare_replay_slices(&serial.1, &leased.1);
        let lineage_diff = compare_lineage_records(&serial.2, &leased.2);
        if !replay_diff.is_empty() || !lineage_diff.is_empty() {
            leased.0.panic_diff(
                &SignalRuntimePolicy::development(),
                &format!("serial-vs-lease-{workers}"),
                format!(
                    "fintech execution differential drift; replay count={}/{} first={:?}; lineage count={}/{} first={:?}",
                    serial.1.frames.len(),
                    leased.1.frames.len(),
                    serial
                        .1
                        .frames
                        .iter()
                        .zip(&leased.1.frames)
                        .enumerate()
                        .find(|(_, (left, right))| left != right),
                    serial.2.len(),
                    leased.2.len(),
                    serial
                        .2
                        .iter()
                        .zip(&leased.2)
                        .enumerate()
                        .find(|(_, (left, right))| left != right),
                ),
                replay_diff.mismatches.len(),
                lineage_diff.mismatches.len(),
            );
        }
    }
}

#[test]
fn focused_parallel_branch_restore_and_evaluate_dirty_regression() {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    trace_adv("[parallel-test] focused-regression:start");
    let mut fixture =
        build_geometry_fixture(SignalRuntimePolicy::development().with_history_limit(8));
    let mut model = ReferenceModel::default();
    let (main, main_snapshot) = seed_geometry_baseline(&mut fixture, &mut model);
    trace_adv("[parallel-test] focused-regression:seeded-main");

    let feature = fixture.runtime.create_branch("feature").unwrap();
    fixture.runtime.switch_branch(feature.clone()).unwrap();
    trace_adv("[parallel-test] focused-regression:feature-branch");

    let mut ctx = ();
    fixture
        .runtime
        .transaction(
            request_execution,
            &mut ctx,
            |tx: &mut GeometryTransaction<'_>| {
                tx.mark_dirty(fixture.source_a, ASPECT_A)?;
                tx.read(fixture.source_a, &|view| {
                    Ok(view.finish(
                        NodeEvaluationResult::from_version(version_ab(4, 1))
                            .with_output_identity("source-a-4"),
                    ))
                })?;
                Ok(())
            },
        )
        .unwrap();
    trace_adv("[parallel-test] focused-regression:mutated-feature");

    fixture
        .runtime
        .evaluate_dirty_checked(
            &(),
            &geometry_checked_evaluator(&fixture),
            &authority().request_lease(request(4, 10_000_000)).unwrap(),
        )
        .unwrap();
    trace_adv("[parallel-test] focused-regression:parallel-evaluated");

    let feature_snapshot = fixture
        .runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    fixture
        .runtime
        .restore_branch_snapshot(feature.clone(), &feature_snapshot)
        .unwrap();
    trace_adv("[parallel-test] focused-regression:feature-restored");

    fixture.runtime.switch_branch(main.clone()).unwrap();
    fixture
        .runtime
        .restore_branch_snapshot(main, &main_snapshot)
        .unwrap();
    trace_adv("[parallel-test] focused-regression:main-restored");

    let replay = fixture.runtime.observe().replay_for_branch(feature.id);
    assert!(
        !replay.frames.is_empty(),
        "parallel branch restore regression should leave observable replay"
    );
}

#[ignore]
#[test]
fn long_geometry_churn_seed_matrix_stays_hard_to_surprise() {
    for seed in [WorkflowSeed(71), WorkflowSeed(89), WorkflowSeed(97)] {
        let _ = geometry_session(
            seed,
            AdversarialWorkflow::FeatureEditRewireRestoreChurn,
            SignalRuntimePolicy::kernel().with_history_limit(12),
            None,
        );
    }
}

#[ignore]
#[test]
fn long_fintech_parallel_churn_seed_matrix_stays_hard_to_surprise() {
    for seed in [WorkflowSeed(101), WorkflowSeed(131), WorkflowSeed(149)] {
        let _ = fintech_session(
            seed,
            AdversarialWorkflow::RiskAlertFlapUnderMemoChurn,
            SignalRuntimePolicy::fintech().with_history_limit(12),
            Some(4),
        );
    }
}
