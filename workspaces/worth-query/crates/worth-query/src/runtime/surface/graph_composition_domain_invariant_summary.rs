use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionDomainInvariantSummary {
    declared_collections: Vec<String>,
    declared_symbols: Vec<String>,
    target_combination_families: Vec<String>,
    lifecycle_families: Vec<String>,
    program_digest: WorthQueryEvidenceIdentity,
    breadth_digest: WorthQueryEvidenceIdentity,
    counter_snapshot: String,
    summary_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphCompositionDomainInvariantSummary {
    pub(crate) fn from_parts(
        declared_collections: Vec<String>,
        declared_symbols: Vec<String>,
        target_combination_families: Vec<String>,
        lifecycle_families: Vec<String>,
        program_digest: WorthQueryEvidenceIdentity,
        breadth_digest: WorthQueryEvidenceIdentity,
        counter_snapshot: String,
    ) -> Result<Self, IdentityDerivationDenial> {
        let summary_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-domain-invariant-summary",
                )
                .field_evidence_identity(WorthQueryEvidenceTag::new("program"), &program_digest)
                .field_evidence_identity(WorthQueryEvidenceTag::new("breadth"), &breadth_digest)
                .field_usize(
                    WorthQueryEvidenceTag::new("declared_collection_count"),
                    declared_collections.len(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("declared_symbol_count"),
                    declared_symbols.len(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("target_combination_count"),
                    target_combination_families.len(),
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("lifecycle_family_count"),
                    lifecycle_families.len(),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("declared_collection"),
                    declared_collections.iter().map(String::as_str),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("declared_symbol"),
                    declared_symbols.iter().map(String::as_str),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("target_combination"),
                    target_combination_families.iter().map(String::as_str),
                )
                .field_value_sequence(
                    WorthQueryEvidenceTag::new("lifecycle_family"),
                    lifecycle_families.iter().map(String::as_str),
                )
                .seal()?;
        Ok(Self {
            declared_collections,
            declared_symbols,
            target_combination_families,
            lifecycle_families,
            program_digest,
            breadth_digest,
            counter_snapshot,
            summary_digest,
        })
    }

    pub fn declared_collections(&self) -> &[String] {
        &self.declared_collections
    }

    pub fn declared_symbols(&self) -> &[String] {
        &self.declared_symbols
    }

    pub fn target_combination_families(&self) -> &[String] {
        &self.target_combination_families
    }

    pub fn lifecycle_families(&self) -> &[String] {
        &self.lifecycle_families
    }

    pub fn program_digest(&self) -> &str {
        self.program_digest.as_str()
    }

    pub fn breadth_digest(&self) -> &str {
        self.breadth_digest.as_str()
    }

    pub fn counter_snapshot(&self) -> &str {
        &self.counter_snapshot
    }

    pub fn summary_digest(&self) -> &str {
        self.summary_digest.as_str()
    }

    pub fn summary_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.summary_digest
    }
}
