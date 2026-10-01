use std::collections::BTreeMap;

use crate::facade::{
    LineageRecord, NodeEvaluationResult, OutputChange, ReplaySlice, SignalError,
    SignalRuntimePolicy, SignalTransaction,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

use super::fintech_world::{
    build_fintech_fixture, fintech_checked_evaluator, fintech_evaluator, seed_fintech_baseline,
    FintechFixture,
};
use super::invariant_oracle::assert_runtime_invariants;
use super::workflow_truth::{
    capture_active_branch_snapshot, trace_adv, AdversarialWorkflow, FailureInjectionPoint, Lcg,
    ReferenceModel, SignalAdversarialHarness, WorkflowDomain, WorkflowSeed,
};
use crate::tests::leased_execution::support::{authority, request};

type FintechTransaction<'a> = SignalTransaction<'a, (), (), (), (), ()>;

fn evaluate_fintech_dirty(fixture: &mut FintechFixture, workers: Option<usize>) {
    if let Some(workers) = workers {
        let lease = authority()
            .request_lease(request(workers, 10_000_000))
            .unwrap();
        fixture
            .runtime
            .evaluate_dirty_checked(&(), &fintech_checked_evaluator(fixture), &lease)
            .unwrap();
    } else {
        fixture
            .runtime
            .evaluate_dirty(&(), &fintech_evaluator(fixture))
            .unwrap();
    }
}

pub(super) fn fintech_session(
    seed: WorkflowSeed,
    workflow: AdversarialWorkflow,
    policy: SignalRuntimePolicy,
    workers: Option<usize>,
) -> (SignalAdversarialHarness, ReplaySlice, Vec<LineageRecord>) {
    let policy = policy
        .with_observation_activation(worth_foundational::ObservationActivationProfile::Continuous);
    if workers.is_some() {
        trace_adv(format!(
            "[fintech {:?} seed={}] setup:start",
            workflow, seed.0
        ));
    }
    let mut harness = SignalAdversarialHarness::new(seed, WorkflowDomain::Fintech, workflow);
    let mut fixture = build_fintech_fixture(policy);
    let mut model = ReferenceModel::default();
    let mut snapshots = BTreeMap::new();
    let mut branch_history = BTreeMap::new();
    let (main, main_snapshot) = seed_fintech_baseline(&mut fixture, &mut model);
    if workers.is_some() {
        trace_adv(format!(
            "[fintech {:?} seed={}] setup:seeded-main",
            workflow, seed.0
        ));
    }
    snapshots.insert(main_snapshot.meta.snapshot_id, main_snapshot.clone());
    branch_history.insert(main.id, vec![main_snapshot.meta.snapshot_id]);
    harness.record(&fixture.runtime, 0, "seed-main-baseline", None);

    let what_if = fixture.runtime.create_branch("what-if").unwrap();
    model
        .branches
        .insert(what_if.id, model.branch(main.id).clone());
    fixture.runtime.switch_branch(what_if.clone()).unwrap();
    model.active = what_if.id;
    let what_if_snapshot = fixture
        .runtime
        .capture_branch_snapshot(fixture.runtime.observe().current_branch())
        .unwrap();
    if workers.is_some() {
        trace_adv(format!(
            "[fintech {:?} seed={}] setup:what-if-ready",
            workflow, seed.0
        ));
    }
    snapshots.insert(what_if_snapshot.meta.snapshot_id, what_if_snapshot.clone());
    branch_history
        .entry(what_if.id)
        .or_default()
        .push(what_if_snapshot.meta.snapshot_id);
    let mut what_if_truth = model.branch(what_if.id).clone();
    what_if_truth.head_snapshot = Some(what_if_snapshot.meta.snapshot_id);
    model
        .snapshots
        .insert(what_if_snapshot.meta.snapshot_id, what_if_truth);
    model.branch_mut(what_if.id).head_snapshot = Some(what_if_snapshot.meta.snapshot_id);
    harness.record(&fixture.runtime, 1, "create-what-if-branch", None);

    let correction = fixture.runtime.create_branch("correction").unwrap();
    model
        .branches
        .insert(correction.id, model.branch(what_if.id).clone());
    fixture.runtime.switch_branch(correction.clone()).unwrap();
    model.active = correction.id;
    let correction_snapshot = fixture
        .runtime
        .capture_branch_snapshot(fixture.runtime.observe().current_branch())
        .unwrap();
    if workers.is_some() {
        trace_adv(format!(
            "[fintech {:?} seed={}] setup:correction-ready",
            workflow, seed.0
        ));
    }
    snapshots.insert(
        correction_snapshot.meta.snapshot_id,
        correction_snapshot.clone(),
    );
    branch_history
        .entry(correction.id)
        .or_default()
        .push(correction_snapshot.meta.snapshot_id);
    let mut correction_truth = model.branch(correction.id).clone();
    correction_truth.head_snapshot = Some(correction_snapshot.meta.snapshot_id);
    model
        .snapshots
        .insert(correction_snapshot.meta.snapshot_id, correction_truth);
    model.branch_mut(correction.id).head_snapshot = Some(correction_snapshot.meta.snapshot_id);
    harness.record(&fixture.runtime, 2, "create-correction-branch", None);

    let mut rng = Lcg::new(seed);
    let mut ctx = ();
    for step in 0..24 {
        if workers.is_some() {
            trace_adv(format!(
                "[fintech {:?} seed={}] step={} branch={}",
                workflow,
                seed.0,
                step,
                fixture.runtime.observe().current_branch().name
            ));
        }
        let action = match workflow {
            AdversarialWorkflow::LateTickCorrectionWithBranchReplay => rng.choose(6),
            AdversarialWorkflow::RiskAlertFlapUnderMemoChurn => rng.choose(5),
            _ => rng.choose(6),
        };

        match action {
            0 => {
                let target =
                    [main.clone(), what_if.clone(), correction.clone()][rng.choose(3)].clone();
                fixture.runtime.switch_branch(target.clone()).unwrap();
                model.active = target.id;
                model.branch_mut(target.id).head_snapshot =
                    fixture.runtime.observe().branch_head_snapshot_id(target.id);
                harness.record(&fixture.runtime, step + 3, "switch-branch", None);
            }
            1 => {
                let delta = rng.small_delta();
                let current = model.branch(model.active).clone();
                let next_a = current.a + delta;
                fixture
                    .runtime
                    .transaction(&mut ctx, |tx: &mut FintechTransaction<'_>| {
                        tx.mark_dirty(fixture.ticks, ASPECT_A)?;
                        tx.read(fixture.ticks, &move |view| {
                            Ok(view.finish(
                                NodeEvaluationResult::from_version(version_ab(next_a, 0))
                                    .with_output_identity(format!("ticks-{next_a}")),
                            ))
                        })?;
                        tx.evaluate_keyed(fixture.keyed, &fixture.memo_key, &|view| {
                            Ok(view.finish(
                                NodeEvaluationResult::from_version(version_ab(next_a, current.b))
                                    .with_output_identity(format!(
                                        "risk-keyed-{next_a}-{}",
                                        current.b
                                    ))
                                    .with_output_change(OutputChange::Refreshed),
                            ))
                        })?;
                        Ok(())
                    })
                    .unwrap();
                model.branch_mut(model.active).a = next_a;
                evaluate_fintech_dirty(&mut fixture, workers);
                if rng.coin() {
                    capture_active_branch_snapshot(
                        &mut fixture.runtime,
                        &mut model,
                        &mut snapshots,
                        &mut branch_history,
                    );
                }
                harness.record(&fixture.runtime, step + 3, "update-ticks", None);
            }
            2 => {
                let delta = rng.small_delta();
                let current = model.branch(model.active).clone();
                let next_b = current.b + delta;
                fixture
                    .runtime
                    .transaction(&mut ctx, |tx: &mut FintechTransaction<'_>| {
                        tx.mark_dirty(fixture.volatility, ASPECT_B)?;
                        tx.read(fixture.volatility, &move |view| {
                            Ok(view.finish(
                                NodeEvaluationResult::from_version(version_ab(0, next_b))
                                    .with_output_identity(format!("volatility-{next_b}")),
                            ))
                        })?;
                        Ok(())
                    })
                    .unwrap();
                model.branch_mut(model.active).b = next_b;
                evaluate_fintech_dirty(&mut fixture, workers);
                if rng.coin() {
                    capture_active_branch_snapshot(
                        &mut fixture.runtime,
                        &mut model,
                        &mut snapshots,
                        &mut branch_history,
                    );
                }
                harness.record(&fixture.runtime, step + 3, "update-volatility", None);
            }
            3 => {
                let current = model.branch(model.active).clone();
                let err =
                    fixture
                        .runtime
                        .transaction(&mut ctx, |tx: &mut FintechTransaction<'_>| {
                            tx.mark_dirty(fixture.ticks, ASPECT_A)?;
                            tx.read(fixture.ticks, &move |view| {
                                Ok(view.finish(
                                    NodeEvaluationResult::from_version(version_ab(
                                        current.a + 10,
                                        0,
                                    ))
                                    .with_output_identity("bad-ticks"),
                                ))
                            })?;
                            Err(SignalError::invalid_input("synthetic branch-local failure"))
                        });
                assert!(err.is_err());
                harness.record(
                    &fixture.runtime,
                    step + 3,
                    "rollback-live-update",
                    Some(FailureInjectionPoint::DuringBranchLocalWork),
                );
            }
            4 => {
                let active = fixture.runtime.observe().current_branch();
                let candidates = branch_history
                    .get(&active.id)
                    .expect("branch restore should always have branch-local snapshot history");
                let snapshot_id = candidates[rng.choose(candidates.len())];
                let snapshot = snapshots
                    .get(&snapshot_id)
                    .expect("branch restore should use a stored branch-local snapshot")
                    .clone();
                let snapshot = if snapshot.meta.branch_id == active.id {
                    snapshot
                } else {
                    capture_active_branch_snapshot(
                        &mut fixture.runtime,
                        &mut model,
                        &mut snapshots,
                        &mut branch_history,
                    )
                };
                fixture
                    .runtime
                    .restore_branch_snapshot(active.clone(), &snapshot)
                    .unwrap();
                let restored = model
                    .snapshots
                    .get(&snapshot.meta.snapshot_id)
                    .unwrap()
                    .clone();
                *model.branch_mut(active.id) = restored;
                model.branch_mut(active.id).head_snapshot =
                    fixture.runtime.observe().branch_head_snapshot_id(active.id);
                harness.record(
                    &fixture.runtime,
                    step + 3,
                    "restore-current-branch",
                    Some(FailureInjectionPoint::DuringSnapshotRestoreChurn),
                );
            }
            _ => {
                evaluate_fintech_dirty(&mut fixture, workers);
                harness.record(&fixture.runtime, step + 3, "drain-dirty", None);
            }
        }

        let report = assert_runtime_invariants(
            fixture.runtime.graph(),
            &model,
            fixture.runtime.observe().current_branch(),
            fixture.ticks,
            ASPECT_A,
            fixture.volatility,
            ASPECT_B,
            policy,
        );
        if !report.errors.is_empty() {
            harness.panic_invariant(
                &policy,
                &format!("workers={workers:?}"),
                format!("step {}: {}", report.step_index, report.errors.join(" | ")),
            );
        }
    }

    for branch in [main.clone(), what_if.clone(), correction.clone()] {
        fixture.runtime.switch_branch(branch.clone()).unwrap();
        let report = assert_runtime_invariants(
            fixture.runtime.graph(),
            &model,
            branch,
            fixture.ticks,
            ASPECT_A,
            fixture.volatility,
            ASPECT_B,
            policy,
        );
        if !report.errors.is_empty() {
            harness.panic_invariant(
                &policy,
                &format!("workers={workers:?}"),
                format!("step {}: {}", report.step_index, report.errors.join(" | ")),
            );
        }
    }

    let replay = fixture
        .runtime
        .replay_for_branch(fixture.runtime.observe().current_branch().id);
    let lineage = fixture
        .runtime
        .graph()
        .observe()
        .lineage_for_node(fixture.risk)
        .to_owned_records();
    (harness, replay, lineage)
}
