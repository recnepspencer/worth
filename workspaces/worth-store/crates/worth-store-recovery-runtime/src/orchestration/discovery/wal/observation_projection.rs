use crate::entry::{
    PhysicalRecoveryWalIntegrityObservation, PhysicalRecoveryWalIntegrityObservationOutcome,
};
use crate::integrity_ingress::{
    RecoveryIntegrityIngressObservation, RecoveryIntegrityIngressObservationOutcome,
    RecoveryIntegrityIngressRejection,
};

pub(in crate::orchestration::discovery) fn public_observation(
    observation: RecoveryIntegrityIngressObservation,
) -> PhysicalRecoveryWalIntegrityObservation {
    let outcome = match observation.outcome() {
        RecoveryIntegrityIngressObservationOutcome::Admitted => {
            PhysicalRecoveryWalIntegrityObservationOutcome::Admitted
        }
        RecoveryIntegrityIngressObservationOutcome::Rejected(
            RecoveryIntegrityIngressRejection::Integrity(rejection),
        ) => PhysicalRecoveryWalIntegrityObservationOutcome::Rejected(rejection),
        RecoveryIntegrityIngressObservationOutcome::Rejected(_) => {
            unreachable!("source-binding failures do not return an admitted WAL inventory")
        }
    };
    PhysicalRecoveryWalIntegrityObservation::new(observation.scope(), outcome)
}
