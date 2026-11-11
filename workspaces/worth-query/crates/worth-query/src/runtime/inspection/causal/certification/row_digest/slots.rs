use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::{WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag};

use worth_runtime_bridge::facade::BridgeCausalEvidenceFamily;

use super::super::super::inventory::CausalEvidenceFamily;
use super::super::super::materialization::{
    QueryCausalEvidenceReferenceArtifact, QueryCausalInspectionArtifact,
};

#[derive(Default)]
pub(super) struct NamedEvidenceSlots {
    pub(super) relational_authority_digest: Option<String>,
    pub(super) bridge_route_digest: Option<String>,
    pub(super) bridge_evaluation_digest: Option<String>,
    pub(super) bridge_source_materialization_digest: Option<String>,
    pub(super) bridge_structural_digest: Option<String>,
    pub(super) bridge_stream_digest: Option<String>,
    pub(super) bridge_preview_digest: Option<String>,
    pub(super) bridge_writeback_digest: Option<String>,
    pub(super) bridge_replay_digest: Option<String>,
    pub(super) signal_invalidation_digest: Option<String>,
    pub(super) signal_evaluation_digest: Option<String>,
    pub(super) signal_forensic_availability_digest: Option<String>,
    pub(super) signal_replay_cursor_digest: Option<String>,
    pub(super) signal_lineage_digest: Option<String>,
    pub(super) signal_provenance_digest: Option<String>,
    pub(super) replay_posture_digest: Option<String>,
}

pub(super) fn named_evidence_slots(
    artifact: &QueryCausalInspectionArtifact,
) -> Result<NamedEvidenceSlots, IdentityDerivationDenial> {
    let references = evidence_references(artifact);
    Ok(NamedEvidenceSlots {
        relational_authority_digest: digest_for_bridge_families(
            references,
            &[BridgeCausalEvidenceFamily::RelationalAuthority],
        )?,
        bridge_route_digest: digest_for_bridge_families(
            references,
            &[BridgeCausalEvidenceFamily::BridgeRoute],
        )?,
        bridge_evaluation_digest: digest_for_bridge_families(
            references,
            &[
                BridgeCausalEvidenceFamily::BridgeHistoricalEvaluation,
                BridgeCausalEvidenceFamily::BridgeHistoricalEvaluationFailure,
            ],
        )?,
        bridge_source_materialization_digest: digest_for_bridge_families(
            references,
            &[
                BridgeCausalEvidenceFamily::BridgeSourceMaterialization,
                BridgeCausalEvidenceFamily::BridgeSourceFailure,
            ],
        )?,
        bridge_structural_digest: digest_for_bridge_families(
            references,
            &[
                BridgeCausalEvidenceFamily::BridgeStructuralRemap,
                BridgeCausalEvidenceFamily::BridgeStructuralBranchComparison,
            ],
        )?,
        bridge_stream_digest: digest_for_bridge_families(
            references,
            &[
                BridgeCausalEvidenceFamily::BridgeStreamReplay,
                BridgeCausalEvidenceFamily::BridgeStreamCheckpoint,
            ],
        )?,
        bridge_preview_digest: digest_for_bridge_families(
            references,
            &[
                BridgeCausalEvidenceFamily::BridgePreviewExecution,
                BridgeCausalEvidenceFamily::BridgePreviewDiscard,
                BridgeCausalEvidenceFamily::BridgePreviewPromotion,
            ],
        )?,
        bridge_writeback_digest: digest_for_bridge_families(references, &writeback_families())?,
        bridge_replay_digest: digest_for_any_family(
            references,
            &[
                BridgeCausalEvidenceFamily::BridgeStreamReplay.as_str(),
                BridgeCausalEvidenceFamily::BridgeWritebackReplay.as_str(),
                CausalEvidenceFamily::BridgeReplay.as_str(),
            ],
        )?,
        signal_invalidation_digest: digest_for_any_family(
            references,
            &[
                BridgeCausalEvidenceFamily::SignalInvalidation.as_str(),
                CausalEvidenceFamily::SignalInvalidation.as_str(),
            ],
        )?,
        signal_evaluation_digest: digest_for_query_families(
            references,
            &[CausalEvidenceFamily::SignalEvaluation],
        )?,
        signal_forensic_availability_digest: digest_for_query_families(
            references,
            &[CausalEvidenceFamily::SignalForensicAvailability],
        )?,
        signal_replay_cursor_digest: digest_for_query_families(
            references,
            &[CausalEvidenceFamily::SignalReplayCursor],
        )?,
        signal_lineage_digest: digest_for_query_families(
            references,
            &[
                CausalEvidenceFamily::SignalLineage,
                CausalEvidenceFamily::Lineage,
            ],
        )?,
        signal_provenance_digest: digest_for_query_families(
            references,
            &[
                CausalEvidenceFamily::SignalProvenance,
                CausalEvidenceFamily::Provenance,
            ],
        )?,
        replay_posture_digest: digest_for_any_family(
            references,
            &[
                BridgeCausalEvidenceFamily::BridgeStreamReplay.as_str(),
                BridgeCausalEvidenceFamily::BridgeWritebackReplay.as_str(),
                CausalEvidenceFamily::BridgeReplay.as_str(),
                CausalEvidenceFamily::SignalReplayCursor.as_str(),
            ],
        )?,
    })
}

