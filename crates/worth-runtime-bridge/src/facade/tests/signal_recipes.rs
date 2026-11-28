use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use worth_signal::facade::runtime::{
    KeyedRecipe, ObservationListener, ObservationNotice, ObservationPolicy, ObservationReadContext,
    Recipe,
};
use worth_signal::facade::{
    Aspect, AspectVersion, AsyncNodeCapabilityDeclaration, AsyncNodePayloadContract,
    AsyncNodePayloadContractId, ChangedRegion, ClockAdvanceRequest, ClockDomain, ClockTick,
    EvaluationContext, ResourceCancellationReason, ScopePath, SignalError, VersionComparatorPolicy,
};

use super::{runtime, BridgeRuntimePolicy};
use crate::facade::BridgeTemporalCondition;

const ASPECT: Aspect = Aspect::new(0);

struct Listener(Arc<AtomicUsize>);

impl ObservationListener<(), (), (), (), ()> for Listener {
    fn on_observation(
        &self,
        _context: ObservationReadContext<'_, (), (), (), (), ()>,
        _notice: &ObservationNotice<'_>,
    ) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

fn evaluate(_: &mut EvaluationContext<'_, ()>) -> Result<AspectVersion, SignalError> {
    Ok(AspectVersion::from_updates([(ASPECT, 7)]))
}

#[test]
fn keyed_recipe_publishes_observations_and_reads_previous_value_from_a_live_wake() {
    let bridge = runtime(BridgeRuntimePolicy::development());
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let recipe = bridge
        .register_signal_recipe(Recipe::new("price", (), evaluate))
        .unwrap();
    let price = bridge
        .bind_signal_recipe(&recipe, KeyedRecipe::new("price", "price:1"))
        .unwrap();
    let other = bridge
        .bind_signal_recipe(&recipe, KeyedRecipe::new("price", "price:2"))
        .unwrap();
    assert_ne!(price.node(), other.node());

    let notices = Arc::new(AtomicUsize::new(0));
    let observer = bridge
        .observe_signal_recipe(
            &price,
            ObservationPolicy::recomputed(),
            Box::new(Listener(Arc::clone(&notices))),
        )
        .unwrap();
    bridge.evaluate_signal_recipe(&other, execution).unwrap();
    assert_eq!(notices.load(Ordering::Relaxed), 0);
    bridge.evaluate_signal_recipe(&price, execution).unwrap();
    assert_eq!(notices.load(Ordering::Relaxed), 1);

    let exact = ChangedRegion::exact(ScopePath::one("price").unwrap());
    let subtree = ChangedRegion::new("related-prices");
    let changed = bridge
        .invalidate_signal_recipe(&price, ASPECT, vec![exact.clone(), subtree.clone()])
        .unwrap();
    let regions = changed.changed_regions.as_slice();
    assert_eq!(regions.len(), 2);
    assert!(regions.contains(&exact));
    assert!(regions.contains(&subtree));

    let wake = bridge
        .schedule_signal_wake(BridgeTemporalCondition::at_or_after(3))
        .unwrap();
    assert!(bridge
        .evaluate_signal_recipe_with_previous(
            &price,
            &wake,
            execution,
            |_, _| -> Result<AspectVersion, SignalError> {
                panic!("an unready wake cannot open previous-value access")
            }
        )
        .is_err());
    bridge
        .advance_signal_clock(
            ClockAdvanceRequest::new(ClockDomain::MonotonicExecution, ClockTick::new(3)),
            std::num::NonZeroUsize::MIN,
        )
        .unwrap();
    let result = bridge
        .evaluate_signal_recipe_with_previous(&price, &wake, execution, |_, previous| {
            assert_eq!(
                previous.aspect_version(),
                AspectVersion::from_updates([(ASPECT, 7)])
            );
            Ok(AspectVersion::from_updates([(ASPECT, 8)]))
        })
        .unwrap();
    assert_eq!(
        result.temporal_evidence.previous_value_references[0].access_wake_id(),
        wake.id()
    );
    assert!(bridge.unobserve_signal_recipe(&observer).unwrap());
}

#[test]
fn keyed_and_interior_capabilities_use_the_same_signal_cancellation_hierarchy() {
    let bridge = runtime(BridgeRuntimePolicy::development());
    let recipe = bridge
        .register_signal_recipe(Recipe::new("price", (), evaluate))
        .unwrap();
    let parent = bridge
        .bind_signal_recipe(&recipe, KeyedRecipe::new("price", "parent"))
        .unwrap();
    let child = bridge
        .bind_signal_recipe(&recipe, KeyedRecipe::new("price", "child"))
        .unwrap();
    let payload = AsyncNodePayloadContract::new(AsyncNodePayloadContractId::new(6));
    let keyed = bridge
        .bind_signal_recipe_async_capability(&child, payload.clone())
        .unwrap();
    let interior = bridge
        .install_signal_recipe_async_capability(
            &parent,
            AsyncNodeCapabilityDeclaration::new(parent.node(), payload)
                .with_declared_dependent_cancellation_nodes([keyed.node()]),
        )
        .unwrap();
    let child_request = bridge
        .admit_signal_recipe_async_request(
            &child,
            bridge.signal_recipe_async_request_intent(&child).unwrap(),
        )
        .unwrap();
    assert!(child_request.report().resource_admission().is_some());

    let parent_request = bridge
        .admit_signal_recipe_async_request(&parent, interior.request_intent())
        .unwrap();
    let report = bridge
        .cancel_signal_recipe_async_request(
            &parent_request,
            ResourceCancellationReason::HostRequested,
        )
        .unwrap();
    assert_eq!(report.root_node(), parent.node());
    assert_eq!(report.affected_nodes(), &[parent.node(), child.node()]);
}

#[test]
fn recipe_comparator_override_controls_dependency_early_cutoff() {
    use std::sync::atomic::AtomicU64;

    let registration = super::support::native_profile_mapping_registration()
        .with_version_comparator(VersionComparatorPolicy::Tolerance { epsilon: 2 });
    let mapping = registration.mapping_id().clone();
    let bridge = crate::facade::RuntimeBridge::builder()
        .with_truth_source(super::StaticSource)
        .with_compute_sink(super::StaticSink)
        .register_mapping(registration)
        .build()
        .unwrap();
    let serial = crate::policy::BridgeExecutionPolicyBaseline::operational()
        .serial_request(worth_execution::CancellationToken::new(), None);
    let execution = worth_execution::ExecutionRequest::serial(&serial);
    let value = AtomicU64::new(7);
    let source = bridge
        .register_signal_recipe(Recipe::new(
            "source",
            (),
            |_: &mut EvaluationContext<'_, ()>| {
                Ok(AspectVersion::from_updates([(
                    ASPECT,
                    value.load(Ordering::Relaxed),
                )]))
            },
        ))
        .unwrap();
    let source = bridge
        .bind_signal_recipe(&source, KeyedRecipe::new("source", "price"))
        .unwrap();
    bridge.evaluate_signal_recipe(&source, execution).unwrap();

