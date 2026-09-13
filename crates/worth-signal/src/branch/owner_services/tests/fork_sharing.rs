use std::env;
use std::hint::black_box;
use std::process::Command;

use crate::branch::validate_signal_branch_name;
use crate::data::aspect::Aspect;
use crate::data::checkpoint::CheckpointBarrier;
use crate::data::dependency::DependencyEdge;
use crate::data::dirty_set::BatchedDirtySet;
use crate::data::effect_mapping::EffectMapping;
use crate::data::graph::SignalGraph;
use crate::data::node::{NodeHotData, NodeState};
use crate::data::proof::DependencyBatchEdit;
use crate::data::resource::{ResourceNodeDeclaration, ResourceNodeId, ResourcePayloadContract};
use crate::data::temporal::{ClockTick, TemporalCondition};
use crate::facade::{
    AspectVersion, EvaluationContext, NodeEvaluationResult, Recipe, SignalError, SignalRuntime,
};
use stats_alloc::{Region, INSTRUMENTED_SYSTEM};

use super::super::SignalOwnerCancellationSource;
use super::runtime_root::runtime_with_two_branches_from_graph;

struct ForkCheckpointEffect;

impl EffectMapping for ForkCheckpointEffect {
    type Domain = ();
    type Effect = ();
    type Impact = ();

    fn route(_: &Self::Effect, sink: &mut BatchedDirtySet<Self::Domain, Self::Impact>) {
        sink.mark_domain_global(());
    }
}

pub(in crate::branch::owner_services) fn seed_nonempty_persistent_branch_state(
    runtime: &mut SignalRuntime<(), (), (), (), ()>,
    resource_node: crate::data::handle::NodeId,
) {
    runtime.set_node_tier(resource_node, ());
    runtime.set_domain_checkpoint_barrier((), CheckpointBarrier::PerCommit);
    runtime
        .schedule_temporal_wake(
            TemporalCondition::after(3).expect("positive delay"),
            ClockTick::new(3),
        )
        .expect("temporal fork fixture schedules");
    runtime
        .declare_resource_node(ResourceNodeDeclaration::new(
            ResourceNodeId::from_node(resource_node),
            ResourcePayloadContract::default(),
        ))
        .expect("resource fork fixture declares");
    let computation = runtime
        .define(Recipe::new(
            "persistent-fork-probe",
            (),
            |view: &mut EvaluationContext<'_, ()>| {
                Ok::<_, SignalError>(
                    view.finish(NodeEvaluationResult::from_version(AspectVersion::zero())),
                )
            },
        ))
        .expect("config fork fixture defines computation");
    let keyed = computation.keyed("persistent-key");
    let keyed_node = keyed.node(runtime);
    let mut context = ();
    runtime
        .transaction(&mut context, |transaction| {
            transaction.record_effect::<ForkCheckpointEffect>(&());
            keyed.evaluate_memoized(transaction, "persistent-memo")?;
            Ok(())
        })
        .expect("checkpoint and memo fork fixture commits");
    assert!(runtime.graph().is_alive(keyed_node));
}

