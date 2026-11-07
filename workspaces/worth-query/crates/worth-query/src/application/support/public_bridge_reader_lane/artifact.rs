use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

use super::{
    WorthQueryPublicBridgeProjectionConsumptionEvidence, WorthQueryPublicBridgeReaderLaneInventory,
    WorthQueryPublicBridgeReaderLanePosture, WorthQueryPublicBridgeReaderLaneSabotage,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublicBridgeReaderLaneCertification {
    posture: WorthQueryPublicBridgeReaderLanePosture,
    projection_reads: Vec<WorthQueryPublicBridgeProjectionConsumptionEvidence>,
    projection_receipt_digests: Vec<String>,
    published_artifact_digests: Vec<String>,
    inventory: WorthQueryPublicBridgeReaderLaneInventory,
    direct_materialization_read_count: usize,
    sabotage: WorthQueryPublicBridgeReaderLaneSabotage,
    digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryPublicBridgeReaderLaneCertification {
    pub fn certify(
        projection_reads: Vec<WorthQueryPublicBridgeProjectionConsumptionEvidence>,
        published_artifact_digests: Vec<String>,
        inventory: WorthQueryPublicBridgeReaderLaneInventory,
        sabotage: WorthQueryPublicBridgeReaderLaneSabotage,
    ) -> Result<Self, IdentityDerivationDenial> {
        let direct_materialization_read_count = inventory.direct_materialization_read_count();
        let has_projection_receipts = !projection_reads.is_empty()
            && projection_reads.iter().all(|read| {
                !read.receipt_digest().is_empty() && !read.fact_set_digest().is_empty()
            });
        let has_published_artifacts = !published_artifact_digests.is_empty()
            && published_artifact_digests
                .iter()
                .all(|digest| !digest.is_empty());
        let projection_receipt_digests = projection_reads
            .iter()
            .map(|read| read.receipt_digest().to_string())
            .collect::<Vec<_>>();
        let posture = if has_projection_receipts
            && has_published_artifacts
            && direct_materialization_read_count == 0
            && sabotage.rejected()
        {
            WorthQueryPublicBridgeReaderLanePosture::Closed
        } else {
            WorthQueryPublicBridgeReaderLanePosture::Open
        };
        let digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_shape(WorthQueryEvidenceTag::new("posture"), posture.as_str())
        .field_value_sequence(
            WorthQueryEvidenceTag::new("projection_receipt_digest"),
            projection_reads
                .iter()
                .map(WorthQueryPublicBridgeProjectionConsumptionEvidence::receipt_digest),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("projection_read_digest"),
            projection_reads
                .iter()
                .map(|read| read.digest().terminal_projection_for_reporting()),
        )
        .field_value_sequence(
            WorthQueryEvidenceTag::new("published_artifact_digest"),
            published_artifact_digests.iter().map(String::as_str),
        )
        .field_value(
            WorthQueryEvidenceTag::new("inventory_digest"),
            inventory.digest().terminal_projection_for_reporting(),
        )
        .field_usize(
            WorthQueryEvidenceTag::new("direct_materialization_read_count"),
            direct_materialization_read_count,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("sabotage_kind"),
            sabotage.kind().as_str(),
        )
        .field_value(
            WorthQueryEvidenceTag::new("sabotage_pattern"),
            sabotage.localized_pattern(),
        )
        .field_shape(
            WorthQueryEvidenceTag::new("sabotage_rejected"),
            if sabotage.rejected() { "true" } else { "false" },
        )
        .seal()?;
        Ok(Self {
            posture,
            projection_reads,
            projection_receipt_digests,
            published_artifact_digests,
            inventory,
            direct_materialization_read_count,
            sabotage,
            digest,
        })
    }

    pub fn posture(&self) -> WorthQueryPublicBridgeReaderLanePosture {
        self.posture
    }

    pub fn projection_receipt_digests(&self) -> &[String] {
        &self.projection_receipt_digests
    }

    pub fn projection_reads(&self) -> &[WorthQueryPublicBridgeProjectionConsumptionEvidence] {
        &self.projection_reads
    }

    pub fn published_artifact_digests(&self) -> &[String] {
        &self.published_artifact_digests
    }

    pub fn direct_materialization_read_count(&self) -> usize {
        self.direct_materialization_read_count
    }

    pub fn inventory(&self) -> &WorthQueryPublicBridgeReaderLaneInventory {
        &self.inventory
    }

    pub fn sabotage(&self) -> &WorthQueryPublicBridgeReaderLaneSabotage {
        &self.sabotage
    }

    pub fn digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.digest
    }
}
