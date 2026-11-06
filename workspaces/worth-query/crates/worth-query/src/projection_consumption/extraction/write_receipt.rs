use super::super::consumed::{
    ConsumedContinuityAuthorityIdentity, ConsumedEffectContinuityFact,
    ConsumedProjectionContractProvenance, ConsumedProjectionFactInventory,
    ConsumedProjectionFactSet, ConsumedProjectionSourceTruth, ConsumedRelationEndpointFact,
    ConsumedSourceReferenceFact, ConsumedTargetIdentityFact, ProjectionFactExtractionCounters,
};
use super::super::contracts::MaterializedProjectionContract;
use super::super::facts::ProjectionFactKind;
use super::super::source::ProjectionSourceFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::ProjectionFactExtractionError;
use crate::runtime::WorthQueryWriteReceipt;

pub(super) fn extract_write_receipt_facts(
    contract: &MaterializedProjectionContract,
    receipt: &WorthQueryWriteReceipt,
) -> Result<ConsumedProjectionFactSet, ProjectionFactExtractionError> {
    super::ensure_contract_family(contract, ProjectionSourceFamily::QueryWriteReceipt)?;
    let receipt_commit_identity = receipt.commit_evidence_identity();
    super::ensure_source_identity(contract.source_identity(), receipt_commit_identity.as_str())?;
    let receipt_source_identity = receipt_commit_identity.as_str().to_string();
    let mut facts = ExtractedWriteReceiptFacts::default();
    for fact_family in contract.fact_families() {
        facts.extract_family(fact_family.kind(), receipt, &receipt_source_identity)?;
    }
    if !super::source_reference_inventory_matches(
        contract.source_reference_identities(),
        &facts.source_references,
    ) {
        return Err(
            ProjectionFactExtractionError::SourceReferenceEvidenceMismatch {
                expected_count: contract.source_reference_identities().len(),
                actual_count: facts.source_references.len(),
            },
        );
    }
    Ok(facts.into_fact_set(contract))
}

#[derive(Default)]
struct ExtractedWriteReceiptFacts {
    target_identities: Vec<ConsumedTargetIdentityFact>,
    source_references: Vec<ConsumedSourceReferenceFact>,
    effect_continuity: Vec<ConsumedEffectContinuityFact>,
    relation_endpoints: Vec<ConsumedRelationEndpointFact>,
    evidence_lookup_width: usize,
}

impl ExtractedWriteReceiptFacts {
    fn extract_family(
        &mut self,
        kind: ProjectionFactKind,
        receipt: &WorthQueryWriteReceipt,
        source_identity: &str,
    ) -> Result<(), ProjectionFactExtractionError> {
        match kind {
            ProjectionFactKind::TargetIdentity => self.extract_target(receipt, source_identity)?,
            ProjectionFactKind::SourceReference => {
                self.evidence_lookup_width += 1;
                self.source_references = write_receipt_source_references(receipt);
            }
            ProjectionFactKind::EffectContinuity => {
                self.extract_continuity(receipt, source_identity)?
            }
            ProjectionFactKind::RelationEndpoint => {
                self.extract_relation_endpoint(receipt, source_identity)?
            }
            ProjectionFactKind::EntityIdentity
            | ProjectionFactKind::ViewLocalIdentity
            | ProjectionFactKind::Membership
            | ProjectionFactKind::DisplayField
            | ProjectionFactKind::DerivedField => {}
        }
        Ok(())
    }

    fn extract_target(
        &mut self,
        receipt: &WorthQueryWriteReceipt,
        source_identity: &str,
    ) -> Result<(), ProjectionFactExtractionError> {
        self.evidence_lookup_width += 1;
        let identity = receipt.target_entity_identity().ok_or_else(|| {
            missing_write_evidence(source_identity, ProjectionFactKind::TargetIdentity)
        })?;
        self.target_identities
            .push(ConsumedTargetIdentityFact::new(identity.clone()));
        Ok(())
    }