    let evaluate_consumer = |ctx: &mut EvaluationContext<'_, ()>| {
        Ok(AspectVersion::from_updates([(
            ASPECT,
            ctx.read(source.node(), ASPECT)?,
        )]))
    };
    let exact = bridge
        .register_signal_recipe(Recipe::new("exact", (), evaluate_consumer))
        .unwrap();
    let approx = bridge
        .register_mapped_signal_recipe(&mapping, Recipe::new("approx", (), evaluate_consumer))
        .unwrap();
    let exact = bridge
        .bind_signal_recipe(&exact, KeyedRecipe::new("exact", "price"))
        .unwrap();
    let approx = bridge
        .bind_signal_recipe(&approx, KeyedRecipe::new("approx", "price"))
        .unwrap();
    bridge.evaluate_signal_recipe(&exact, execution).unwrap();
    bridge.evaluate_signal_recipe(&approx, execution).unwrap();

    value.store(8, Ordering::Relaxed);
    bridge
        .invalidate_signal_recipe(&source, ASPECT, Vec::new())
        .unwrap();
    bridge.evaluate_signal_recipe(&source, execution).unwrap();
    let exact = bridge.evaluate_signal_recipe(&exact, execution).unwrap();
    let approx = bridge.evaluate_signal_recipe(&approx, execution).unwrap();
    assert_eq!(exact.execution_report.unwrap().tasks_executed, 1);
    assert_eq!(approx.execution_report.unwrap().tasks_executed, 0);
}
