//! Transport and ownership controls, not real Store panic/recovery evidence.
use super::{classify_delivery, PhysicalStoreCloseFailureCause as Cause};

#[test]
fn joined_delivery_requires_the_actual_transported_value() {
    assert_eq!(classify_delivery(Ok(()), Some(73)), Ok(73));
    assert_eq!(
        classify_delivery::<u32>(Ok(()), None),
        Err((Cause::OutcomeUnavailable, None))
    );
}

#[test]
fn failed_join_preserves_evidence_received_before_worker_exit() {
    assert_eq!(
        classify_delivery(Err(Box::new("transport panic")), Some(73)),
        Err((Cause::WorkerPanicked, Some(73)))
    );
    assert_eq!(
        classify_delivery::<u32>(Err(Box::new("transport panic")), None),
        Err((Cause::WorkerPanicked, None))
    );
}

#[test]
fn consuming_failure_preserves_cause_and_phase_observation() {
    use crate::physical_runtime::instance::termination::{
        PhysicalStoreClosePhase, PhysicalStoreCloseProgressOwner,
    };
    let progress = PhysicalStoreCloseProgressOwner::new();
    let identity = crate::physical_runtime::RuntimeIdentity::from_reopened(
        std::num::NonZeroU64::new(91).unwrap(),
    );
    let failure = super::PhysicalStoreCloseFailure::new(
        super::PhysicalCloseTaskCustody::accepted(identity),
        Cause::WorkerPanicked,
        None,
        Some(progress.observation()),
    );
    assert_eq!(failure.runtime_identity(), identity);
    assert_eq!(failure.cause(), &Cause::WorkerPanicked);
    assert!(failure.actual_outcome().is_none());
    assert_eq!(failure.observation().unwrap().latest_phase(), None);
    // Drive only the descriptive owner; no physical shutdown is claimed.
    progress.record(PhysicalStoreClosePhase::SignalDisposed);
    let (cause, actual, observation) = failure.into_parts();
    assert_eq!(cause, Cause::WorkerPanicked);
    assert!(actual.is_none());
    assert_eq!(
        observation.unwrap().latest_phase(),
        Some(PhysicalStoreClosePhase::SignalDisposed)
    );
}
