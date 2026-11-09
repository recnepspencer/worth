use super::super::{WorthQueryAuthorityLane, WorthQueryRuntimeInspectionEvidence};
use super::mutation::WorthQueryWriteReceipt;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

pub struct WorthQueryArtifactInspector<'a> {
    pub(in crate::runtime) receipt: &'a WorthQueryWriteReceipt,
    pub(in crate::runtime) runtime_evidence: WorthQueryRuntimeInspectionEvidence,
}

impl<'a> WorthQueryArtifactInspector<'a> {
    pub fn canonical(&self) -> WorthQueryInspectedArtifact {
        WorthQueryInspectedArtifact::new(
            "canonical",
            self.receipt.commit_evidence_identity().clone(),
            self.receipt.snapshot_evidence_identity().clone(),
        )
    }

    pub fn workflow(&self) -> WorthQueryInspectedArtifact {
        WorthQueryInspectedArtifact::new(
            "workflow",
            self.receipt.commit_evidence_identity().clone(),
            self.receipt.snapshot_evidence_identity().clone(),
        )
    }

    pub fn bridge_authority(&self) -> WorthQueryInspectedArtifact {
        WorthQueryInspectedArtifact::new(
            "bridge-authority",
            self.receipt.commit_evidence_identity().clone(),
            self.receipt.snapshot_evidence_identity().clone(),
        )
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.receipt.authority_lane()
    }

    pub fn runtime_evidence(&self) -> &WorthQueryRuntimeInspectionEvidence {
        &self.runtime_evidence
    }

    pub fn live_patch_artifacts(
        &self,
    ) -> Result<Vec<WorthQueryEvidenceIdentity>, IdentityDerivationDenial> {
        Ok(self
            .receipt
            .deltas()
            .iter()
            .map(|delta| {
                worth_query_evidence_identity(
                    WorthQueryEvidenceScope::WriteReceiptInspectionArtifact,
                )
                .field_shape(WorthQueryEvidenceTag::new("role"), "live-patch-artifact")
                .field_value(WorthQueryEvidenceTag::new("collection"), delta.collection())
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("entity_identity"),
                    &delta.entity_identity.evidence_identity(),
                )
                .seal()
            })
            .collect::<Result<Vec<_>, _>>()?)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInspectedArtifact {
    pub(super) family: String,
    pub(super) identity: WorthQueryEvidenceIdentity,
    pub(super) basis: WorthQueryEvidenceIdentity,
}

impl WorthQueryInspectedArtifact {
    pub(in crate::runtime) fn new(
        family: impl Into<String>,
        identity: WorthQueryEvidenceIdentity,
        basis: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self {
            family: family.into(),
            identity,
            basis,
        }
    }

    pub fn family(&self) -> &str {
        &self.family
    }

    pub fn identity(&self) -> &str {
        self.identity.as_str()
    }

    pub fn basis(&self) -> &str {
        self.basis.as_str()
    }
}