    fn extract_continuity(
        &mut self,
        receipt: &WorthQueryWriteReceipt,
        source_identity: &str,
    ) -> Result<(), ProjectionFactExtractionError> {
        self.evidence_lookup_width += 1;
        let continuity = receipt.continuity_mutation_evidence().ok_or_else(|| {
            missing_write_evidence(source_identity, ProjectionFactKind::EffectContinuity)
        })?;
        self.effect_continuity
            .push(ConsumedEffectContinuityFact::new(
                continuity.family(),
                continuity.outcome_class(),
                ConsumedContinuityAuthorityIdentity::new(
                    continuity.prior_authoritative_identity().as_str(),
                ),
                continuity
                    .successor_authoritative_identities()
                    .iter()
                    .map(|identity| ConsumedContinuityAuthorityIdentity::new(identity.as_str()))
                    .collect(),
                continuity.resolved_target_entity_identity().cloned(),
                continuity
                    .target_collection()
                    .map(|collection| collection.as_str().to_string()),
                continuity.lineage_digest().as_str(),
                continuity.continuity_resolution_digest().as_str(),
            ));
        Ok(())
    }

    fn extract_relation_endpoint(
        &mut self,
        receipt: &WorthQueryWriteReceipt,
        source_identity: &str,
    ) -> Result<(), ProjectionFactExtractionError> {
        self.evidence_lookup_width += 1;
        let resolved = receipt.target_evidence().resolved();
        if resolved.collection().is_none() || resolved.entity_identity().is_none() {
            return Err(missing_write_evidence(
                source_identity,
                ProjectionFactKind::RelationEndpoint,
            ));
        }
        self.relation_endpoints
            .push(ConsumedRelationEndpointFact::new(
                resolved.target_class(),
                resolved
                    .collection()
                    .map(|collection| collection.as_str().to_string()),
                resolved.entity_identity().cloned(),
            ));
        Ok(())
    }

    fn into_fact_set(
        self,
        contract: &MaterializedProjectionContract,
    ) -> Result<ConsumedProjectionFactSet, IdentityDerivationDenial> {
        let extracted_count = self.target_identities.len()
            + self.source_references.len()
            + self.effect_continuity.len()
            + self.relation_endpoints.len();
        Ok(ConsumedProjectionFactSet::new(
            ConsumedProjectionContractProvenance::from_contract(contract),
            ConsumedProjectionSourceTruth::from_contract(
                contract,
                crate::projection_consumption::ConsumedNativeLayoutProof::from_contract(
                    contract, 0,
                ),
            ),
            ProjectionFactExtractionCounters::new(
                contract.fact_families().len(),
                contract.fact_families().len(),
                extracted_count,
                0,
                self.evidence_lookup_width,
            ),
            ConsumedProjectionFactInventory {
                entity_identities: Vec::new(),
                view_local_identities: Vec::new(),
                memberships: Vec::new(),
                display_fields: Vec::new(),
                derived_fields: Vec::new(),
                target_identities: self.target_identities,
                source_references: self.source_references,
                effect_continuity_facts: self.effect_continuity,
                relation_endpoints: self.relation_endpoints,
            },
        )?)
    }
}

fn missing_write_evidence(
    source_identity: &str,
    fact_kind: ProjectionFactKind,
) -> ProjectionFactExtractionError {
    ProjectionFactExtractionError::MissingWriteReceiptEvidence {
        source_identity: source_identity.to_owned(),
        fact_kind,
    }
}

fn write_receipt_source_references(
    receipt: &WorthQueryWriteReceipt,
) -> Vec<ConsumedSourceReferenceFact> {
    let mut references = Vec::new();
    if let Some(provenance) = receipt.provenance_evidence() {
        references.push(ConsumedSourceReferenceFact::new(
            "bridge_provenance_execution_record",
            provenance.execution_record_digest().as_str(),
        ));
    }
    if let Some(symbolic) = receipt.symbolic_target_reference_evidence() {
        references.push(ConsumedSourceReferenceFact::new(
            "symbolic_target_reference",
            symbolic.symbol().as_str(),
        ));
    }
    references
}
