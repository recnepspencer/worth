use crate::application::WorthQueryCapabilityFamily;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreparedContinuationBasisKind {
    Current,
    Historical,
    PreviewDerived,
}

impl WorthQueryPreparedContinuationBasisKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Current => "current",
            Self::Historical => "historical",
            Self::PreviewDerived => "preview_derived",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreparedContinuationFreshnessPosture {
    Stable,
    Stale,
}

impl WorthQueryPreparedContinuationFreshnessPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Stale => "stale",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreparedContinuationDriftKind {
    AsyncRequest,
    Replay,
    Remask,
    PreviewCrossedResidue,
    StaleCompletion,
}

impl WorthQueryPreparedContinuationDriftKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AsyncRequest => "async_request",
            Self::Replay => "replay",
            Self::Remask => "remask",
            Self::PreviewCrossedResidue => "preview_crossed_residue",
            Self::StaleCompletion => "stale_completion",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPreparedContinuationAuthorityWitness {
    Runtime,
    RuntimeBridgeFacade,
    RelationalFacade,
    SignalFacade,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreparedContinuationBasisWitness {
    kind: WorthQueryPreparedContinuationBasisKind,
    basis_identity: WorthQueryEvidenceIdentity,
    expected_lower_runtime_binding_identity: Option<WorthQueryEvidenceIdentity>,
    source_basis_identity: Option<WorthQueryEvidenceIdentity>,
}

impl WorthQueryPreparedContinuationBasisWitness {
    pub(crate) fn new(
        kind: WorthQueryPreparedContinuationBasisKind,
        basis_identity: WorthQueryEvidenceIdentity,
        expected_lower_runtime_binding_identity: Option<WorthQueryEvidenceIdentity>,
        source_basis_identity: Option<WorthQueryEvidenceIdentity>,
    ) -> Self {
        Self {
            kind,
            basis_identity,
            expected_lower_runtime_binding_identity,
            source_basis_identity,
        }
    }

    pub fn kind(&self) -> WorthQueryPreparedContinuationBasisKind {
        self.kind
    }

    pub fn basis_identity_digest(&self) -> &str {
        self.basis_identity.as_str()
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn expected_lower_runtime_binding_digest(&self) -> Option<&str> {
        self.expected_lower_runtime_binding_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn expected_lower_runtime_binding_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.expected_lower_runtime_binding_identity.as_ref()
    }

    pub fn source_basis_identity_digest(&self) -> Option<&str> {
        self.source_basis_identity
            .as_ref()
            .map(WorthQueryEvidenceIdentity::as_str)
    }

    pub fn source_basis_identity(&self) -> Option<&WorthQueryEvidenceIdentity> {
        self.source_basis_identity.as_ref()
    }
}

pub(crate) fn continuation_readmission_basis_identity(
    basis_kind: WorthQueryPreparedContinuationBasisKind,
    identity: impl AsRef<str>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::ContinuationReadmissionBasis)
        .field_shape(
            WorthQueryEvidenceTag::new("basis_kind"),
            basis_kind.as_str(),
        )
        .field_value(WorthQueryEvidenceTag::new("basis_identity"), identity)
        .seal()
}

pub(crate) fn continuation_readmission_lower_runtime_binding_identity(
    identity: impl AsRef<str>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(
        WorthQueryEvidenceScope::ContinuationReadmissionLowerRuntimeBinding,
    )
    .field_value(
        WorthQueryEvidenceTag::new("lower_runtime_binding"),
        identity,
    )
    .seal()
}

pub(crate) fn continuation_readmission_source_basis_identity(
    identity: impl AsRef<str>,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    worth_query_evidence_identity(WorthQueryEvidenceScope::ContinuationReadmissionSourceBasis)
        .field_value(WorthQueryEvidenceTag::new("source_basis"), identity)
        .seal()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPreparedContinuationExecutionReadmission {
    basis_witness: WorthQueryPreparedContinuationBasisWitness,
    authority_witness: WorthQueryPreparedContinuationAuthorityWitness,
    freshness_posture: WorthQueryPreparedContinuationFreshnessPosture,
    drift_kind: Option<WorthQueryPreparedContinuationDriftKind>,
    required_capability_families: Vec<WorthQueryCapabilityFamily>,
}

impl WorthQueryPreparedContinuationExecutionReadmission {
    pub(crate) fn new(
        basis_witness: WorthQueryPreparedContinuationBasisWitness,
        authority_witness: WorthQueryPreparedContinuationAuthorityWitness,
        freshness_posture: WorthQueryPreparedContinuationFreshnessPosture,
        drift_kind: Option<WorthQueryPreparedContinuationDriftKind>,
        required_capability_families: Vec<WorthQueryCapabilityFamily>,
    ) -> Self {
        Self {
            basis_witness,
            authority_witness,
            freshness_posture,
            drift_kind,
            required_capability_families,
        }
    }

    pub fn basis_witness(&self) -> &WorthQueryPreparedContinuationBasisWitness {
        &self.basis_witness
    }

    pub fn authority_witness(&self) -> WorthQueryPreparedContinuationAuthorityWitness {
        self.authority_witness
    }

    pub fn freshness_posture(&self) -> WorthQueryPreparedContinuationFreshnessPosture {
        self.freshness_posture
    }

    pub fn drift_kind(&self) -> Option<WorthQueryPreparedContinuationDriftKind> {
        self.drift_kind
    }

    pub fn required_capability_families(&self) -> &[WorthQueryCapabilityFamily] {
        &self.required_capability_families
    }
}
