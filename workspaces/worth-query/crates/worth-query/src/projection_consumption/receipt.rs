use super::consumed::ConsumedProjectionFactSet;
use super::contracts::ProjectionContractSupportPosture;
use super::eligibility::ProjectionConsumptionWarningKind;
use super::envelope::SelfDescribingProjectionConsumptionEnvelope;
use super::facts::ProjectionMaterializedFactPosture;
use super::identity::{compose_receipt_digest, compose_receipt_integrity_digest};
use super::receipt_transitions::ProjectionConsumptionDeferredNeighborFamily;
use super::receipt_transitions::ProjectionConsumptionTransitionRules;
use super::source::ProjectionSourceFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectionConsumptionReceipt {
    declaration_digest: String,
    contract_digest: String,
    fact_set_digest: String,
    source_family: ProjectionSourceFamily,
    source_identity: String,
    support_posture: ProjectionContractSupportPosture,
    materialized_fact_posture: Option<ProjectionMaterializedFactPosture>,
    admitted_fact_family_count: usize,
    extracted_fact_count: usize,
    authority_reopen_count: usize,
    deferred_neighbors: Vec<ProjectionConsumptionDeferredNeighborFamily>,
    counter_snapshot_digest: String,
    integrity_digest: String,
    receipt_digest: String,
}

impl ProjectionConsumptionReceipt {
    pub(crate) fn from_fact_set(
        fact_set: &ConsumedProjectionFactSet,
    ) -> Result<Self, IdentityDerivationDenial> {
        let counter_snapshot_digest = fact_set.counters().digest()?;
        let transition_rules = ProjectionConsumptionTransitionRules::current_phase_five_surface()?;
        let deferred_neighbors = transition_rules
            .rules()
            .iter()
            .filter_map(|rule| rule.deferred_neighbor())
            .collect::<Vec<_>>();
        let materialized_fact_posture_digest = fact_set
            .materialized_fact_posture()
            .map(ProjectionMaterializedFactPosture::posture_digest);
        let integrity_digest = compose_receipt_integrity_digest(
            fact_set.fact_set_digest(),
            &counter_snapshot_digest,
            fact_set.source_identity(),
            materialized_fact_posture_digest,
        )?;
        let receipt_digest = compose_receipt_digest(
            fact_set.declaration_digest(),
            fact_set.contract_digest(),
            fact_set.fact_set_digest(),
            fact_set.source_family(),
            fact_set.source_identity(),
            fact_set.support_posture(),
            materialized_fact_posture_digest,
            fact_set.support_posture().warning_kinds(),
            &deferred_neighbors,
        )?;
        Ok(Self {
            declaration_digest: fact_set.declaration_digest().to_string(),
            contract_digest: fact_set.contract_digest().to_string(),
            fact_set_digest: fact_set.fact_set_digest().to_string(),
            source_family: fact_set.source_family(),
            source_identity: fact_set.source_identity().to_string(),
            support_posture: fact_set.support_posture().clone(),
            materialized_fact_posture: fact_set.materialized_fact_posture().cloned(),
            admitted_fact_family_count: fact_set.counters().admitted_fact_family_count(),
            extracted_fact_count: fact_set.counters().extracted_fact_count(),
            authority_reopen_count: fact_set.counters().authority_reopen_count(),
            deferred_neighbors,
            counter_snapshot_digest,
            integrity_digest,
            receipt_digest,
        })
    }

    pub fn declaration_digest(&self) -> &str {
        &self.declaration_digest
    }

    pub fn contract_digest(&self) -> &str {
        &self.contract_digest
    }

    pub fn fact_set_digest(&self) -> &str {
        &self.fact_set_digest
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

    pub fn materialized_fact_posture(&self) -> Option<&ProjectionMaterializedFactPosture> {
        self.materialized_fact_posture.as_ref()
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

    pub fn deferred_neighbors(&self) -> &[ProjectionConsumptionDeferredNeighborFamily] {
        &self.deferred_neighbors
    }

    pub fn counter_snapshot_digest(&self) -> &str {
        &self.counter_snapshot_digest
    }

    pub fn integrity_digest(&self) -> &str {
        &self.integrity_digest
    }

    pub fn receipt_digest(&self) -> &str {
        &self.receipt_digest
    }

    pub fn transition_rules(
        &self,
    ) -> Result<ProjectionConsumptionTransitionRules, IdentityDerivationDenial> {
        ProjectionConsumptionTransitionRules::current_phase_five_surface()
    }

    pub fn projection_consumption_envelope(
        &self,
    ) -> Result<SelfDescribingProjectionConsumptionEnvelope, IdentityDerivationDenial> {
        SelfDescribingProjectionConsumptionEnvelope::from_receipt(self)
    }
}
