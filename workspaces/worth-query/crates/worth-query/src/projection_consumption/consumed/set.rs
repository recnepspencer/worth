use super::facts::{
    ConsumedEffectContinuityFact, ConsumedEntityIdentityFact, ConsumedMembershipFact,
    ConsumedRelationEndpointFact, ConsumedSourceReferenceFact, ConsumedTargetIdentityFact,
    ConsumedViewLocalIdentityFact,
};
use super::field_value_fact::ConsumedFieldValueFact;
use super::native_layout::ConsumedNativeLayoutProof;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::projection_consumption::identity::{
    compose_consumed_projection_fact_set_digest, compose_extraction_counters_digest,
};
use crate::projection_consumption::receipt::ProjectionConsumptionReceipt;
use crate::projection_consumption::ProjectionMaterializedFactPosture;

use super::super::contracts::ProjectionContractSupportPosture;
use super::super::source::{ProjectionSourceFamily, ProjectionSourceIdentity};
use super::super::MaterializedProjectionContract;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ProjectionFactExtractionCounters {
    declared_fact_family_count: usize,
    admitted_fact_family_count: usize,
    extracted_fact_count: usize,
    source_row_width_consumed: usize,
    source_evidence_lookup_width: usize,
    authority_reopen_count: usize,
}

impl ProjectionFactExtractionCounters {
    pub fn declared_fact_family_count(&self) -> usize {
        self.declared_fact_family_count
    }

    pub fn admitted_fact_family_count(&self) -> usize {
        self.admitted_fact_family_count
    }

    pub fn extracted_fact_count(&self) -> usize {
        self.extracted_fact_count
    }

    pub fn source_row_width_consumed(&self) -> usize {
        self.source_row_width_consumed
    }

    pub fn source_evidence_lookup_width(&self) -> usize {
        self.source_evidence_lookup_width
    }

    pub fn authority_reopen_count(&self) -> usize {
        self.authority_reopen_count
    }

    pub(crate) fn digest(&self) -> Result<String, IdentityDerivationDenial> {
        compose_extraction_counters_digest(self)
    }

