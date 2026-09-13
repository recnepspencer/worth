use std::panic::{catch_unwind, AssertUnwindSafe};
use std::rc::Rc;

use worth_proof::TransitionOutcome;

use crate::branch::SignalBranchRetirementReason;
use crate::data::checkpoint::CheckpointBarrier;
use crate::data::error::SignalError;
use crate::data::event_subscriber::{EventSubscriber, SubscriberId};
use crate::data::graph::SignalGraph;
use crate::data::resource::{
    ResourceNodeDeclaration, ResourceNodeId, ResourcePayloadContract, ResourcePayloadContractId,
    ResourceRequestIntent,
};
use crate::data::subscriber_context::SubscriberContext;
use crate::logic::transaction::{
    ObservationListener, ObservationNotice, ObservationPolicy, ObservationReadContext,
    ObservedNodeSet, SignalRuntime,
};

use super::super::SignalOwnerServiceIssuanceDenial;

struct EmptySubscriber;

impl EventSubscriber for EmptySubscriber {
    type Event = ();
    type DataId = ();
    type RuntimeContext = ();

    fn id(&self) -> SubscriberId {
        SubscriberId::new(91)
    }

    fn name(&self) -> &'static str {
        "phase-3-empty-subscriber"
    }

    fn requires(&self) -> &'static [()] {
        &[]
    }

    fn provides(&self) -> &'static [()] {
        &[]
    }

    fn on_event(&mut self, _event: &()) {}

    fn on_checkpoint(
        &mut self,
        _barrier: CheckpointBarrier,
        _ctx: &mut SubscriberContext<()>,
        _runtime: &mut (),
    ) -> Result<(), SignalError> {
        Ok(())
    }
}

struct EmptyObservationListener;

struct ThreadSafeEffect;
struct ThreadSafeContext;

impl ObservationListener<(), (), (), (), ()> for EmptyObservationListener {
    fn on_observation(
        &self,
        _ctx: ObservationReadContext<'_, (), (), (), (), ()>,
        _notice: &ObservationNotice<'_>,
    ) {
    }
}

#[test]
fn concrete_callback_states_deny_sealing_without_partial_construction() {
    let mut subscriber_runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    subscriber_runtime
        .event_bus_mut()
        .subscribe(Box::new(EmptySubscriber))
        .expect("test subscriber registers");
    assert!(matches!(
        subscriber_runtime.owner_port_slots(),
        Err(SignalOwnerServiceIssuanceDenial::EventSubscriberStateConfigured)
    ));

    let mut observation_runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let observation_handle = observation_runtime.observations_mut().register_nodes(
        ObservationPolicy::default(),
        ObservedNodeSet::default(),
        Box::new(EmptyObservationListener),
    );
    assert!(matches!(
        observation_runtime.owner_port_slots(),
        Err(SignalOwnerServiceIssuanceDenial::ObservationRegistrationStateConfigured)
    ));
    assert!(observation_runtime
        .observations_mut()
        .unsubscribe(observation_handle));
    observation_runtime
        .owner_port_slots()
        .expect("the same denied root seals after its registration is removed");

    let mut healthy = SignalRuntime::build_for::<()>(SignalGraph::new());
    healthy
        .owner_port_slots()
        .expect("denied roots leave no shared construction residue");
    assert!(catch_unwind(AssertUnwindSafe(|| {
        let _ = healthy.event_bus_mut();
    }))
    .is_err());
}

#[test]
fn concrete_managed_queue_state_denies_sealing_before_partition() {
    let mut graph = SignalGraph::new();
    let node = graph.node().build();
    let mut runtime = SignalRuntime::build_for::<()>(graph);
    runtime
        .declare_resource_node(ResourceNodeDeclaration::new(
            ResourceNodeId::from_node(node),
            ResourcePayloadContract::new(ResourcePayloadContractId::new(17)),
        ))
        .expect("the test resource declaration lowers");
    let admitted = runtime
        .admit_resource_request(ResourceRequestIntent::new(ResourceNodeId::from_node(node)))
        .expect("the test resource request admits")
        .admitted_request();
    let queue = runtime
        .bind_resource_managed_queue(admitted, 1)
        .expect("the admitted request binds one managed queue");

    assert!(matches!(
        runtime.owner_port_slots(),
        Err(
            SignalOwnerServiceIssuanceDenial::ManagedQueueStateConfigured {
                bound_queue_count: 1
            }
        )
    ));
    runtime
        .enqueue_resource_managed_queue(&queue, 1)
        .expect("the denied root retains its exact queue binding");
}

#[test]
fn private_slot_accepts_composition_types_while_local_preflight_remains_separate() {
    fn issue_composition_slots<E, Ctx>(runtime: &mut SignalRuntime<(), (), E, Ctx, ()>)
    where
        E: Send + Sync + 'static,
        Ctx: Send + Sync + 'static,
    {
        runtime
            .owner_port_slots()
            .expect("composition-capable effect and context issue the private slots");
    }

    let mut composition_runtime = SignalRuntime::builder(SignalGraph::new())
        .with_events::<ThreadSafeEffect>()
        .with_context::<ThreadSafeContext>()
        .with_kernel_defaults()
        .build();
    issue_composition_slots(&mut composition_runtime);

    let local_runtime = SignalRuntime::builder(SignalGraph::new())
        .with_events::<Rc<()>>()
        .with_context::<Rc<()>>()
        .with_kernel_defaults()
        .build();
    assert_eq!(
        local_runtime.owner_service_issuance_capability(),
        Ok(()),
        "local-only runtimes remain valid; their exclusion is the slot's compile-time fence"
    );
}

#[test]
fn overfull_preseal_retirement_history_denies_partition_without_consuming_runtime() {
    const MAXIMUM_RETAINED_RECEIPTS: usize = 4_096;

    let mut runtime = SignalRuntime::build_for::<()>(SignalGraph::new());
    let selected = runtime.current_branch();
    let selected_basis = runtime
        .observe_signal_branch_basis(selected.clone())
        .expect("the selected source admits once for sequential retirement history");
    let mut final_retired_branch = None;
    for index in 0..=MAXIMUM_RETAINED_RECEIPTS {
        let (branch, basis) = runtime
            .fork_signal_branch(format!("receipt-history-{index}"), &selected_basis)
            .expect("one bounded sibling forks")
            .into_parts();
        let plan = match runtime.plan_signal_branch_retirement(
            branch.clone(),
            basis,
            SignalBranchRetirementReason::Superseded,
        ) {
            TransitionOutcome::Success(plan) => plan,
            other => panic!("the sequential sibling retirement plans: {other:?}"),
        };
        assert!(matches!(
            runtime.retire_signal_branch(plan),
            TransitionOutcome::Success(_)
        ));
        final_retired_branch = Some(branch);
    }
    let final_retired_branch = final_retired_branch.expect("the history is populated");

    assert!(matches!(
        runtime.owner_port_slots(),
        Err(
            SignalOwnerServiceIssuanceDenial::RetirementReceiptCapacityExhausted {
                maximum_retained_receipts: MAXIMUM_RETAINED_RECEIPTS,
            }
        )
    ));
    assert_eq!(runtime.current_branch(), selected);
    assert_eq!(
        runtime
            .branch_retirement_receipt(final_retired_branch.id)
            .map(|receipt| receipt.retired_branch().id),
        Some(final_retired_branch.id),
        "typed partition denial preserves the exact unsealed retirement history"
    );
}
