use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphCompositionBreadth {
    component_count: usize,
    symbolic_entity_declaration_count: usize,
    symbolic_relation_declaration_count: usize,
    breadth_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryGraphCompositionBreadth {
    pub(crate) fn new(
        component_count: usize,
        symbolic_entity_declaration_count: usize,
        symbolic_relation_declaration_count: usize,
    ) -> Result<Self, IdentityDerivationDenial> {
        let breadth_digest =
            worth_query_evidence_identity(WorthQueryEvidenceScope::MutationEvidenceAggregateDigest)
                .field_shape(
                    WorthQueryEvidenceTag::new("role"),
                    "graph-composition-breadth",
                )
                .field_usize(WorthQueryEvidenceTag::new("components"), component_count)
                .field_usize(
                    WorthQueryEvidenceTag::new("symbolic_entities"),
                    symbolic_entity_declaration_count,
                )
                .field_usize(
                    WorthQueryEvidenceTag::new("symbolic_relations"),
                    symbolic_relation_declaration_count,
                )
                .seal()?;
        Ok(Self {
            component_count,
            symbolic_entity_declaration_count,
            symbolic_relation_declaration_count,
            breadth_digest,
        })
    }

    pub(crate) fn empty() -> Result<Self, IdentityDerivationDenial> {
        Ok(Self::new(0, 0, 0)?)
    }

    pub fn component_count(&self) -> usize {
        self.component_count
    }

    pub fn symbolic_entity_declaration_count(&self) -> usize {
        self.symbolic_entity_declaration_count
    }

    pub fn symbolic_relation_declaration_count(&self) -> usize {
        self.symbolic_relation_declaration_count
    }

    pub fn breadth_digest(&self) -> &str {
        self.breadth_digest.as_str()
    }

    pub fn breadth_evidence_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.breadth_digest
    }
}
