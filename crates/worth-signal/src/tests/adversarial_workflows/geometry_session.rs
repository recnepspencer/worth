use std::collections::BTreeMap;

use crate::facade::{
    ChangedRegion, EvaluationRequestMode, LineageRecord, NodeEvaluationResult, ReplaySlice,
    SignalError, SignalRuntimePolicy, SignalTransaction,
};
use crate::tests::support::{version_ab, ASPECT_A, ASPECT_B};

use super::geometry_world::{
    build_geometry_fixture, geometry_checked_evaluator, geometry_evaluator, seed_geometry_baseline,
    GeometryFixture,
};
use super::invariant_oracle::assert_runtime_invariants;
use super::workflow_truth::{
    capture_active_branch_snapshot, trace_adv, AdversarialWorkflow, FailureInjectionPoint, Lcg,
    ReferenceModel, SignalAdversarialHarness, WorkflowDomain, WorkflowSeed,
};
use crate::tests::leased_execution::support::{authority, request};

type GeometryTransaction<'a> = SignalTransaction<'a, (), (), (), (), ()>;

fn evaluate_geometry_dirty(fixture: &mut GeometryFixture, workers: Option<usize>) {
    if let Some(workers) = workers {
        let lease = authority()
            .request_lease(request(workers, 10_000_000))
            .unwrap();
        fixture
            .runtime
            .evaluate_dirty_checked(&(), &geometry_checked_evaluator(fixture), &lease)
            .unwrap();
    } else {
        fixture
            .runtime
            .evaluate_dirty(&(), &geometry_evaluator(fixture))
            .unwrap();
    }
}

fn evaluate_geometry_demand(fixture: &mut GeometryFixture, workers: Option<usize>) {
    if let Some(workers) = workers {
        let lease = authority()
            .request_lease(request(workers, 10_000_000))
            .unwrap();
        fixture
            .runtime
            .evaluate_checked(
                &[fixture.demand_gate],
                EvaluationRequestMode::ForceOnDemand,
                &(),
                &geometry_checked_evaluator(fixture),
                &lease,
            )
            .unwrap();
    } else {
        fixture
            .runtime
            .evaluate_with_plan(
                fixture.demand_gate,
                &(),
                &geometry_evaluator(fixture),
                EvaluationRequestMode::ForceOnDemand,
            )
            .unwrap();
    }
}

