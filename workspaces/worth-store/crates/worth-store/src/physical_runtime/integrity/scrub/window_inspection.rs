use super::progress::{
    PhysicalIntegrityScrubCounters as RuntimeCounters, PhysicalIntegrityScrubWindowObservation,
};
use crate::physical_runtime::PhysicalWorkSettlementEvidence;
use worth_store_physical_integrity::*;

pub(super) fn inspect(
    ordinal: u64,
    scope: PhysicalArtifactScope,
    evidence: PhysicalWorkSettlementEvidence,
    late: bool,
    counters: &mut RuntimeCounters,
    validator: &mut PhysicalIntegrityScrubValidator,
    checkpoint_source: &mut Option<worth_store_physical_backend::InspectionSourceVersion>,
) -> PhysicalIntegrityScrubWindowObservation {
    counters.acquired_bytes += evidence.completed_payload_bytes();
    let mut selector_identity = None;
    let (outcome, validation_counters) = match evidence {
        PhysicalWorkSettlementEvidence::Inspection {
            physical,
            bytes,
            scheduler,
        } => {
            let same_source = if matches!(
                physical.range().target(),
                worth_store_physical_format::PhysicalArtifactReadTarget::Checkpoint(_)
            ) {
                match physical.source_version() {
                    Some(version) => {
                        let unchanged = checkpoint_source
                            .as_ref()
                            .is_none_or(|prior| prior == version);
                        *checkpoint_source = Some(version.clone());
                        unchanged
                    }
                    None => false,
                }
            } else {
                true
            };
            if late
                || !physical.stable()
                || !same_source
                || physical.completed_bytes() != scope.byte_range().length()
                || !matches!(
                    scheduler,
                    worth_store_io_scheduler::QueueExecutionOutcome::Executed(_)
                )
            {
                validator.invalidate(scope);
                indeterminate(
                    scope,
                    physical.completed_bytes(),
                    late,
                    physical.stable() && same_source,
                )
            } else {
                let window = PhysicalIntegrityScrubWindow::new(
                    ordinal,
                    scope,
                    UntrustedPhysicalArtifact::from_bounded_bytes(&bytes),
                );
                let (inspection, validation) = validator.inspect(window);
                selector_identity = inspection.selector_identity();
                (inspection.outcome(), validation)
            }
        }
        PhysicalWorkSettlementEvidence::InspectionDenied(failure)
            if failure.kind() == worth_store_physical_backend::ArtifactTreeFailureKind::Absent =>
        {
            validator.invalidate(scope);
            (
                PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(
                    UnknownPhysicalIntegrityPosture::new(
                        scope,
                        UnknownPhysicalIntegrityCause::ExpectedArtifactAbsent,
                    ),
                )),
                PhysicalIntegrityObservationCounters::empty(scope.artifact_family()),
            )
        }
        _ => {
            validator.invalidate(scope);
            indeterminate(scope, 0, late, false)
        }
    };
    counters.completed_windows += 1;
    let quarantine = match outcome {
        PhysicalIntegrityObservationOutcome::Intact(_) => {
            counters.validated_windows += 1;
            None
        }
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Damaged(
            localization,
        )) => {
            counters.damaged_windows += 1;
            Some(PhysicalQuarantineObservation::new(
                localization,
                PhysicalQuarantinePosture::DamageObserved,
                std::num::NonZeroU64::new(ordinal + 1).expect("bounded ordinal"),
            ))
        }
        PhysicalIntegrityObservationOutcome::Rejected(
            PhysicalIntegrityRejection::Indeterminate(_),
        ) => {
            counters.indeterminate_windows += 1;
            None
        }
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unknown(_)) => {
            counters.unknown_windows += 1;
            None
        }
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Unsupported(
            _,
        )) => {
            counters.unsupported_windows += 1;
            None
        }
    };
    PhysicalIntegrityScrubWindowObservation {
        selector_identity,
        ordinal,
        outcome,
        validation_counters,
        quarantine,
        counters: *counters,
    }
}

fn indeterminate(
    scope: PhysicalArtifactScope,
    bytes: u64,
    late: bool,
    stable: bool,
) -> (
    PhysicalIntegrityObservationOutcome,
    PhysicalIntegrityObservationCounters,
) {
    let cause = if late {
        IndeterminatePhysicalIntegrityCause::ObservationBoundExhausted
    } else if !stable {
        IndeterminatePhysicalIntegrityCause::SourceChangedDuringInspection
    } else {
        IndeterminatePhysicalIntegrityCause::StableRangeNotProven
    };
    let range = PhysicalByteRange::new(scope.byte_range().offset(), bytes).ok();
    (
        PhysicalIntegrityObservationOutcome::Rejected(PhysicalIntegrityRejection::Indeterminate(
            IndeterminatePhysicalIntegrityPosture::new(scope, cause, range),
        )),
        PhysicalIntegrityObservationCounters::empty(scope.artifact_family()),
    )
}
