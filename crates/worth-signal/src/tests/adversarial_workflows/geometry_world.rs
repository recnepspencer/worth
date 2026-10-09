use crate::facade::{
    BoundedSignalInputs, DeclaredSignalInput, EvaluationRequestMode, KeyedComputation,
    NodeContract, NodeEvaluationResult, NodeId, OutputChange, SignalBranchHandle, SignalError,
    SignalGraph, SignalRuntime, SignalRuntimePolicy, SignalSnapshotV1, SignalTransaction,
};
use crate::tests::support::{
    define_keyed_computation, mask_a, mask_b, version_ab, DependencyBatchBuilder, ASPECT_A,
    ASPECT_B,
};

use super::workflow_truth::{BranchTruth, ReferenceModel};

type DefaultRuntime = SignalRuntime<(), (), (), (), ()>;
type DefaultTx<'a> = SignalTransaction<'a, (), (), (), (), ()>;

pub(super) struct GeometryFixture {
    pub(super) runtime: DefaultRuntime,
    pub(super) source_a: NodeId,
    pub(super) source_b: NodeId,
    pub(super) delta_gate: NodeId,
    pub(super) filtered_gate: NodeId,
    pub(super) demand_gate: NodeId,
    pub(super) fused: NodeId,
    pub(super) keyed: NodeId,
    pub(super) memo_key: KeyedComputation,
}

pub(super) fn geometry_evaluator(
    fixture: &GeometryFixture,
) -> impl for<'ctx> Fn(
    &mut crate::logic::context::EvaluationContext<'ctx, ()>,
) -> Result<crate::logic::evaluation::EvaluationOutput, SignalError>
       + Sync {
    let source_a = fixture.source_a;
    let source_b = fixture.source_b;
    let delta_gate = fixture.delta_gate;
    let filtered_gate = fixture.filtered_gate;
    let demand_gate = fixture.demand_gate;
    let fused = fixture.fused;
    move |ctx| {
        let node = ctx.node();
        if node == delta_gate {
            let a = ctx.read_aspect_version(source_a, ASPECT_A)?.get(ASPECT_A);
            return Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(a, 0))
                    .with_output_identity(format!("geom-delta-{a}"))
                    .with_continuity_token("geom-delta"),
            ));
        }
        if node == filtered_gate {
            let b = ctx.read_aspect_version(source_b, ASPECT_B)?.get(ASPECT_B);
            return Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(0, b))
                    .with_output_identity(format!("geom-filter-{b}"))
                    .with_continuity_token("geom-filter"),
            ));
        }
        if node == demand_gate {
            let a = ctx.read_aspect_version(source_a, ASPECT_A)?.get(ASPECT_A);
            return Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(a, 0))
                    .with_output_identity(format!("geom-demand-{a}"))
                    .with_continuity_token("geom-demand"),
            ));
        }
        if node == fused {
            let a = ctx.read_aspect_version(delta_gate, ASPECT_A)?.get(ASPECT_A);
            let b = ctx
                .read_aspect_version(filtered_gate, ASPECT_B)?
                .get(ASPECT_B);
            let demand = ctx
                .read_aspect_version(demand_gate, ASPECT_A)?
                .get(ASPECT_A);
            return Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(a.max(demand), b))
                    .with_output_identity(format!("geom-fused-{a}-{b}-{demand}"))
                    .with_continuity_token("geom-fused"),
            ));
        }
        Err(SignalError::invalid_input(format!(
            "unexpected geometry node {node}"
        )))
    }
}

pub(super) fn geometry_checked_evaluator(
    fixture: &GeometryFixture,
) -> impl for<'graph, 'work, 'run, 'lease> Fn(
    &mut crate::logic::checked_context::CheckedEvaluationContext<'graph, 'work, 'run, 'lease, ()>,
) -> Result<
    crate::logic::evaluation::EvaluationOutput,
    SignalError,
> + Sync {
    let (source_a, source_b, delta_gate, filtered_gate, demand_gate, fused) = (
        fixture.source_a,
        fixture.source_b,
        fixture.delta_gate,
        fixture.filtered_gate,
        fixture.demand_gate,
        fixture.fused,
    );
    move |ctx| {
        ctx.work()
            .checkpoint(128)
            .map_err(|_| SignalError::invalid_input("geometry kernel work exhausted"))?;
        let node = ctx.node();
        let (version, identity, continuity) = if node == delta_gate {
            let a = ctx.read(source_a, ASPECT_A)?;
            (version_ab(a, 0), format!("geom-delta-{a}"), "geom-delta")
        } else if node == filtered_gate {
            let b = ctx.read(source_b, ASPECT_B)?;
            (version_ab(0, b), format!("geom-filter-{b}"), "geom-filter")
        } else if node == demand_gate {
            let a = ctx.read(source_a, ASPECT_A)?;
            (version_ab(a, 0), format!("geom-demand-{a}"), "geom-demand")
        } else if node == fused {
            let a = ctx.read(delta_gate, ASPECT_A)?;
            let b = ctx.read(filtered_gate, ASPECT_B)?;
            let demand = ctx.read(demand_gate, ASPECT_A)?;
            (
                version_ab(a.max(demand), b),
                format!("geom-fused-{a}-{b}-{demand}"),
                "geom-fused",
            )
        } else {
            return Err(SignalError::invalid_input(format!(
                "unexpected geometry node {node}"
            )));
        };
        Ok(ctx.finish(
            NodeEvaluationResult::from_version(version)
                .with_output_identity(identity)
                .with_continuity_token(continuity),
        ))
    }
}

