use super::CheckpointInspectionAggregate;
use crate::*;
use worth_store_physical_format::integrity_declarations::PhysicalIntegrityArtifactFamily as Family;

pub(crate) fn inspect_checkpoint_window(
    aggregate: &mut Option<CheckpointInspectionAggregate>,
    window: PhysicalIntegrityScrubWindow<'_>,
) -> (
    PhysicalIntegrityScrubInspection,
    PhysicalIntegrityObservationCounters,
) {
    let input = window.artifact();
    let scope = window.scope();
    macro_rules! include {
        ($validate:ident, $result:ident, $include:ident) => {{
            let Some(prior) = aggregate.take().filter(|prior| prior.accepts_scope(scope)) else {
                return missing(scope, input.byte_count());
            };
            let (result, counters) = $validate(input, scope);
            match result {
                $result::Intact(record) => match prior.$include(&record) {
                    Ok(next) => {
                        *aggregate = Some(next);
                        intact(scope, counters)
                    }
                    Err(rejection) => rejected(scope, input.byte_count(), rejection),
                },
                $result::Rejected(rejection) => {
                    *aggregate = None;
                    rejected(scope, input.byte_count(), rejection)
                }
            }
        }};
    }
    match scope.artifact_family() {
        Family::CheckpointStreamHeader => {
            *aggregate = None;
            let staged_scope = PhysicalArtifactScope::checkpoint_stream_header(
                CheckpointStreamHeaderScopeIdentity::staged(scope.store_identity()),
                scope.byte_range(),
            );
            let (result, counters) = validate_checkpoint_stream_header(input, staged_scope);
            match result {
                CheckpointStreamHeaderIntegrityValidation::Intact(header) => {
                    if scope
                        .checkpoint_identity()
                        .is_some_and(|identity| identity != header.checkpoint_identity())
                    {
                        return missing(scope, input.byte_count());
                    }
                    *aggregate = Some(CheckpointInspectionAggregate::new(&header));
                    intact(scope, counters)
                }
                CheckpointStreamHeaderIntegrityValidation::Rejected(rejection) => {
                    rejected(scope, input.byte_count(), rejection)
                }
            }
        }
        Family::CheckpointDirtyBasis => include!(
            validate_checkpoint_dirty_basis,
            CheckpointDirtyBasisIntegrityValidation,
            dirty
        ),
        Family::CheckpointBindingCompaction => include!(
            validate_checkpoint_binding_compaction,
            CheckpointBindingCompactionIntegrityValidation,
            compaction
        ),
        Family::CheckpointBinding => include!(
            validate_checkpoint_binding,
            CheckpointBindingIntegrityValidation,
            binding
        ),
        Family::CheckpointFooter => {
            let Some(prior) = aggregate.take().filter(|prior| prior.accepts_scope(scope)) else {
                return missing(scope, input.byte_count());
            };
            let (result, counters) =
                super::footer::validate_with_expected(input, scope, || prior.expected(scope));
            match result {
                CheckpointFooterIntegrityValidation::Intact(_) => intact(scope, counters),
                CheckpointFooterIntegrityValidation::Rejected(rejection) => {
                    rejected(scope, input.byte_count(), rejection)
                }
            }
        }
        _ => unreachable!("checkpoint family dispatch"),
    }
}

fn intact(
    scope: PhysicalArtifactScope,
    counters: PhysicalIntegrityObservationCounters,
) -> (
    PhysicalIntegrityScrubInspection,
    PhysicalIntegrityObservationCounters,
) {
    (
        PhysicalIntegrityScrubInspection::new(PhysicalIntegrityObservationOutcome::Intact(scope)),
        counters,
    )
}
fn rejected(
    scope: PhysicalArtifactScope,
    bytes: u64,
    rejection: PhysicalIntegrityRejection,
) -> (
    PhysicalIntegrityScrubInspection,
    PhysicalIntegrityObservationCounters,
) {
    (
        PhysicalIntegrityScrubInspection::new(PhysicalIntegrityObservationOutcome::Rejected(
            rejection,
        )),
        PhysicalIntegrityObservationCounters::one_rejected(
            scope.artifact_family(),
            bytes,
            rejection,
        ),
    )
}
fn missing(
    scope: PhysicalArtifactScope,
    bytes: u64,
) -> (
    PhysicalIntegrityScrubInspection,
    PhysicalIntegrityObservationCounters,
) {
    rejected(
        scope,
        bytes,
        PhysicalIntegrityRejection::Unknown(UnknownPhysicalIntegrityPosture::new(
            scope,
            UnknownPhysicalIntegrityCause::ExpectedScopeUnavailable,
        )),
    )
}
