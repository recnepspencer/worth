use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryOrdinaryRuntimePostureKind {
    Current,
    Remasked,
    Pending,
    Failed,
    Stale,
    Cancelled,
    Retried,
    Revalidating,
    Superseded,
    Denied,
    Unresolved,
    Unsupported,
}

impl WorthQueryOrdinaryRuntimePostureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Remasked => "remasked",
            Self::Pending => "pending",
            Self::Failed => "failed",
            Self::Stale => "stale",
            Self::Cancelled => "cancelled",
            Self::Retried => "retried",
            Self::Revalidating => "revalidating",
            Self::Superseded => "superseded",
            Self::Denied => "denied",
            Self::Unresolved => "unresolved",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryOrdinaryRuntimeCausePostureKind {
    Ordinary,
    TimeOnly,
    MixedCause,
}

impl WorthQueryOrdinaryRuntimeCausePostureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ordinary => "ordinary",
            Self::TimeOnly => "time_only",
            Self::MixedCause => "mixed_cause",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryOrdinaryRuntimeAsyncPostureKind {
    Pending,
    Current,
    Failed,
    Stale,
    Cancelled,
    Retried,
    Revalidating,
    Superseded,
    Denied,
    Unresolved,
}

impl WorthQueryOrdinaryRuntimeAsyncPostureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Current => "current",
            Self::Failed => "failed",
            Self::Stale => "stale",
            Self::Cancelled => "cancelled",
            Self::Retried => "retried",
            Self::Revalidating => "revalidating",
            Self::Superseded => "superseded",
            Self::Denied => "denied",
            Self::Unresolved => "unresolved",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryOrdinaryRuntimeBasisPostureKind {
    Stable,
    BasisDrift,
    GenerationDrift,
}

impl WorthQueryOrdinaryRuntimeBasisPostureKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::BasisDrift => "basis_drift",
            Self::GenerationDrift => "generation_drift",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryOrdinaryRuntimeRemaskPostureKind {
    PolicyDrift,
    TenantDrift,
    RelationshipProofDrift,
    SchemaContextDrift,
}

impl WorthQueryOrdinaryRuntimeRemaskPostureKind {
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
pub struct WorthQueryOrdinaryRuntimePosture {
    kind: WorthQueryOrdinaryRuntimePostureKind,
    cause_posture: WorthQueryOrdinaryRuntimeCausePostureKind,
    async_posture: Option<WorthQueryOrdinaryRuntimeAsyncPostureKind>,
    basis_posture: WorthQueryOrdinaryRuntimeBasisPostureKind,
    remask_posture: Option<WorthQueryOrdinaryRuntimeRemaskPostureKind>,
    support_evidence_identity: WorthQueryEvidenceIdentity,
    posture_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryOrdinaryRuntimePosture {
    pub fn new(
        kind: WorthQueryOrdinaryRuntimePostureKind,
        cause_posture: WorthQueryOrdinaryRuntimeCausePostureKind,
        async_posture: Option<WorthQueryOrdinaryRuntimeAsyncPostureKind>,
        basis_posture: WorthQueryOrdinaryRuntimeBasisPostureKind,
        remask_posture: Option<WorthQueryOrdinaryRuntimeRemaskPostureKind>,
        support_evidence_digest: impl Into<String>,
    ) -> Result<Self, IdentityDerivationDenial> {
        let support_evidence_digest = support_evidence_digest.into();
        let support_evidence_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_ordinary_runtime_posture_support_digest_v1",
                )
                .field_shape(
                    WorthQueryEvidenceTag::new("support_digest"),
                    &support_evidence_digest,
                )
                .seal()?;
        Ok(Self::new_with_support_identity(
            kind,
            cause_posture,
            async_posture,
            basis_posture,
            remask_posture,
            support_evidence_identity,
        ))
    }

    pub fn new_with_support_identity(
        kind: WorthQueryOrdinaryRuntimePostureKind,
        cause_posture: WorthQueryOrdinaryRuntimeCausePostureKind,
        async_posture: Option<WorthQueryOrdinaryRuntimeAsyncPostureKind>,
        basis_posture: WorthQueryOrdinaryRuntimeBasisPostureKind,
        remask_posture: Option<WorthQueryOrdinaryRuntimeRemaskPostureKind>,
        support_evidence_identity: WorthQueryEvidenceIdentity,
    ) -> Result<Self, IdentityDerivationDenial> {
        let posture_identity =
            worth_query_evidence_identity(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
                .field_shape(
                    WorthQueryEvidenceTag::new("identity_family"),
                    "worth_query_ordinary_runtime_posture_v1",
                )
                .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
                .field_shape(WorthQueryEvidenceTag::new("cause"), cause_posture.as_str())
                .optional_shape(
                    WorthQueryEvidenceTag::new("async"),
                    async_posture.map(WorthQueryOrdinaryRuntimeAsyncPostureKind::as_str),
                )
                .field_shape(WorthQueryEvidenceTag::new("basis"), basis_posture.as_str())
                .optional_shape(
                    WorthQueryEvidenceTag::new("remask"),
                    remask_posture.map(WorthQueryOrdinaryRuntimeRemaskPostureKind::as_str),
                )
                .field_evidence_identity(
                    WorthQueryEvidenceTag::new("support"),
                    &support_evidence_identity,
                )
                .seal()?;
        Ok(Self {
            kind,
            cause_posture,
            async_posture,
            basis_posture,
            remask_posture,
            support_evidence_identity,
            posture_identity,
        })
    }

    pub fn kind(&self) -> WorthQueryOrdinaryRuntimePostureKind {
        self.kind
    }

    pub fn cause_posture(&self) -> WorthQueryOrdinaryRuntimeCausePostureKind {
        self.cause_posture
    }

    pub fn async_posture(&self) -> Option<WorthQueryOrdinaryRuntimeAsyncPostureKind> {
        self.async_posture
    }

    pub fn basis_posture(&self) -> WorthQueryOrdinaryRuntimeBasisPostureKind {
        self.basis_posture
    }

    pub fn remask_posture(&self) -> Option<WorthQueryOrdinaryRuntimeRemaskPostureKind> {
        self.remask_posture
    }

    pub fn support_evidence_digest(&self) -> &str {
        self.support_evidence_identity.as_str()
    }

    pub fn support_evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.support_evidence_identity
    }

    pub fn posture_digest(&self) -> &str {
        self.posture_identity.as_str()
    }

    pub fn evidence_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.posture_identity
    }
}