pub(super) fn geometry_session(
    seed: WorkflowSeed,
    workflow: AdversarialWorkflow,
    policy: SignalRuntimePolicy,
    workers: Option<usize>,
) -> (SignalAdversarialHarness, ReplaySlice, Vec<LineageRecord>) {
    let policy = policy
        .with_observation_activation(worth_foundational::ObservationActivationProfile::Continuous);
    if workers.is_some() {
        trace_adv(format!(
            "[geometry {:?} seed={}] setup:start",
            workflow, seed.0
        ));
    }
    let mut harness = SignalAdversarialHarness::new(seed, WorkflowDomain::GeometryKernel, workflow);
    let mut fixture = build_geometry_fixture(policy);
    let mut model = ReferenceModel::default();
    let mut snapshots = BTreeMap::new();
    let mut branch_history = BTreeMap::new();
    let (main, main_snapshot) = seed_geometry_baseline(&mut fixture, &mut model);
    if workers.is_some() {
        trace_adv(format!(
            "[geometry {:?} seed={}] setup:seeded-main",
            workflow, seed.0
        ));
    }
    snapshots.insert(main_snapshot.meta.snapshot_id, main_snapshot.clone());
    branch_history.insert(main.id, vec![main_snapshot.meta.snapshot_id]);
    harness.record(&fixture.runtime, 0, "seed-main-baseline", None);

    let feature = fixture.runtime.create_branch("feature").unwrap();
    model
        .branches
        .insert(feature.id, model.branch(main.id).clone());
    fixture.runtime.switch_branch(feature.clone()).unwrap();
    model.active = feature.id;
    harness.record(&fixture.runtime, 1, "create-feature-branch", None);
    let feature_snapshot = fixture
        .runtime
        .capture_branch_snapshot(fixture.runtime.observe().current_branch())
        .unwrap();
    if workers.is_some() {
        trace_adv(format!(
            "[geometry {:?} seed={}] setup:feature-ready",
            workflow, seed.0
        ));
    }
    snapshots.insert(feature_snapshot.meta.snapshot_id, feature_snapshot.clone());
    branch_history
        .entry(feature.id)
        .or_default()
        .push(feature_snapshot.meta.snapshot_id);
    let mut feature_truth = model.branch(feature.id).clone();
    feature_truth.head_snapshot = Some(feature_snapshot.meta.snapshot_id);
    model
        .snapshots
        .insert(feature_snapshot.meta.snapshot_id, feature_truth);
    model.branch_mut(feature.id).head_snapshot = Some(feature_snapshot.meta.snapshot_id);

    let analysis = fixture.runtime.create_branch("analysis").unwrap();
    model
        .branches
        .insert(analysis.id, model.branch(feature.id).clone());
    fixture.runtime.switch_branch(analysis.clone()).unwrap();
    model.active = analysis.id;
    let analysis_snapshot = fixture
        .runtime
        .capture_branch_snapshot(fixture.runtime.observe().current_branch())
        .unwrap();
    if workers.is_some() {
        trace_adv(format!(
            "[geometry {:?} seed={}] setup:analysis-ready",
            workflow, seed.0
        ));
    }
    snapshots.insert(
        analysis_snapshot.meta.snapshot_id,
        analysis_snapshot.clone(),
    );
    branch_history
        .entry(analysis.id)
        .or_default()
        .push(analysis_snapshot.meta.snapshot_id);
    let mut analysis_truth = model.branch(analysis.id).clone();
    analysis_truth.head_snapshot = Some(analysis_snapshot.meta.snapshot_id);
    model
        .snapshots
        .insert(analysis_snapshot.meta.snapshot_id, analysis_truth);
    model.branch_mut(analysis.id).head_snapshot = Some(analysis_snapshot.meta.snapshot_id);
    harness.record(&fixture.runtime, 2, "create-analysis-branch", None);

    let mut rng = Lcg::new(seed);
    let mut ctx = ();
    for step in 0..24 {
        if workers.is_some() {
            trace_adv(format!(
                "[geometry {:?} seed={}] step={} branch={}",
                workflow,
                seed.0,
                step,
                fixture.runtime.observe().current_branch().name
            ));
        }
        let action = match workflow {
            AdversarialWorkflow::FeatureEditRewireRestoreChurn => rng.choose(6),
            AdversarialWorkflow::PartitionScopeCliffSession => rng.choose(5),
            _ => rng.choose(6),
        };

        match action {
            0 => {
                let target =
                    [main.clone(), feature.clone(), analysis.clone()][rng.choose(3)].clone();
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
                let result =
                    fixture
                        .runtime
                        .transaction(&mut ctx, |tx: &mut GeometryTransaction<'_>| {
                            tx.mark_dirty_with_regions(
                                fixture.source_a,
                                ASPECT_A,
                                &[ChangedRegion::new("wing").with_detail(format!("panel-{step}"))],
                            )?;
                            tx.read(fixture.source_a, &move |view| {
                                Ok(view.finish(
                                    NodeEvaluationResult::from_version(version_ab(next_a, 0))
                                        .with_output_identity(format!("geom-source-a-{next_a}")),
                                ))
                            })?;
                            Ok(())
                        });
                result.unwrap();
                model.branch_mut(model.active).a = next_a;
                evaluate_geometry_dirty(&mut fixture, workers);
                if rng.coin() {
                    capture_active_branch_snapshot(
                        &mut fixture.runtime,
                        &mut model,
                        &mut snapshots,
                        &mut branch_history,
                    );
                }
                harness.record(&fixture.runtime, step + 3, "update-source-a", None);
            }
            2 => {
                let delta = rng.small_delta();
                let current = model.branch(model.active).clone();
                let next_b = current.b + delta;
                let result =
                    fixture
                        .runtime
                        .transaction(&mut ctx, |tx: &mut GeometryTransaction<'_>| {
                            tx.mark_dirty_with_regions(
                                fixture.source_b,
                                ASPECT_B,
                                &[ChangedRegion::new("lod")],
                            )?;
                            tx.read(fixture.source_b, &move |view| {
                                Ok(view.finish(
                                    NodeEvaluationResult::from_version(version_ab(0, next_b))
                                        .with_output_identity(format!("geom-source-b-{next_b}")),
                                ))
                            })?;
                            Ok(())
                        });
                result.unwrap();
                model.branch_mut(model.active).b = next_b;
                evaluate_geometry_dirty(&mut fixture, workers);
                if rng.coin() {
                    capture_active_branch_snapshot(
                        &mut fixture.runtime,
                        &mut model,
                        &mut snapshots,
                        &mut branch_history,
                    );
                }
                harness.record(&fixture.runtime, step + 3, "update-source-b", None);
            }
            3 => {
                let delta = rng.small_delta();
                let current = model.branch(model.active).clone();
                let bad_a = current.a + delta;
                let err =
                    fixture
                        .runtime
                        .transaction(&mut ctx, |tx: &mut GeometryTransaction<'_>| {
                            tx.mark_dirty(fixture.source_a, ASPECT_A)?;
                            tx.read(fixture.source_a, &move |view| {
                                Ok(view.finish(
                                    NodeEvaluationResult::from_version(version_ab(bad_a, 0))
                                        .with_output_identity(format!("geom-source-a-bad-{bad_a}")),
                                ))
                            })?;
                            Err(SignalError::invalid_input("synthetic geometry rollback"))
                        });
                assert!(err.is_err());
                harness.record(
                    &fixture.runtime,
                    step + 3,
                    "rollback-source-a-update",
                    Some(FailureInjectionPoint::DuringEvaluation),
                );
            }
            4 => {
                evaluate_geometry_demand(&mut fixture, workers);
                harness.record(&fixture.runtime, step + 3, "force-on-demand", None);
            }
            _ => {
                let active = fixture.runtime.observe().current_branch();
                let candidates = branch_history
                    .get(&active.id)
                    .expect("branch restore should always have branch-local snapshot history");
                let snapshot_id = candidates[rng.choose(candidates.len())];
                let restore_snapshot = snapshots
                    .get(&snapshot_id)
                    .expect("branch restore should use a stored branch-local snapshot")
                    .clone();
                let restore_snapshot = if restore_snapshot.meta.branch_id == active.id {
                    restore_snapshot
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
                    .restore_branch_snapshot(active.clone(), &restore_snapshot)
                    .unwrap();
                let restored = model
                    .snapshots
                    .get(&restore_snapshot.meta.snapshot_id)
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
        }

        let report = assert_runtime_invariants(
            fixture.runtime.graph(),
            &model,
            fixture.runtime.observe().current_branch(),
            fixture.source_a,
            ASPECT_A,
            fixture.source_b,
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

    for branch in [main.clone(), feature.clone(), analysis.clone()] {
        fixture.runtime.switch_branch(branch.clone()).unwrap();
        let report = assert_runtime_invariants(
            fixture.runtime.graph(),
            &model,
            branch,
            fixture.source_a,
            ASPECT_A,
            fixture.source_b,
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
        .lineage_for_node(fixture.fused)
        .to_owned_records();
    (harness, replay, lineage)
}
