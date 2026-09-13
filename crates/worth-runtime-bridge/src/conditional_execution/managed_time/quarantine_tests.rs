use super::budget_tests::{active, install, observe, Owner};
use super::quarantine::Fault;
use super::*;
use crate::policy::BridgeConditionalRetentionBudget;

pub(crate) fn assert_lane_quarantine(factory: impl Fn(BridgeConditionalRetentionBudget) -> Owner) {
    for fault in [Fault::Divergence, Fault::Unwind] {
        for during_schedule in [true, false] {
            let (owner, lowering) = factory(BridgeConditionalRetentionBudget::development());
            let broken = install(&owner, &lowering, "clock:broken", 2).unwrap();
            let sibling = install(&owner, &lowering, "clock:sibling", 1).unwrap();
            if !during_schedule {
                owner
                    .reconcile_managed_temporal_intent(active(&broken, "intent:one", 1, 5))
                    .unwrap();
            }
            let cell = owner
                .test_runtime()
                .admit_managed_clock_lane(&broken)
                .unwrap();
            cell.lock().unwrap().fault = Some(fault);
            let failure = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                if during_schedule {
                    owner
                        .reconcile_managed_temporal_intent(active(&broken, "intent:one", 1, 5))
                        .map(|_| ())
                } else {
                    observe(&owner, &broken).map(|_| ())
                }
            }));
            match fault {
                Fault::Divergence => assert!(
                    matches!(failure, Ok(Err(ref e)) if e.kind() == BridgeManagedTemporalDenialKind::LaneQuarantined)
                ),
                Fault::Unwind => assert!(failure.is_err()),
            }
            {
                let lane = cell
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                assert!(lane.quarantined);
                let counts = lane.signal.temporal_wake_summary().unwrap();
                if during_schedule {
                    assert_eq!(
                        counts.scheduled_count(),
                        1,
                        "Signal movement actually happened"
                    );
                    assert!(
                        lane.intents.is_empty(),
                        "Bridge map publication did not happen"
                    );
                } else {
                    assert_eq!(
                        counts.ready_count(),
                        1,
                        "Signal promotion actually happened"
                    );
                    assert_eq!(
                        lane.last_observation(),
                        None,
                        "Bridge observation was not published"
                    );
                }
            }
            for _ in 0..2 {
                assert!(
                    matches!(observe(&owner, &broken), Err(ref e) if e.kind() == BridgeManagedTemporalDenialKind::LaneQuarantined)
                );
                assert!(
                    matches!(owner.reconcile_managed_temporal_intent(active(&broken, "intent:two", 2, 7)),
                    Err(ref e) if e.kind() == BridgeManagedTemporalDenialKind::LaneQuarantined)
                );
            }
            owner
                .reconcile_managed_temporal_intent(active(&sibling, "intent:sibling", 1, 5))
                .unwrap();
            let BridgeManagedClockObservationOutcome::Accepted(accepted) =
                observe(&owner, &sibling).unwrap()
            else {
                panic!("sibling progresses");
            };
            assert_eq!(accepted.due().wakes().len(), 1);
            drop(accepted);
            owner.close_managed_clock(broken).unwrap();
            owner.close_managed_clock(sibling).unwrap();
            drop(cell);
            assert_eq!(owner.test_runtime().retention.usage(), (0, 0, 0));
        }
    }
}