    pub(crate) fn new(
        declared_fact_family_count: usize,
        admitted_fact_family_count: usize,
        extracted_fact_count: usize,
        source_row_width_consumed: usize,
        source_evidence_lookup_width: usize,
    ) -> Self {
        Self {
            declared_fact_family_count,
            admitted_fact_family_count,
            extracted_fact_count,
            source_row_width_consumed,
            source_evidence_lookup_width,
            authority_reopen_count: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConsumedProjectionContractProvenance {
    declaration_digest: String,
    contract_digest: String,
    support_posture: ProjectionContractSupportPosture,
    materialized_fact_posture: Option<ProjectionMaterializedFactPosture>,
}

impl ConsumedProjectionContractProvenance {
    pub(crate) fn from_contract(contract: &MaterializedProjectionContract) -> Self {
        Self {
            declaration_digest: contract.declaration_digest().to_owned(),
            contract_digest: contract.contract_digest().to_owned(),
            support_posture: contract.support_posture().clone(),
            materialized_fact_posture: contract.materialized_fact_posture().cloned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConsumedProjectionSourceTruth {
    source_family: ProjectionSourceFamily,
    source_identity: ProjectionSourceIdentity,
    native_layout: ConsumedNativeLayoutProof,
}

impl ConsumedProjectionSourceTruth {
    pub(crate) fn from_contract(
        contract: &MaterializedProjectionContract,
        native_layout: ConsumedNativeLayoutProof,
    ) -> Self {
        Self {
            source_family: contract.source_family(),
            source_identity: contract.source_identity_handle().clone(),
            native_layout,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ConsumedProjectionFactInventory {
    pub(crate) entity_identities: Vec<ConsumedEntityIdentityFact>,
    pub(crate) view_local_identities: Vec<ConsumedViewLocalIdentityFact>,
    pub(crate) memberships: Vec<ConsumedMembershipFact>,
    pub(crate) display_fields: Vec<ConsumedFieldValueFact>,
    pub(crate) derived_fields: Vec<ConsumedFieldValueFact>,
    pub(crate) target_identities: Vec<ConsumedTargetIdentityFact>,
    pub(crate) source_references: Vec<ConsumedSourceReferenceFact>,
    pub(crate) effect_continuity_facts: Vec<ConsumedEffectContinuityFact>,
    pub(crate) relation_endpoints: Vec<ConsumedRelationEndpointFact>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConsumedProjectionFactSet {
    provenance: ConsumedProjectionContractProvenance,
    source_truth: ConsumedProjectionSourceTruth,
    counters: ProjectionFactExtractionCounters,
    fact_set_digest: String,
    facts: ConsumedProjectionFactInventory,
}

impl ConsumedProjectionFactSet {
    pub fn declaration_digest(&self) -> &str {
        &self.provenance.declaration_digest
    }

    pub fn contract_digest(&self) -> &str {
        &self.provenance.contract_digest
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_truth.source_family
    }

    pub fn source_identity(&self) -> &str {
        self.source_truth.source_identity.as_str()
    }

    pub fn source_identity_handle(&self) -> &ProjectionSourceIdentity {
        &self.source_truth.source_identity
    }

    pub fn support_posture(&self) -> &ProjectionContractSupportPosture {
        &self.provenance.support_posture
    }

    pub fn counters(&self) -> &ProjectionFactExtractionCounters {
        &self.counters
    }

    pub fn materialized_fact_posture(&self) -> Option<&ProjectionMaterializedFactPosture> {
        self.provenance.materialized_fact_posture.as_ref()
    }

    pub fn fact_set_digest(&self) -> &str {
        &self.fact_set_digest
    }

    pub(crate) fn native_layout(&self) -> &ConsumedNativeLayoutProof {
        &self.source_truth.native_layout
    }

    pub fn entity_identities(&self) -> &[ConsumedEntityIdentityFact] {
        &self.facts.entity_identities
    }

    pub fn view_local_identities(&self) -> &[ConsumedViewLocalIdentityFact] {
        &self.facts.view_local_identities
    }

    pub fn memberships(&self) -> &[ConsumedMembershipFact] {
        &self.facts.memberships
    }

    pub fn display_fields(&self) -> &[ConsumedFieldValueFact] {
        &self.facts.display_fields
    }

    pub fn derived_fields(&self) -> &[ConsumedFieldValueFact] {
        &self.facts.derived_fields
    }

    pub fn target_identities(&self) -> &[ConsumedTargetIdentityFact] {
        &self.facts.target_identities
    }

    pub fn source_references(&self) -> &[ConsumedSourceReferenceFact] {
        &self.facts.source_references
    }

    pub fn effect_continuity_facts(&self) -> &[ConsumedEffectContinuityFact] {
        &self.facts.effect_continuity_facts
    }

    pub fn relation_endpoints(&self) -> &[ConsumedRelationEndpointFact] {
        &self.facts.relation_endpoints
    }

    pub fn issue_receipt(&self) -> Result<ProjectionConsumptionReceipt, IdentityDerivationDenial> {
        ProjectionConsumptionReceipt::from_fact_set(self)
    }

    pub(crate) fn new(
        provenance: ConsumedProjectionContractProvenance,
        source_truth: ConsumedProjectionSourceTruth,
        counters: ProjectionFactExtractionCounters,
        facts: ConsumedProjectionFactInventory,
    ) -> Result<Self, IdentityDerivationDenial> {
        let fact_set_digest = compose_consumed_projection_fact_set_digest(
            &provenance.declaration_digest,
            &provenance.contract_digest,
            source_truth.source_family,
            source_truth.source_identity.as_str(),
            &provenance.support_posture,
            provenance.materialized_fact_posture.as_ref(),
            &counters,
            &facts.entity_identities,
            &facts.view_local_identities,
            &facts.memberships,
            &facts.display_fields,
            &facts.derived_fields,
            &facts.target_identities,
            &facts.source_references,
            &facts.effect_continuity_facts,
            &facts.relation_endpoints,
        )?;
        Ok(Self {
            provenance,
            source_truth,
            counters,
            fact_set_digest,
            facts,
        })
    }
}
