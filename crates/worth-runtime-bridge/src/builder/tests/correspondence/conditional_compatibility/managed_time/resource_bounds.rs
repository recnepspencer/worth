use crate::facade::{
    BridgeConditionalProviderSet, BridgeConditionalRetentionBudget,
    BridgeManagedClockInstallationParts, BridgeManagedTemporalDenialKind,
};
use std::sync::Arc;

#[test]
fn exact_and_one_short_retention_at_real_sealed_owner_boundaries() {
    crate::conditional_execution::assert_retention_boundaries(owner);
}

#[test]
fn incomplete_signal_to_bridge_publication_permanently_fences_only_its_clock() {
    crate::conditional_execution::assert_lane_quarantine(owner);
}

fn owner(
    budget: BridgeConditionalRetentionBudget,
) -> (
    crate::facade::BridgeSealedRuntimeAssembly,
    Arc<crate::facade::BridgeInstalledConditionalLowering>,
) {
    let (mut builder, request) = super::super::installation_fixture_with_runtime(
        super::super::always_eligible_contract("query:one"),
        &["managed-budget"],
        BridgeConditionalProviderSet::new().compute(CountingCompute),
        &[],
        |registrations| {
            let mut bridge = super::super::runtime(super::super::exact_mapping(), registrations);
            bridge.policy = bridge.policy.with_conditional_retention(budget);
            bridge
        },
    );
    let lowering = builder.install(request).unwrap();
    (builder.seal().unwrap(), lowering)
}

struct CountingCompute;
impl crate::facade::BridgeConditionalProviderSemantics for CountingCompute {
    type SemanticContract = u64;
    fn semantic_contract(&self) -> u64 {
        1
    }
    fn retained_heap_bytes(
        &self,
        _semantic_contract: &Self::SemanticContract,
    ) -> Result<
        crate::facade::BridgeConditionalProviderHeapRetention,
        crate::facade::BridgeConditionalProviderRetentionOverflow,
    > {
        Ok(crate::facade::BridgeConditionalProviderHeapRetention::none())
    }
}
impl crate::facade::BridgeConditionalComputeProvider for CountingCompute {
    fn compute(
        &self,
        context: &mut dyn std::any::Any,
    ) -> Result<worth_signal::facade::NodeEvaluationResult, String> {
        if let Some(contacts) = context.downcast_mut::<usize>() {
            *contacts += 1;
        }
        crate::facade::BridgeConditionalComputeProvider::compute(&super::super::Compute(1), context)
    }
}

#[test]
fn clock_budget_denial_preserves_installation_and_close_returns_its_capacity() {
    let (owner, lowering) = owner(BridgeConditionalRetentionBudget {
        maximum_managed_clocks: Some(1),
        maximum_reserved_temporal_intents: Some(2),
        ..BridgeConditionalRetentionBudget::development()
    });
    let install = |name: &str, wakes| {
        owner.install_managed_clock(BridgeManagedClockInstallationParts {
            lowering: &lowering,
            binding_identity: Arc::from(name),
            source_identity: Arc::from("clock:source"),
            timeline_identity: Arc::from("clock:timeline"),
            maximum_active_intents: wakes,
            maximum_due_wakes_per_observation: 1,
        })
    };
    let overflow = install("clock:one", usize::MAX);
    assert!(
        matches!(overflow, Err(ref denial) if denial.kind() == BridgeManagedTemporalDenialKind::RetentionCapacityExhausted)
    );
    assert_eq!(
        owner
            .conditional_lifecycle_probe()
            .live_managed_clock_count(),
        0
    );
    let first = install("clock:one", 2).unwrap();
    assert!(
        matches!(install("clock:two", 1), Err(ref denial) if denial.kind() == BridgeManagedTemporalDenialKind::RetentionCapacityExhausted)
    );
    assert_eq!(
        owner
            .conditional_lifecycle_probe()
            .live_managed_clock_count(),
        1
    );
    owner
        .reconcile_managed_temporal_intent(super::active(&first, "intent:one", 1, 5))
        .unwrap();
    let closure = owner.close_managed_clock(first).unwrap();
    assert_eq!(closure.scheduled_wakes(), 1);
    assert_eq!(
        owner
            .conditional_lifecycle_probe()
            .live_managed_clock_count(),
        0
    );
    let replacement = install("clock:two", 2).unwrap();
    owner.close_managed_clock(replacement).unwrap();
}

