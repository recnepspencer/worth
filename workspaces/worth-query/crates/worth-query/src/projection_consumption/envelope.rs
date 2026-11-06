use super::contracts::ProjectionContractSupportPosture;
use super::eligibility::ProjectionConsumptionWarningKind;
use super::identity::{
    compose_envelope_boundary_digest, compose_envelope_digest, compose_envelope_performance_digest,
    compose_envelope_source_refs_digest,
};
use super::receipt::ProjectionConsumptionReceipt;
use super::receipt_transitions::ProjectionConsumptionDeferredNeighborFamily;
use super::source::ProjectionSourceFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionEnvelopeSourceRefs {
    receipt_digest: String,
    fact_set_digest: String,
    contract_digest: String,
    source_refs_digest: String,
}

impl ProjectionConsumptionEnvelopeSourceRefs {
    fn from_receipt(
        receipt: &ProjectionConsumptionReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let receipt_digest = receipt.receipt_digest().to_string();
        let fact_set_digest = receipt.fact_set_digest().to_string();
        let contract_digest = receipt.contract_digest().to_string();
        let source_refs_digest = compose_envelope_source_refs_digest(
            &receipt_digest,
            &fact_set_digest,
            &contract_digest,
        )?;
        Ok(Self {
            receipt_digest,
            fact_set_digest,
            contract_digest,
            source_refs_digest,
        })
    }

    pub fn receipt_digest(&self) -> &str {
        &self.receipt_digest
    }

    pub fn fact_set_digest(&self) -> &str {
        &self.fact_set_digest
    }

    pub fn contract_digest(&self) -> &str {
        &self.contract_digest
    }

    pub fn source_refs_digest(&self) -> &str {
        &self.source_refs_digest
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelfDescribingProjectionConsumptionEnvelope {
    source_family: ProjectionSourceFamily,
    source_identity: String,
    support_posture: ProjectionContractSupportPosture,
    admitted_fact_family_count: usize,
    extracted_fact_count: usize,
    authority_reopen_count: usize,
    transition_rules_digest: String,
    deferred_neighbors: Vec<ProjectionConsumptionDeferredNeighborFamily>,
    integrity_digest: String,
    performance_digest: Box<str>,
    boundary_digest: String,
    sources: ProjectionConsumptionEnvelopeSourceRefs,
    envelope_digest: String,
}

impl SelfDescribingProjectionConsumptionEnvelope {
    pub(crate) fn from_receipt(
        receipt: &ProjectionConsumptionReceipt,
    ) -> Result<Self, IdentityDerivationDenial> {
        let sources = ProjectionConsumptionEnvelopeSourceRefs::from_receipt(receipt)?;
        let transition_rules = receipt.transition_rules();
        let deferred_neighbors = receipt.deferred_neighbors().to_vec();
        let transition_rules_digest = transition_rules.rules_digest().to_string();
        let integrity_digest = receipt.integrity_digest().to_string();
        let performance_digest = compose_envelope_performance_digest(
            receipt.receipt_digest(),
            receipt.counter_snapshot_digest(),
        )?
        .into_boxed_str();
        let boundary_digest = compose_envelope_boundary_digest(
            receipt.source_family(),
            receipt.source_identity(),
            receipt.support_posture(),
            receipt.warning_kinds(),
        )?;
        let envelope_digest = compose_envelope_digest(
            receipt.receipt_digest(),
            &integrity_digest,
            &performance_digest,
            &boundary_digest,
            &transition_rules_digest,
            sources.source_refs_digest(),
        )?;
        Ok(Self {
            source_family: receipt.source_family(),
            source_identity: receipt.source_identity().to_string(),
            support_posture: receipt.support_posture().clone(),
            admitted_fact_family_count: receipt.admitted_fact_family_count(),
            extracted_fact_count: receipt.extracted_fact_count(),
            authority_reopen_count: receipt.authority_reopen_count(),
            transition_rules_digest,
            deferred_neighbors,
            integrity_digest,
            performance_digest,
            boundary_digest,
            sources,
            envelope_digest,
        })
    }

    pub fn source_family(&self) -> ProjectionSourceFamily {
        self.source_family
    }

    pub fn source_identity(&self) -> &str {
        &self.source_identity
    }

    pub fn support_posture(&self) -> &ProjectionContractSupportPosture {
        &self.support_posture
    }

    pub fn warning_kinds(&self) -> &[ProjectionConsumptionWarningKind] {
        self.support_posture.warning_kinds()
    }

    pub fn admitted_fact_family_count(&self) -> usize {
        self.admitted_fact_family_count
    }

    pub fn extracted_fact_count(&self) -> usize {
        self.extracted_fact_count
    }

    pub fn authority_reopen_count(&self) -> usize {
        self.authority_reopen_count
    }

    pub fn transition_rules_digest(&self) -> &str {
        &self.transition_rules_digest
    }

    pub fn deferred_neighbors(&self) -> &[ProjectionConsumptionDeferredNeighborFamily] {
        &self.deferred_neighbors
    }

    pub fn integrity_digest(&self) -> &str {
        &self.integrity_digest
    }

    pub fn performance_digest(&self) -> &str {
        &self.performance_digest
    }

    pub fn boundary_digest(&self) -> &str {
        &self.boundary_digest
    }

    pub fn sources(&self) -> &ProjectionConsumptionEnvelopeSourceRefs {
        &self.sources
    }

    pub fn envelope_digest(&self) -> &str {
        &self.envelope_digest
    }
}