#[test]
fn exact_fork_and_first_write_have_no_node_count_allocation_slope() {
    const CHILD_PROCESS: &str = "WORTH_SIGNAL_FORK_ALLOCATION_CHILD";
    const TEST_NAME: &str = "branch::owner_services::tests::fork_sharing::exact_fork_and_first_write_have_no_node_count_allocation_slope";
    if env::var_os(CHILD_PROCESS).is_none() {
        let output = Command::new(env::current_exe().expect("test executable resolves"))
            .arg("--exact")
            .arg(TEST_NAME)
            .arg("--nocapture")
            .arg("--test-threads=1")
            .env(CHILD_PROCESS, "1")
            .output()
            .expect("isolated allocation-probe process starts");
        let stdout = String::from_utf8_lossy(&output.stdout);
        print!("{stdout}");
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
        assert!(
            output.status.success(),
            "isolated allocation-probe process failed"
        );
        assert!(
            stdout.contains(TEST_NAME) && stdout.contains("test result: ok. 1 passed; 0 failed;"),
            "isolated allocation probe must execute exactly one named test"
        );
        return;
    }

    let mut allocation_samples = Vec::new();
    let mut first_write_samples = Vec::new();
    let mut eager_copy_bytes = None;
    for node_count in [64, 4_096, 65_536] {
        let mut graph = SignalGraph::new();
        let nodes = (0..node_count)
            .map(|_| graph.create_node())
            .collect::<Vec<_>>();
        let topology = DependencyBatchEdit::from_pairs(
            nodes
                .windows(2)
                .map(|pair| (pair[1], vec![DependencyEdge::new(pair[0], Aspect::new(1))])),
        );
        graph
            .apply_dependency_batch_edit(&topology)
            .expect("scale probe topology installs");
        let changed_node = nodes[node_count / 2];
        graph
            .set_node_state(changed_node, NodeState::Clean)
            .expect("first-write probe starts from known source state");

        let (mut runtime, source_branch, _, _) = runtime_with_two_branches_from_graph(graph);
        seed_nonempty_persistent_branch_state(&mut runtime, nodes[0]);
        let source_basis = runtime
            .observe_signal_branch_basis(source_branch.clone())
            .expect("seeded scale source observes");
        let (_, mutation, _) = runtime.owner_port_slots().expect("runtime seals");
        let owner = mutation.upgrade_owner().expect("owner remains live");
        let source_admission = owner.admit().expect("source inspection admits");
        let source_cell = owner
            .lookup_cell(&source_admission, source_branch.id)
            .expect("source cell is live");
        if node_count == 4_096 {
            let deep_clone_region = Region::new(&INSTRUMENTED_SYSTEM);
            source_cell
                .with_state(&source_admission, |source, _| {
                    black_box(source.state().graph().clone());
                })
                .expect("eager-copy sensitivity probe admits");
            eager_copy_bytes = Some(deep_clone_region.change().bytes_allocated);
        }
        let requested_identity = validate_signal_branch_name(format!("scale-fork-{node_count}"))
            .expect("scale identity validates");
        let before = owner.cost_snapshot();
        let region = Region::new(&INSTRUMENTED_SYSTEM);
        let ready = owner
            .reserve_fork_output(&source_admission, &source_cell)
            .expect("fork output retention reserves")
            .fork(
                &source_basis,
                requested_identity,
                &SignalOwnerCancellationSource::new().token(),
            )
            .expect("persistent destination installs");
        let (destination_handle, destination_basis) = ready.into_destination_parts();
        let destination = owner
            .lookup_cell(&source_admission, destination_handle.id)
            .expect("the handed-off destination is installed");
        let allocation = region.change();
        allocation_samples.push((
            node_count,
            allocation.allocations,
            allocation.bytes_allocated,
        ));
        let after = owner.cost_snapshot();
        assert_eq!(
            after.forked_mutable_graph_nodes_copied(),
            before.forked_mutable_graph_nodes_copied(),
            "node count {node_count} must not create mutable-node copy work"
        );

        let destination_admission = owner.admit().expect("destination inspection admits");
        let source_identity = source_cell
            .with_state(&source_admission, |source, _| {
                source.state().persistent_identity()
            })
            .expect("source identity capture admits");
        let destination_identity = destination
            .with_state(&destination_admission, |destination, _| {
                destination.state().persistent_identity()
            })
            .expect("destination identity capture admits");
        let sharing = source_identity.sharing_with(&destination_identity);
        assert!(sharing.graph.arena_root_shared, "arena size {node_count}");
        assert!(
            sharing.graph.topology_root_shared,
            "topology size {node_count}"
        );
        assert!(sharing.graph.cause_root_shared, "cause size {node_count}");
        assert!(sharing.config_roots_shared, "config size {node_count}");
        assert!(sharing.derived_roots_shared, "derived size {node_count}");

        let first_write_region = Region::new(&INSTRUMENTED_SYSTEM);
        destination
            .with_state(&destination_admission, |destination, _| {
                destination
                    .state_mut()
                    .graph_mut()
                    .set_node_state(changed_node, NodeState::Dirty)
                    .expect("destination first write is valid");
            })
            .expect("destination first write admits");
        let first_write = first_write_region.change();
        first_write_samples.push((
            node_count,
            first_write.allocations,
            first_write.bytes_allocated,
        ));
        destination
            .with_state(&destination_admission, |destination, _| {
                assert_eq!(
                    destination.state().graph().get_state(changed_node),
                    Ok(NodeState::Dirty),
                    "the measured destination first write must actually mutate"
                );
            })
            .expect("destination mutation remains inspectable");
        source_cell
            .with_state(&source_admission, |source, _| {
                assert_eq!(
                    source.state().graph().get_state(changed_node),
                    Ok(NodeState::Clean),
                    "destination first write must not mutate the source"
                );
            })
            .expect("source isolation remains inspectable");
        drop(destination_basis);
    }
    eprintln!("whole owner fork allocations by node scale: {allocation_samples:?}");
    eprintln!("destination first-write allocations by node scale: {first_write_samples:?}");
    eprintln!("eager 4096-node graph clone bytes: {eager_copy_bytes:?}");
    let minimum_calls = allocation_samples
        .iter()
        .map(|(_, calls, _)| *calls)
        .min()
        .expect("allocation cases exist");
    let minimum_bytes = allocation_samples
        .iter()
        .map(|(_, _, bytes)| *bytes)
        .min()
        .expect("allocation cases exist");
    let maximum_bytes = allocation_samples
        .iter()
        .map(|(_, _, bytes)| *bytes)
        .max()
        .expect("allocation cases exist");
    for (node_count, calls, bytes) in allocation_samples {
        assert!(
            calls <= minimum_calls + 8,
            "fork allocation calls slope with {node_count} nodes: {calls} vs {minimum_calls}"
        );
        assert!(
            bytes <= minimum_bytes + 8 * 1_024,
            "fork allocated bytes slope with {node_count} nodes: {bytes} vs {minimum_bytes}"
        );
    }
    let minimum_first_write_calls = first_write_samples
        .iter()
        .map(|(_, calls, _)| *calls)
        .min()
        .expect("first-write allocation cases exist");
    let minimum_first_write_bytes = first_write_samples
        .iter()
        .map(|(_, _, bytes)| *bytes)
        .min()
        .expect("first-write allocation cases exist");
    let maximum_first_write_bytes = first_write_samples
        .iter()
        .map(|(_, _, bytes)| *bytes)
        .max()
        .expect("first-write allocation cases exist");
    let one_page_byte_allowance = 64 * std::mem::size_of::<NodeHotData>() + 8 * 1_024;
    for (node_count, calls, bytes) in first_write_samples {
        assert!(
            calls <= minimum_first_write_calls + 8,
            "first-write allocation calls slope with {node_count} nodes: {calls} vs {minimum_first_write_calls}"
        );
        assert!(
            bytes <= minimum_first_write_bytes + one_page_byte_allowance,
            "first-write bytes exceed one bounded hot page with {node_count} nodes: {bytes} vs {minimum_first_write_bytes}"
        );
    }
    let eager_copy_bytes = eager_copy_bytes.expect("large eager-copy sensitivity case ran");
    assert!(
        eager_copy_bytes > minimum_bytes + 8 * 1_024,
        "the allocation bound must reject the real 4096-node eager clone: eager={eager_copy_bytes}, persistent_min={minimum_bytes}"
    );
    assert!(
        eager_copy_bytes > maximum_bytes,
        "the measured eager clone must allocate more than every persistent fork"
    );
    assert!(
        eager_copy_bytes > maximum_first_write_bytes + one_page_byte_allowance,
        "the first-write bound must reject a deferred whole-graph clone: eager={eager_copy_bytes}, first_write_max={maximum_first_write_bytes}"
    );
}