fn writeback_families() -> [BridgeCausalEvidenceFamily; 6] {
    [
        BridgeCausalEvidenceFamily::BridgeWritebackAdmission,
        BridgeCausalEvidenceFamily::BridgeWritebackMapperEnvelope,
        BridgeCausalEvidenceFamily::BridgeWritebackMappedFamilyInput,
        BridgeCausalEvidenceFamily::BridgeWritebackMapper,
        BridgeCausalEvidenceFamily::BridgeWritebackExecution,
        BridgeCausalEvidenceFamily::BridgeWritebackReplay,
    ]
}

fn evidence_references(
    artifact: &QueryCausalInspectionArtifact,
) -> &[QueryCausalEvidenceReferenceArtifact] {
    match artifact {
        QueryCausalInspectionArtifact::Admitted(artifact) => artifact.evidence_references(),
        QueryCausalInspectionArtifact::Advisory(artifact) => artifact.evidence_references(),
        QueryCausalInspectionArtifact::Denied(_) => &[],
    }
}

fn digest_for_bridge_families(
    references: &[QueryCausalEvidenceReferenceArtifact],
    families: &[BridgeCausalEvidenceFamily],
) -> Result<Option<String>, IdentityDerivationDenial> {
    let family_names = families
        .iter()
        .map(BridgeCausalEvidenceFamily::as_str)
        .collect::<Vec<_>>();
    Ok(digest_for_any_family(references, &family_names)?)
}

fn digest_for_query_families(
    references: &[QueryCausalEvidenceReferenceArtifact],
    families: &[CausalEvidenceFamily],
) -> Result<Option<String>, IdentityDerivationDenial> {
    let family_names = families
        .iter()
        .map(CausalEvidenceFamily::as_str)
        .collect::<Vec<_>>();
    Ok(digest_for_any_family(references, &family_names)?)
}

fn digest_for_any_family(
    references: &[QueryCausalEvidenceReferenceArtifact],
    families: &[&str],
) -> Result<Option<String>, IdentityDerivationDenial> {
    let reference_identities = references
        .iter()
        .filter(|reference| families.contains(&reference.family()))
        .map(|reference| reference.reference_receipt_evidence_identity())
        .collect::<Vec<_>>();
    if reference_identities.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        WorthQueryEvidenceIdentity::compose(
            WorthQueryEvidenceScope::CausalInspectionCertificationFailureEvidence,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "causal_inspection_named_evidence_slot_digest_v1",
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("family"),
            families.iter().copied(),
        )
        .field_evidence_identity_sequence(
            WorthQueryEvidenceTag::new("reference"),
            reference_identities,
        )
        .seal()?
        .as_str()
        .to_string(),
    ))
}
