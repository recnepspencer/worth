use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::WorthQueryImpactSemanticProjection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvalidationSemanticAccessKey {
    contract_key: worth_foundational::facade::AspectKey,
    contract_identity: worth_foundational::facade::AspectIdentity,
    contract_revision: worth_foundational::facade::AspectContractRevision,
    field_path: crate::projection_consumption::ProjectionFactFieldPath,
    expected_shape: worth_foundational::facade::AspectValuePosture,
    absence: worth_foundational::facade::AbsenceLaw,
    lane: crate::domain_installation::WorthQueryNativeFactLane,
}

impl WorthQueryInvalidationSemanticAccessKey {
    pub fn contract_key(&self) -> &worth_foundational::facade::AspectKey {
        &self.contract_key
    }

    pub fn field_path(&self) -> &crate::projection_consumption::ProjectionFactFieldPath {
        &self.field_path
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConsumerInvalidationSemanticProjection {
    identity: WorthQueryEvidenceIdentity,
    impact: WorthQueryImpactSemanticProjection,
    affected_native_keys: Vec<WorthQueryInvalidationSemanticAccessKey>,
    conditional_path:
        Vec<crate::domain_installation::operation_execution::WorthQueryConditionalTraceMeaning>,
    disposition: super::super::WorthQueryConsumerInvalidationDisposition,
    cause: super::super::WorthQueryConsumerInvalidationCause,
    locality: super::super::WorthQueryConsumerInvalidationLocality,
    continuation: super::super::WorthQueryConsumerInvalidationContinuation,
}

impl WorthQueryConsumerInvalidationSemanticProjection {
    pub(crate) const fn identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.identity
    }

    pub const fn impact(&self) -> &WorthQueryImpactSemanticProjection {
        &self.impact
    }

    pub fn affected_native_keys(&self) -> &[WorthQueryInvalidationSemanticAccessKey] {
        &self.affected_native_keys
    }

    pub fn conditional_path(
        &self,
    ) -> &[crate::domain_installation::operation_execution::WorthQueryConditionalTraceMeaning] {
        &self.conditional_path
    }

    pub const fn disposition(&self) -> super::super::WorthQueryConsumerInvalidationDisposition {
        self.disposition
    }

    pub fn canonical_bytes(&self) -> &[u8; 32] {
        self.identity.canonical_digest().value().bytes()
    }

    pub fn semantically_converges_with(&self, candidate: &Self) -> bool {
        self.identity.eq_same_scheme(&candidate.identity) == Ok(true) && self == candidate
    }
}

impl super::super::WorthQueryConsumerInvalidationDelta {
    pub fn semantic_projection(&self) -> WorthQueryConsumerInvalidationSemanticProjection {
        let impact = self.impact().semantic_projection();
        let affected_native_keys = self
            .affected_native_keys()
            .iter()
            .map(semantic_access_key)
            .collect::<Vec<_>>();
        let conditional_path = self
            .conditional_provenance()
            .iter()
            .map(crate::domain_installation::operation_execution::workflow_conditional_trace::conditional_trace_meaning)
            .collect();
        let identity = super::encoding::invalidation_identity(self, &impact, &affected_native_keys);
        WorthQueryConsumerInvalidationSemanticProjection {
            identity,
            impact,
            affected_native_keys,
            conditional_path,
            disposition: self.disposition(),
            cause: self.cause().clone(),
            locality: self.locality(),
            continuation: self.continuation(),
        }
    }
}

fn semantic_access_key(
    key: &crate::domain_installation::WorthQueryNativeAccessKey,
) -> WorthQueryInvalidationSemanticAccessKey {
    WorthQueryInvalidationSemanticAccessKey {
        contract_key: key.contract_key().clone(),
        contract_identity: key.contract_identity(),
        contract_revision: key.contract_revision(),
        field_path: key.field_path().clone(),
        expected_shape: key.expected_shape(),
        absence: key.absence_posture(),
        lane: key.lane(),
    }
}

impl WorthQueryInvalidationSemanticAccessKey {
    pub(super) const fn contract_identity(&self) -> worth_foundational::facade::AspectIdentity {
        self.contract_identity
    }

    pub(super) const fn contract_revision(
        &self,
    ) -> worth_foundational::facade::AspectContractRevision {
        self.contract_revision
    }

    pub(super) const fn expected_shape(&self) -> worth_foundational::facade::AspectValuePosture {
        self.expected_shape
    }

    pub(super) const fn absence(&self) -> worth_foundational::facade::AbsenceLaw {
        self.absence
    }

    pub(super) const fn lane(&self) -> crate::domain_installation::WorthQueryNativeFactLane {
        self.lane
    }
}
