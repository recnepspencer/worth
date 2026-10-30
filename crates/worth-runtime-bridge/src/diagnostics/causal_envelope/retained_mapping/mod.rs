pub(crate) mod digest_basis;
pub(crate) mod retained_artifact_digest;

use super::{
    BridgeCausalEnvelopeCounters, BridgeCausalEnvelopeDenial, BridgeCausalEnvelopeDenialKind,
    BridgeCausalEvidenceFamily, BridgeCausalEvidenceReference,
};
use crate::diagnostics::BridgeDiagnosticsFacade;
use crate::identity::BridgeIdentityEvidence;
use retained_artifact_digest::{
    bulk_planning_record_digest, continuity_record_digest,
    historical_evaluation_failure_record_digest, historical_evaluation_record_digest,
    merge_record_digest, preview_discard_record_digest, preview_execution_record_digest,
    preview_promotion_record_digest, route_record_digest, source_failure_record_digest,
    source_materialization_record_digest, stream_checkpoint_record_digest,
    stream_replay_record_digest, structural_branch_comparison_record_digest,
    structural_remap_record_digest, writeback_admission_record_digest,
    writeback_execution_record_digest, writeback_mapped_family_input_digest,
    writeback_mapper_envelope_digest, writeback_mapper_record_digest,
    writeback_replay_record_digest,
};

pub(crate) fn retained_record_evidence_identity(
    facade: &BridgeDiagnosticsFacade,
    reference: &BridgeCausalEvidenceReference,
) -> Result<Option<BridgeIdentityEvidence>, BridgeCausalEnvelopeDenial> {
    let reference_identity = reference.reference_evidence_identity();
    match reference.family() {
        BridgeCausalEvidenceFamily::BridgeConditionalObservation => {
            Err(BridgeCausalEnvelopeDenial::new(
                BridgeCausalEnvelopeDenialKind::NativeObservationRequired,
                reference.family(),
                reference.owner(),
                reference.family().expected_owner(),
                reference.reference_evidence_identity().clone(),
                BridgeCausalEnvelopeCounters::empty(),
            ))
        }
        BridgeCausalEvidenceFamily::BridgeBulkPlanning => {
            Ok(bulk_planning_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeRoute => {
            Ok(route_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeHistoricalEvaluation => Ok(
            historical_evaluation_record_digest(facade, reference_identity),
        ),
        BridgeCausalEvidenceFamily::BridgeHistoricalEvaluationFailure => Ok(
            historical_evaluation_failure_record_digest(facade, reference_identity),
        ),
        BridgeCausalEvidenceFamily::BridgePreviewExecution => {
            Ok(preview_execution_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgePreviewDiscard => {
            Ok(preview_discard_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgePreviewPromotion => {
            Ok(preview_promotion_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeSourceMaterialization => Ok(
            source_materialization_record_digest(facade, reference_identity),
        ),
        BridgeCausalEvidenceFamily::BridgeSourceFailure => {
            Ok(source_failure_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeStructuralRemap => {
            Ok(structural_remap_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeStructuralBranchComparison => Ok(
            structural_branch_comparison_record_digest(facade, reference_identity),
        ),
        BridgeCausalEvidenceFamily::BridgeStreamReplay => {
            Ok(stream_replay_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeStreamCheckpoint => {
            Ok(stream_checkpoint_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeContinuity => {
            Ok(continuity_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeMerge => {
            Ok(merge_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeWritebackAdmission => Ok(
            writeback_admission_record_digest(facade, reference_identity),
        ),
        BridgeCausalEvidenceFamily::BridgeWritebackMapperEnvelope => {
            Ok(writeback_mapper_envelope_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeWritebackMappedFamilyInput => Ok(
            writeback_mapped_family_input_digest(facade, reference_identity),
        ),
        BridgeCausalEvidenceFamily::BridgeWritebackMapper => {
            Ok(writeback_mapper_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::BridgeWritebackExecution => Ok(
            writeback_execution_record_digest(facade, reference_identity),
        ),
        BridgeCausalEvidenceFamily::BridgeWritebackReplay => {
            Ok(writeback_replay_record_digest(facade, reference_identity))
        }
        BridgeCausalEvidenceFamily::QueryObservation
        | BridgeCausalEvidenceFamily::RelationalAuthority
        | BridgeCausalEvidenceFamily::SignalInvalidation
        | BridgeCausalEvidenceFamily::SignalEvaluation
        | BridgeCausalEvidenceFamily::SignalForensicAvailability
        | BridgeCausalEvidenceFamily::SignalReplayCursor
        | BridgeCausalEvidenceFamily::SignalLineage
        | BridgeCausalEvidenceFamily::SignalProvenance => Err(BridgeCausalEnvelopeDenial::new(
            BridgeCausalEnvelopeDenialKind::EvidenceOwnerMismatch,
            reference.family(),
            reference.owner(),
            reference.family().expected_owner(),
            reference.reference_evidence_identity().clone(),
            BridgeCausalEnvelopeCounters::new(1, 1, 1, 0, 0, 0, 1),
        )),
    }
}