fn bounded(inputs: impl IntoIterator<Item = DeclaredSignalInput>) -> NodeContract {
    NodeContract::wildcard().with_bounded_inputs(BoundedSignalInputs::new(inputs))
}

pub(super) fn build_geometry_fixture(policy: SignalRuntimePolicy) -> GeometryFixture {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(policy);

    let source_a = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .output_identity()
        .partitioned_output()
        .build();
    let source_b = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .output_identity()
        .partitioned_output()
        .build();
    let delta_gate = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([DeclaredSignalInput::new(source_a, ASPECT_A)]))
        .reads_aspects(mask_a())
        .delta_threshold(2.0)
        .build();
    let filtered_gate = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([DeclaredSignalInput::new(source_b, ASPECT_B)]))
        .reads_aspects(mask_b())
        .aspect_filter(mask_b())
        .build();
    let demand_gate = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([DeclaredSignalInput::new(source_a, ASPECT_A)]))
        .on_demand()
        .build();
    let fused = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([
            DeclaredSignalInput::new(delta_gate, ASPECT_A),
            DeclaredSignalInput::new(filtered_gate, ASPECT_B),
            DeclaredSignalInput::new(demand_gate, ASPECT_A),
        ]))
        .output_identity()
        .build();

    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_partition_detail_dependency(delta_gate, source_a, ASPECT_A, "wing", "left")
        .unwrap()
        .append_partition_dependency(filtered_gate, source_b, ASPECT_B, "lod")
        .unwrap()
        .append_dependency(demand_gate, source_a, ASPECT_A)
        .unwrap()
        .append_dependency(fused, delta_gate, ASPECT_A)
        .unwrap()
        .append_dependency(fused, filtered_gate, ASPECT_B)
        .unwrap()
        .append_dependency(fused, demand_gate, ASPECT_A)
        .unwrap();
    dependencies.commit().unwrap();

    let family = define_keyed_computation(&mut runtime, "geom-workflow", ());
    let keyed_def = family.keyed("feature-panel");
    let keyed = keyed_def.node(&mut runtime);
    let memo_key = keyed_def.memoized("mesh-v1");

    GeometryFixture {
        runtime,
        source_a,
        source_b,
        delta_gate,
        filtered_gate,
        demand_gate,
        fused,
        keyed,
        memo_key,
    }
}

pub(super) fn seed_geometry_baseline(
    fixture: &mut GeometryFixture,
    model: &mut ReferenceModel,
) -> (SignalBranchHandle, SignalSnapshotV1) {
    // This standalone caller declares the operational serial memory policy.
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        worth_execution::CancellationToken::new(),
        None,
    );
    let request_execution = worth_execution::ExecutionRequest::serial(&serial_request);

    let mut ctx = ();
    fixture
        .runtime
        .transaction(request_execution, &mut ctx, |tx: &mut DefaultTx<'_>| {
            tx.read(fixture.source_a, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("geom-source-a-1"),
                ))
            })?;
            tx.read(fixture.source_b, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(0, 1))
                        .with_output_identity("geom-source-b-1"),
                ))
            })?;
            tx.evaluate_keyed(fixture.keyed, &fixture.memo_key, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 1))
                        .with_output_identity("geom-keyed-1")
                        .with_output_change(OutputChange::Refreshed),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    fixture
        .runtime
        .evaluate_dirty(&(), &geometry_evaluator(fixture))
        .unwrap();
    fixture
        .runtime
        .evaluate_with_plan(
            fixture.demand_gate,
            &(),
            &geometry_evaluator(fixture),
            EvaluationRequestMode::ForceOnDemand,
        )
        .unwrap();

    let main = fixture.runtime.observe().current_branch();
    model.active = main.id;
    model.branches.insert(
        main.id,
        BranchTruth {
            a: 1,
            b: 1,
            head_snapshot: None,
        },
    );
    let snapshot = fixture
        .runtime
        .capture_snapshot()
        .expect("snapshot capture should succeed without managed queue bindings");
    let mut truth = model.branch(main.id).clone();
    truth.head_snapshot = Some(snapshot.meta.snapshot_id);
    model.snapshots.insert(snapshot.meta.snapshot_id, truth);
    model.branch_mut(main.id).head_snapshot = Some(snapshot.meta.snapshot_id);
    (main, snapshot)
}