#[test]
fn insufficient_bridge_bytes_deny_before_clock_membership() {
    let (owner, lowering) = owner(BridgeConditionalRetentionBudget {
        maximum_retained_bytes: Some(1),
        ..BridgeConditionalRetentionBudget::development()
    });
    let result = owner.install_managed_clock(BridgeManagedClockInstallationParts {
        lowering: &lowering,
        binding_identity: Arc::from("clock:denied"),
        source_identity: Arc::from("clock:source"),
        timeline_identity: Arc::from("clock:timeline"),
        maximum_active_intents: 1,
        maximum_due_wakes_per_observation: 1,
    });
    assert!(
        matches!(result, Err(ref denial) if denial.kind() == BridgeManagedTemporalDenialKind::RetentionCapacityExhausted)
    );
    assert_eq!(
        owner
            .conditional_lifecycle_probe()
            .live_managed_clock_count(),
        0
    );
}

#[test]
fn sampled_clock_rejects_raw_observations_and_reboot_before_temporal_movement() {
    use crate::facade::{BridgeManagedClockObservationOutcome, BridgeManagedClockObservationParts};
    use std::time::Duration;
    use worth_execution::time::{ControlledTime, TimeSampler};

    let (owner, lowering) = owner(BridgeConditionalRetentionBudget::development());
    let view = owner.conditional_execution_view();
    let time = Arc::new(ControlledTime::at_unix_secs(10));
    let sampler = Arc::new(TimeSampler::new(time.clone()));
    let binding = view
        .install_sampled_managed_clock(
            BridgeManagedClockInstallationParts {
                lowering: &lowering,
                binding_identity: Arc::from("clock:sealed"),
                source_identity: Arc::from("clock:source"),
                timeline_identity: Arc::from("clock:timeline"),
                maximum_active_intents: 1,
                maximum_due_wakes_per_observation: 1,
            },
            sampler,
        )
        .unwrap();
    owner
        .reconcile_managed_temporal_intent(super::active(&binding, "intent:one", 1, 5))
        .unwrap();

    let raw = view.observe_managed_clock(BridgeManagedClockObservationParts {
        binding: &binding,
        source_identity: "clock:source",
        timeline_identity: "clock:timeline",
        sequence: 1,
        observed_coordinate: 5,
    });
    assert!(
        matches!(raw, Err(error) if error.kind() == BridgeManagedTemporalDenialKind::ForeignClockSource)
    );

    time.advance(Duration::from_nanos(4));
    time.step_wall_back(Duration::from_secs(20));
    let before_due = view.observe_sampled_managed_clock(&binding, 1).unwrap();
    let BridgeManagedClockObservationOutcome::Accepted(before_due) = before_due else {
        panic!("first observation should advance");
    };
    assert_eq!(before_due.observed_coordinate(), 4);
    assert!(before_due.due().wakes().is_empty());

    time.reboot_epoch();
    time.advance(Duration::from_nanos(5));
    let after_reboot = view.observe_sampled_managed_clock(&binding, 2);
    assert!(
        matches!(after_reboot, Err(error) if error.kind() == BridgeManagedTemporalDenialKind::ForeignClockTimeline)
    );
    let closure = owner.close_managed_clock(binding).unwrap();
    assert_eq!(closure.scheduled_wakes(), 1);
    assert_eq!(closure.ready_wakes(), 0);
}
