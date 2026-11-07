use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublicBridgeProjectionConsumptionEvidence {
    consumed_title: String,
    receipt_digest: String,
    fact_set_digest: String,
    source_identity: String,
    extracted_fact_count: usize,
    requested_field: String,
    digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryPublicBridgeProjectionConsumptionEvidence {
    pub fn new(
        consumed_title: impl Into<String>,
        receipt_digest: impl Into<String>,
        fact_set_digest: impl Into<String>,
        source_identity: impl Into<String>,
        extracted_fact_count: usize,
        requested_field: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let consumed_title = consumed_title.into();
        let receipt_digest = receipt_digest.into();
        let fact_set_digest = fact_set_digest.into();
        let source_identity = source_identity.into();
        let requested_field = requested_field.into();
        let digest = worth_query_evidence_identity(
            WorthQueryEvidenceScope::RuntimeHostileCertificationArtifact,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("consumed_title"),
            &consumed_title,
        )
        .field_value(
            WorthQueryEvidenceTag::new("receipt_digest"),
            &receipt_digest,
        )
        .field_value(
            WorthQueryEvidenceTag::new("fact_set_digest"),
            &fact_set_digest,
        )
        .field_value(
            WorthQueryEvidenceTag::new("source_identity"),
            &source_identity,
        )
        .field_usize(
            WorthQueryEvidenceTag::new("extracted_fact_count"),
            extracted_fact_count,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("requested_field"),
            &requested_field,
        )
        .seal()?;
        Ok(Self {
            consumed_title,
            receipt_digest,
            fact_set_digest,
            source_identity,
            extracted_fact_count,
            requested_field,
            digest,
        })
    }

    pub fn consumed_title(&self) -> &str {
        &self.consumed_title
    }

    pub fn receipt_digest(&self) -> &str {
        &self.receipt_digest
    }

    pub fn fact_set_digest(&self) -> &str {
        &self.fact_set_digest
    }

    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    pub fn extracted_fact_count(&self) -> usize {
        self.extracted_fact_count
    }

    pub fn requested_field(&self) -> &str {
        &self.requested_field
    }

    pub fn digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.digest
    }
}
