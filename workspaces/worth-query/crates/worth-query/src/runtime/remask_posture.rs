use super::WorthQueryRuntimeStateKind;
use crate::evidence_identity::WorthQueryEvidenceIdentity;

use super::evidence_identities::runtime_remask_posture_identity;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeRemaskDispositionKind {
    Remasked,
    Denied,
}

impl WorthQueryRuntimeRemaskDispositionKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Remasked => "remasked",
            Self::Denied => "denied",
        }
    }

    pub(crate) fn state_kind(self) -> WorthQueryRuntimeStateKind {
        match self {
            Self::Remasked => WorthQueryRuntimeStateKind::Remasked,
            Self::Denied => WorthQueryRuntimeStateKind::Denied,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeRemaskReasonKind {
    PolicyDrift,
    TenantDrift,
    RelationshipProofDrift,
    SchemaContextDrift,
}

impl WorthQueryRuntimeRemaskReasonKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PolicyDrift => "policy_drift",
            Self::TenantDrift => "tenant_drift",
            Self::RelationshipProofDrift => "relationship_proof_drift",
            Self::SchemaContextDrift => "schema_context_drift",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeRemaskProjection {
    disposition_kind: WorthQueryRuntimeRemaskDispositionKind,
    reason_kind: WorthQueryRuntimeRemaskReasonKind,
    policy_digest: String,
    tenant_truth_digest: String,
    tenant_schema_digest: String,
    relationship_proof_digest: String,
    schema_context_digest: String,
}

impl WorthQueryRuntimeRemaskProjection {
    pub fn remasked(
        reason_kind: WorthQueryRuntimeRemaskReasonKind,
        policy_digest: impl Into<String>,
        tenant_truth_digest: impl Into<String>,
        tenant_schema_digest: impl Into<String>,
        relationship_proof_digest: impl Into<String>,
        schema_context_digest: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryRuntimeRemaskDispositionKind::Remasked,
            reason_kind,
            policy_digest,
            tenant_truth_digest,
            tenant_schema_digest,
            relationship_proof_digest,
            schema_context_digest,
        )
    }

    pub fn denied(
        reason_kind: WorthQueryRuntimeRemaskReasonKind,
        policy_digest: impl Into<String>,
        tenant_truth_digest: impl Into<String>,
        tenant_schema_digest: impl Into<String>,
        relationship_proof_digest: impl Into<String>,
        schema_context_digest: impl Into<String>,
    ) -> Self {
        Self::new(
            WorthQueryRuntimeRemaskDispositionKind::Denied,
            reason_kind,
            policy_digest,
            tenant_truth_digest,
            tenant_schema_digest,
            relationship_proof_digest,
            schema_context_digest,
        )
    }

    fn new(
        disposition_kind: WorthQueryRuntimeRemaskDispositionKind,
        reason_kind: WorthQueryRuntimeRemaskReasonKind,
        policy_digest: impl Into<String>,
        tenant_truth_digest: impl Into<String>,
        tenant_schema_digest: impl Into<String>,
        relationship_proof_digest: impl Into<String>,
        schema_context_digest: impl Into<String>,
    ) -> Self {
        Self {
            disposition_kind,
            reason_kind,
            policy_digest: policy_digest.into(),
            tenant_truth_digest: tenant_truth_digest.into(),
            tenant_schema_digest: tenant_schema_digest.into(),
            relationship_proof_digest: relationship_proof_digest.into(),
            schema_context_digest: schema_context_digest.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeRemaskPosture {
    disposition_kind: WorthQueryRuntimeRemaskDispositionKind,
    reason_kind: WorthQueryRuntimeRemaskReasonKind,
    support_identity: WorthQueryEvidenceIdentity,
    basis_identity: WorthQueryEvidenceIdentity,
    policy_identity: WorthQueryEvidenceIdentity,
    tenant_truth_identity: WorthQueryEvidenceIdentity,
    tenant_schema_identity: WorthQueryEvidenceIdentity,
    relationship_proof_identity: WorthQueryEvidenceIdentity,
    schema_context_identity: WorthQueryEvidenceIdentity,
    remask_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeRemaskPosture {
    fn new(
        disposition_kind: WorthQueryRuntimeRemaskDispositionKind,
        reason_kind: WorthQueryRuntimeRemaskReasonKind,
        support_identity: WorthQueryEvidenceIdentity,
        basis_identity: WorthQueryEvidenceIdentity,
        policy_digest: impl Into<String>,
        tenant_truth_digest: impl Into<String>,
        tenant_schema_digest: impl Into<String>,
        relationship_proof_digest: impl Into<String>,
        schema_context_digest: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let policy_digest = policy_digest.into();
        let tenant_truth_digest = tenant_truth_digest.into();
        let tenant_schema_digest = tenant_schema_digest.into();
        let relationship_proof_digest = relationship_proof_digest.into();
        let schema_context_digest = schema_context_digest.into();
        let policy_identity =
            remask_drift_label_identity("remask_policy_label_v1", &policy_digest)?;
        let tenant_truth_identity =
            remask_drift_label_identity("remask_tenant_truth_label_v1", &tenant_truth_digest)?;
        let tenant_schema_identity =
            remask_drift_label_identity("remask_tenant_schema_label_v1", &tenant_schema_digest)?;
        let relationship_proof_identity = remask_drift_label_identity(
            "remask_relationship_proof_label_v1",
            &relationship_proof_digest,
        )?;
        let schema_context_identity =
            remask_drift_label_identity("remask_schema_context_label_v1", &schema_context_digest)?;
        let remask_identity = runtime_remask_posture_identity(
            disposition_kind,
            reason_kind,
            &support_identity,
            &basis_identity,
            &policy_identity,
            &tenant_truth_identity,
            &tenant_schema_identity,
            &relationship_proof_identity,
            &schema_context_identity,
        )?;
        Ok(Self {
            disposition_kind,
            reason_kind,
            support_identity,
            basis_identity,
            policy_identity,
            tenant_truth_identity,
            tenant_schema_identity,
            relationship_proof_identity,
            schema_context_identity,
            remask_identity,
        })
    }

    pub(crate) fn from_activation_projection(
        projection: &WorthQueryRuntimeRemaskProjection,
        support_identity: &WorthQueryEvidenceIdentity,
        basis_identity: &WorthQueryEvidenceIdentity,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self::new(
            projection.disposition_kind,
            projection.reason_kind,
            support_identity.clone(),
            basis_identity.clone(),
            projection.policy_digest.clone(),
            projection.tenant_truth_digest.clone(),
            projection.tenant_schema_digest.clone(),
            projection.relationship_proof_digest.clone(),
            projection.schema_context_digest.clone(),
        )?)
    }

    pub fn disposition_kind(&self) -> WorthQueryRuntimeRemaskDispositionKind {
        self.disposition_kind
    }

    pub fn reason_kind(&self) -> WorthQueryRuntimeRemaskReasonKind {
        self.reason_kind
    }

    pub fn support_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_identity
    }

    pub fn support_for_reporting(&self) -> &str {
        self.support_identity.as_str()
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn basis_for_reporting(&self) -> &str {
        self.basis_identity.as_str()
    }

    pub fn policy_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.policy_identity
    }

    pub fn policy_for_reporting(&self) -> &str {
        self.policy_identity.as_str()
    }

    pub fn tenant_truth_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.tenant_truth_identity
    }

    pub fn tenant_truth_for_reporting(&self) -> &str {
        self.tenant_truth_identity.as_str()
    }

    pub fn tenant_schema_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.tenant_schema_identity
    }

    pub fn tenant_schema_for_reporting(&self) -> &str {
        self.tenant_schema_identity.as_str()
    }

    pub fn relationship_proof_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.relationship_proof_identity
    }

    pub fn relationship_proof_for_reporting(&self) -> &str {
        self.relationship_proof_identity.as_str()
    }

    pub fn schema_context_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.schema_context_identity
    }

    pub fn schema_context_for_reporting(&self) -> &str {
        self.schema_context_identity.as_str()
    }

    pub fn remask_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.remask_identity
    }

    pub fn remask_for_reporting(&self) -> &str {
        self.remask_identity.as_str()
    }
}

fn remask_drift_label_identity(
    identity_family: &str,
    digest: &str,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                identity_family,
            )
            .field_shape(WorthQueryEvidenceTag::new("digest"), digest)
            .seal()?,
    )
}
