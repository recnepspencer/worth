use crate::facade::{
    BoundedSignalInputs, DeclaredSignalInput, KeyedComputation, NodeContract, NodeEvaluationResult,
    NodeId, OutputChange, SignalBranchHandle, SignalError, SignalGraph, SignalRuntime,
    SignalRuntimePolicy, SignalSnapshotV1, SignalTransaction,
};
use crate::tests::support::{
    define_keyed_computation, mask_a, mask_b, version_ab, DependencyBatchBuilder, ASPECT_A,
    ASPECT_B,
};

use super::workflow_truth::{BranchTruth, ReferenceModel};

type DefaultRuntime = SignalRuntime<(), (), (), (), ()>;
type DefaultTx<'a> = SignalTransaction<'a, (), (), (), (), ()>;

pub(super) struct FintechFixture {
    pub(super) runtime: DefaultRuntime,
    pub(super) ticks: NodeId,
    pub(super) volatility: NodeId,
    pub(super) throttle: NodeId,
    pub(super) alert: NodeId,
    pub(super) risk: NodeId,
    pub(super) keyed: NodeId,
    pub(super) memo_key: KeyedComputation,
}

pub(super) fn fintech_evaluator(
    fixture: &FintechFixture,
) -> impl for<'ctx> Fn(
    &mut crate::logic::context::EvaluationContext<'ctx, ()>,
) -> Result<crate::logic::evaluation::EvaluationOutput, SignalError>
       + Sync {
    let ticks = fixture.ticks;
    let volatility = fixture.volatility;
    let throttle = fixture.throttle;
    let alert = fixture.alert;
    let risk = fixture.risk;
    move |ctx| {
        let node = ctx.node();
        if node == throttle {
            let a = ctx.read_aspect_version(ticks, ASPECT_A)?.get(ASPECT_A);
            return Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(a, 0))
                    .with_output_identity(format!("throttle-{a}"))
                    .with_continuity_token("throttle"),
            ));
        }
        if node == alert {
            let b = ctx.read_aspect_version(volatility, ASPECT_B)?.get(ASPECT_B);
            return Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(0, b))
                    .with_output_identity(format!("alert-{b}"))
                    .with_continuity_token("alert"),
            ));
        }
        if node == risk {
            let a = ctx.read_aspect_version(throttle, ASPECT_A)?.get(ASPECT_A);
            let b = ctx.read_aspect_version(alert, ASPECT_B)?.get(ASPECT_B);
            return Ok(ctx.finish(
                NodeEvaluationResult::from_version(version_ab(a, b))
                    .with_output_identity(format!("risk-{a}-{b}"))
                    .with_continuity_token("risk-surface"),
            ));
        }
        Err(SignalError::invalid_input(format!(
            "unexpected fintech node {node}"
        )))
    }
}

pub(super) fn fintech_checked_evaluator(
    fixture: &FintechFixture,
) -> impl for<'graph, 'work, 'run, 'lease> Fn(
    &mut crate::logic::checked_context::CheckedEvaluationContext<'graph, 'work, 'run, 'lease, ()>,
) -> Result<
    crate::logic::evaluation::EvaluationOutput,
    SignalError,
> + Sync {
    let (ticks, volatility, throttle, alert, risk) = (
        fixture.ticks,
        fixture.volatility,
        fixture.throttle,
        fixture.alert,
        fixture.risk,
    );
    move |ctx| {
        ctx.work()
            .checkpoint(128)
            .map_err(|_| SignalError::invalid_input("fintech kernel work exhausted"))?;
        let node = ctx.node();
        let (version, identity, continuity) = if node == throttle {
            let a = ctx.read(ticks, ASPECT_A)?;
            (version_ab(a, 0), format!("throttle-{a}"), "throttle")
        } else if node == alert {
            let b = ctx.read(volatility, ASPECT_B)?;
            (version_ab(0, b), format!("alert-{b}"), "alert")
        } else if node == risk {
            let a = ctx.read(throttle, ASPECT_A)?;
            let b = ctx.read(alert, ASPECT_B)?;
            (version_ab(a, b), format!("risk-{a}-{b}"), "risk-surface")
        } else {
            return Err(SignalError::invalid_input(format!(
                "unexpected fintech node {node}"
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

pub(super) fn build_fintech_fixture(policy: SignalRuntimePolicy) -> FintechFixture {
    let mut runtime = SignalRuntime::builder(SignalGraph::new())
        .with_kernel_defaults()
        .build();
    runtime.set_runtime_policy(policy);

    let ticks = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .output_identity()
        .build();
    let volatility = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([]))
        .output_identity()
        .build();
    let throttle = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([DeclaredSignalInput::new(ticks, ASPECT_A)]))
        .reads_aspects(mask_a())
        .delta_threshold(2.0)
        .build();
    let alert = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([DeclaredSignalInput::new(volatility, ASPECT_B)]))
        .reads_aspects(mask_b())
        .aspect_filter(mask_b())
        .build();
    let risk = runtime
        .graph_mut()
        .node()
        .with_contract(bounded([
            DeclaredSignalInput::new(throttle, ASPECT_A),
            DeclaredSignalInput::new(alert, ASPECT_B),
        ]))
        .output_identity()
        .build();

    let mut dependencies = DependencyBatchBuilder::new(runtime.graph_mut());
    dependencies
        .append_dependency(throttle, ticks, ASPECT_A)
        .unwrap()
        .append_dependency(alert, volatility, ASPECT_B)
        .unwrap()
        .append_dependency(risk, throttle, ASPECT_A)
        .unwrap()
        .append_dependency(risk, alert, ASPECT_B)
        .unwrap();
    dependencies.commit().unwrap();

    let family = define_keyed_computation(&mut runtime, "fintech-workflow", ());
    let keyed_def = family.keyed("risk-book");
    let keyed = keyed_def.node(&mut runtime);
    let memo_key = keyed_def.memoized("risk-v1");

    FintechFixture {
        runtime,
        ticks,
        volatility,
        throttle,
        alert,
        risk,
        keyed,
        memo_key,
    }
}

pub(super) fn seed_fintech_baseline(
    fixture: &mut FintechFixture,
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
            tx.read(fixture.ticks, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 0))
                        .with_output_identity("tick-1"),
                ))
            })?;
            tx.read(fixture.volatility, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(0, 1))
                        .with_output_identity("vol-1"),
                ))
            })?;
            tx.evaluate_keyed(fixture.keyed, &fixture.memo_key, &|view| {
                Ok(view.finish(
                    NodeEvaluationResult::from_version(version_ab(1, 1))
                        .with_output_identity("risk-keyed-1")
                        .with_output_change(OutputChange::Refreshed),
                ))
            })?;
            Ok(())
        })
        .unwrap();

    fixture
        .runtime
        .evaluate_dirty(&(), &fintech_evaluator(fixture))
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
