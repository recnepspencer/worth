//! Genuine sealed-owner lifecycle and original-allocation evidence.
use super::*;
use crate::branch::{
    AdmittedSignalBranchBasis, SignalBranchBasisPort, SignalOwnerLifecycleObservation,
};
use crate::data::graph::SignalGraph;
use crate::logic::transaction::SignalRuntime;
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
type Runtime = SignalRuntime<(), (), (), (), ()>;
fn policy() -> SerialRequest {
    SerialRequest::from_memory(
        SerialMemoryBudget::new(
            crate::runtime_policy::SignalRuntimePolicy::operational().serial_memory_bytes,
        ),
        CancellationToken::new(),
        None,
    )
}
fn sealed(
    graph: SignalGraph,
) -> (
    Runtime,
    SignalBranchBasisPort<(), (), ()>,
    AdmittedSignalBranchBasis,
) {
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    let basis = runtime
        .observe_signal_branch_basis(runtime.current_branch())
        .unwrap();
    let (port, _, _) = runtime.owner_port_slots().unwrap();
    (runtime, port, basis)
}
fn busy(runtime: Runtime, request: &ExecutionRequest<'_, '_>) -> Runtime {
    let refused = runtime.try_prepare_initial_empty_stop(request).unwrap_err();
    assert_eq!(refused.reason(), Denial::Busy);
    refused.into_runtime()
}
#[test]
fn preparation_resume_preserves_actual_basis_and_service_execution() {
    let (runtime, port, basis) = sealed(SignalGraph::new());
    let policy = policy();
    let request = ExecutionRequest::serial(&policy);
    let descriptor = basis.descriptor().clone();
    let prepared = runtime.try_prepare_initial_empty_stop(&request).unwrap();
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    assert!(port.compare_current_exact(&basis).is_err());
    let runtime = prepared.resume().unwrap();
    assert_eq!(basis.descriptor(), &descriptor);
    port.compare_current_exact(&basis).unwrap();
    // Execute a real canonical transaction after resuming the original owner.
    let owner = port.upgrade_owner().unwrap();
    let admission = owner.admit().unwrap();
    let cell = owner
        .lookup_cell(&admission, basis.owner_branch_id())
        .unwrap();
    let cancellation = crate::branch::SignalOwnerCancellationSource::new();
    let result = cell
        .advance_exact::<(), (), _>(
            ExecutionRequest::serial(&policy),
            &admission,
            &basis,
            &mut (),
            &cancellation.token(),
            |_| Ok(()),
        )
        .unwrap();
    assert_eq!(result.into_parts().0.generation().get(), 1);
    drop(admission);
    let refused = runtime
        .try_prepare_initial_empty_stop(&request)
        .unwrap_err();
    assert_eq!(refused.reason(), Denial::Unsupported);
}
#[test]
fn every_actual_proof_lock_busy_preserves_the_same_open_runtime() {
    let (mut runtime, port, basis) = sealed(SignalGraph::new());
    let owner = port.upgrade_owner().unwrap();
    let admission = owner.admit().unwrap();
    let cell = owner
        .lookup_cell(&admission, basis.owner_branch_id())
        .unwrap();
    drop(admission);
    let policy = policy();
    let request = ExecutionRequest::serial(&policy);
    let witness = owner.initial_empty_root.get().unwrap();
    macro_rules! held_busy {
        ($guard:expr) => {{
            let held = $guard.unwrap();
            runtime = busy(runtime, &request);
            assert_eq!(
                port.owner_lifecycle_observation(),
                SignalOwnerLifecycleObservation::Open
            );
            drop(held);
        }};
    }
    held_busy!(owner.lifecycle.try_initial_empty_stop_admission());
    held_busy!(owner.registry.try_initial_empty_stop_cell(witness.branch));
    held_busy!(cell.try_initial_empty_stop_guard(witness.incarnation, &witness.observation));
    held_busy!(owner.metadata.try_initial_empty_stop_guard());
    held_busy!(owner.retention.try_initial_empty_stop_guard());
    held_busy!(owner.conditional_retention.try_initial_empty_stop_guard());
    held_busy!(owner.conditional_temporal.try_initial_empty_stop_guard());
    port.compare_current_exact(&basis).unwrap();
    runtime
        .try_prepare_initial_empty_stop(&request)
        .unwrap()
        .resume()
        .unwrap();
}
#[test]
fn competing_real_admission_defeats_the_final_zero_count_cas() {
    let (runtime, port, basis) = sealed(SignalGraph::new());
    let owner = port.upgrade_owner().unwrap();
    let final_cut = owner.lifecycle.try_initial_empty_stop_admission().unwrap();
    // Service admission does not acquire transition_gate; this is the exact
    // competing increment which can occur while the owner checks its fields.
    let admission = owner.admit().unwrap();
    assert_eq!(final_cut.begin(), Err(Denial::Busy));
    let policy = policy();
    let request = ExecutionRequest::serial(&policy);
    let runtime = busy(runtime, &request);
    drop(admission);
    port.compare_current_exact(&basis).unwrap();
    runtime
        .try_prepare_initial_empty_stop(&request)
        .unwrap()
        .resume()
        .unwrap();
}
#[test]
fn commit_retains_real_sealed_service_custody_until_explicit_reclaim() {
    let mut graph = SignalGraph::new();
    let claimant = crate::data::aspect::SignalAspectLoweringOwner::fresh();
    graph.claim_aspect_lowering_owner(&claimant).unwrap();
    let probe = graph.lifecycle_probe();
    let (runtime, port, basis) = sealed(graph);
    let source = worth_proof::ConditionalSourceObservationOwner::fresh();
    let conditional = runtime
        .issue_conditional_execution_service(&basis, &claimant, &source.authority())
        .unwrap();
    let owner = port.upgrade_owner().unwrap();
    let policy = policy();
    let request = ExecutionRequest::serial(&policy);
    let prepared = runtime.try_prepare_initial_empty_stop(&request).unwrap();
    // Actual nonzero issuance/basis custody is retained, never classified empty.
    let held = owner.retention.try_initial_empty_stop_guard().unwrap();
    let (prepared, reason) = prepared.commit(&request).unwrap_err();
    assert_eq!(reason, Denial::Busy);
    drop(held);
    let stopped = prepared.commit(&request).unwrap();
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    assert!(port.compare_current_exact(&basis).is_err());
    drop(conditional);
    drop(basis);
    drop(owner);
    assert!(probe.is_live());
    stopped.reclaim();
    assert!(!probe.is_live());
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closed
    );
}
#[test]
fn original_stop_preserves_open_or_prepared_custody_without_cleanup() {
    let (runtime, port, basis) = sealed(SignalGraph::new());
    let cancel = worth_execution::CancellationSource::new();
    let cancelled =
        SerialRequest::from_memory(SerialMemoryBudget::new(Some(1 << 20)), cancel.token(), None);
    cancel.cancel();
    let stopped = ExecutionRequest::serial(&cancelled);
    let refused = runtime
        .try_prepare_initial_empty_stop(&stopped)
        .unwrap_err();
    assert!(matches!(refused.reason(), Denial::Stopped(_)));
    let runtime = refused.into_runtime();
    port.compare_current_exact(&basis).unwrap();
    let policy = policy();
    let request = ExecutionRequest::serial(&policy);
    let prepared = runtime.try_prepare_initial_empty_stop(&request).unwrap();
    let (prepared, reason) = prepared.commit(&stopped).unwrap_err();
    assert!(matches!(reason, Denial::Stopped(_)));
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Closing
    );
    let runtime = prepared.resume().unwrap();
    port.compare_current_exact(&basis).unwrap();
    runtime
        .try_prepare_initial_empty_stop(&request)
        .unwrap()
        .commit(&request)
        .unwrap()
        .reclaim();
}
#[test]
fn nonempty_or_reconstructed_graphs_and_multiple_branches_refuse_before_stop() {
    let mut allocated = SignalGraph::new();
    let node = allocated.node().build();
    allocated.rollback_created_nodes(&[node]);
    let policy = policy();
    let request = ExecutionRequest::serial(&policy);
    for graph in [allocated, SignalGraph::new().clone()] {
        let (runtime, port, basis) = sealed(graph);
        let refused = runtime
            .try_prepare_initial_empty_stop(&request)
            .unwrap_err();
        assert_eq!(refused.reason(), Denial::Unsupported);
        port.compare_current_exact(&basis).unwrap();
        drop(refused.into_runtime());
    }
    let (mut runtime, _, basis) = sealed(SignalGraph::new());
    runtime.fork_signal_branch("later", &basis).unwrap();
    let refused = runtime
        .try_prepare_initial_empty_stop(&request)
        .unwrap_err();
    assert_eq!(refused.reason(), Denial::Unsupported);
}

#[test]
fn poisoned_owner_metadata_refuses_without_stopping_or_losing_the_root() {
    let (runtime, port, _basis) = sealed(SignalGraph::new());
    let owner = port.upgrade_owner().unwrap();
    let poison = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _held = owner.metadata.try_initial_empty_stop_guard().unwrap();
        panic!("poison original owner metadata");
    }));
    assert!(poison.is_err());
    let policy = policy();
    let request = ExecutionRequest::serial(&policy);
    let refused = runtime
        .try_prepare_initial_empty_stop(&request)
        .unwrap_err();
    assert_eq!(refused.reason(), Denial::Poisoned);
    assert_eq!(
        port.owner_lifecycle_observation(),
        SignalOwnerLifecycleObservation::Open
    );
    drop(refused.into_runtime());
}
