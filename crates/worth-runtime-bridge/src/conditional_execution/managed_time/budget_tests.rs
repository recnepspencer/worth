mod decision;
mod layout_oracle;

use super::super::{BridgeInstalledConditionalLowering, BridgeSealedRuntimeAssembly};
use super::*;
use crate::policy::BridgeConditionalRetentionBudget;
use std::sync::Arc;

pub(super) type Owner = (
    BridgeSealedRuntimeAssembly,
    Arc<BridgeInstalledConditionalLowering>,
);

pub(crate) fn assert_retention_boundaries(
    factory: impl Fn(BridgeConditionalRetentionBudget) -> Owner,
) {
    clock_and_binding(&factory);
    due_output(&factory);
    decision::decision_and_reentry(&factory);
}

fn clock_and_binding(factory: &impl Fn(BridgeConditionalRetentionBudget) -> Owner) {
    let charge = layout_oracle::clock(2) + layout_oracle::binding();
    for ceiling in [charge, charge - 1] {
        let (owner, lowering) = factory(budget(ceiling));
        let ledger = Arc::clone(&owner.test_runtime().retention);
        let result = install(&owner, &lowering, "clock:one", 2);
        assert_eq!(result.is_ok(), ceiling == charge);
        if let Ok(binding) = result {
            assert_eq!(ledger.usage(), (1, 2, charge));
            owner.close_managed_clock(binding).unwrap();
        }
        assert_eq!(owner.test_runtime().managed_clock_count(), 0);
        assert_eq!(ledger.usage(), (0, 0, 0));
    }
}

fn due_output(factory: &impl Fn(BridgeConditionalRetentionBudget) -> Owner) {
    let baseline = layout_oracle::baseline();
    let resident = layout_oracle::clock(1) + layout_oracle::binding() + baseline;
    let output = layout_oracle::due(1);
    for ceiling in [resident + output, resident + output - 1] {
        let (owner, lowering) = factory(budget(ceiling));
        let ledger = Arc::clone(&owner.test_runtime().retention);
        let binding = install(&owner, &lowering, "clock:one", 1).unwrap();
        owner
            .reconcile_managed_temporal_intent(active(&binding, "intent:one", 1, 5))
            .unwrap();
        let result = observe(&owner, &binding);
        if ceiling == resident + output {
            let BridgeManagedClockObservationOutcome::Accepted(accepted) = result.unwrap() else {
                panic!("accepted");
            };
            let mut wakes = accepted.into_due().into_wakes();
            let wake = wakes.pop().unwrap();
            owner.close_managed_clock(binding).unwrap();
            assert_eq!(
                ledger.usage(),
                (0, 0, baseline + output),
                "the escaped wake owns its baseline and output allocation"
            );
            drop(wake);
            assert_eq!(
                ledger.usage(),
                (0, 0, output),
                "empty buffer still owns allocated capacity"
            );
            let mut empty = wakes.into_iter();
            assert!(empty.next().is_none());
            assert_eq!(
                ledger.usage(),
                (0, 0, output),
                "empty iterator retains the same backing"
            );
            drop(empty);
            assert_eq!(ledger.usage(), (0, 0, 0));
        } else {
            assert!(
                matches!(result, Err(ref error) if error.kind() == BridgeManagedTemporalDenialKind::RetentionCapacityExhausted)
            );
            let lane = owner
                .test_runtime()
                .admit_managed_clock_lane(&binding)
                .unwrap();
            let lane = lock_lane(&lane).unwrap();
            assert_eq!(lane.last_observation(), None);
            let summary = lane.signal.temporal_wake_summary().unwrap();
            assert_eq!((summary.scheduled_count(), summary.ready_count()), (1, 0));
            assert!(
                !lane.quarantined,
                "resource denial before effects remains retryable"
            );
            drop(lane);
            owner.close_managed_clock(binding).unwrap();
        }
    }
}

fn budget(bytes: u64) -> BridgeConditionalRetentionBudget {
    BridgeConditionalRetentionBudget {
        maximum_retained_bytes: bytes,
        ..BridgeConditionalRetentionBudget::development()
    }
}

pub(super) fn install(
    owner: &BridgeSealedRuntimeAssembly,
    lowering: &Arc<BridgeInstalledConditionalLowering>,
    name: &str,
    capacity: usize,
) -> Result<BridgeManagedClockBinding, BridgeManagedTemporalDenial> {
    owner.install_managed_clock(BridgeManagedClockInstallationParts {
        lowering,
        binding_identity: Arc::from(name),
        source_identity: Arc::from("clock:source"),
        timeline_identity: Arc::from("clock:timeline"),
        maximum_active_intents: capacity,
        maximum_due_wakes_per_observation: 1,
    })
}

pub(super) fn active<'a>(
    binding: &'a BridgeManagedClockBinding,
    identity: &str,
    revision: u64,
    due: u64,
) -> BridgeManagedTemporalIntentReconciliationParts<'a> {
    BridgeManagedTemporalIntentReconciliationParts {
        binding,
        identity: BridgeManagedTemporalIntentIdentity::declare(Arc::from(identity)).unwrap(),
        revision,
        due_coordinate: due,
        idempotency_identity: Arc::from("idempotency:one"),
        source_record_identity: crate::facade::RelationalBridgeRecordIdentityParts::entity(0, 1, 1),
        lifecycle: BridgeManagedTemporalIntentLifecycle::Active,
    }
}

pub(super) fn observe(
    owner: &BridgeSealedRuntimeAssembly,
    binding: &BridgeManagedClockBinding,
) -> Result<BridgeManagedClockObservationOutcome, BridgeManagedTemporalDenial> {
    owner.observe_managed_clock(BridgeManagedClockObservationParts {
        binding,
        source_identity: "clock:source",
        timeline_identity: "clock:timeline",
        sequence: 1,
        observed_coordinate: 5,
    })
}
