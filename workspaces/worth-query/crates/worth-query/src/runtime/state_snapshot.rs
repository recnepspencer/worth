use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};
use crate::ordinary_outcome::WorthQueryOrdinaryRuntimePosture;

use super::{
    WorthQueryAuthorityLane, WorthQueryRuntimeAsyncResultState, WorthQueryRuntimeRemaskPosture,
};

#[non_exhaustive]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthQueryRuntimeStateKind {
    Ready,
    Remasked,
    Pending,
    Stale,
    Failed,
    Cancelled,
    Retried,
    Revalidating,
    Superseded,
    Denied,
    Unresolved,
    Unsupported,
}

impl WorthQueryRuntimeStateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ready => "ready",
            Self::Remasked => "remasked",
            Self::Pending => "pending",
            Self::Stale => "stale",
            Self::Failed => "failed",
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

impl std::fmt::Display for WorthQueryRuntimeStateKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeStateSnapshot {
    kind: WorthQueryRuntimeStateKind,
    basis_identity: WorthQueryEvidenceIdentity,
    result_shape_identity: WorthQueryEvidenceIdentity,
    authority_lane: WorthQueryAuthorityLane,
    explanation: String,
    ordinary_runtime_posture: Option<WorthQueryOrdinaryRuntimePosture>,
    async_result_state: Option<WorthQueryRuntimeAsyncResultState>,
    remask_posture: Option<WorthQueryRuntimeRemaskPosture>,
    state_digest: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeStateSnapshot {
    pub fn ready(
        basis_identity: WorthQueryEvidenceIdentity,
        result_shape_identity: WorthQueryEvidenceIdentity,
        authority_lane: WorthQueryAuthorityLane,
        explanation: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        Ok(Self::new(
            WorthQueryRuntimeStateKind::Ready,
            basis_identity,
            result_shape_identity,
            authority_lane,
            explanation,
        )?)
    }

    pub fn deferred(
        kind: WorthQueryRuntimeStateKind,
        basis_identity: WorthQueryEvidenceIdentity,
        result_shape_identity: WorthQueryEvidenceIdentity,
        authority_lane: WorthQueryAuthorityLane,
        explanation: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        assert!(
            kind != WorthQueryRuntimeStateKind::Ready,
            "ready state should use WorthQueryRuntimeStateSnapshot::ready"
        );
        Ok(Self::new(
            kind,
            basis_identity,
            result_shape_identity,
            authority_lane,
            explanation,
        )?)
    }

    pub fn with_async_result_state(
        mut self,
        async_result_state: WorthQueryRuntimeAsyncResultState,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        if self.remask_posture.is_none() {
            self.kind = async_result_state.kind().state_kind();
        }
        self.async_result_state = Some(async_result_state);
        self.state_digest = compute_state_digest(
            self.kind,
            &self.basis_identity,
            &self.result_shape_identity,
            self.authority_lane,
            &self.explanation,
            self.ordinary_runtime_posture.as_ref(),
            self.async_result_state.as_ref(),
            self.remask_posture.as_ref(),
        )?;
        Ok(self)
    }

    pub fn with_ordinary_runtime_posture(
        mut self,
        ordinary_runtime_posture: WorthQueryOrdinaryRuntimePosture,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        self.ordinary_runtime_posture = Some(ordinary_runtime_posture);
        self.state_digest = compute_state_digest(
            self.kind,
            &self.basis_identity,
            &self.result_shape_identity,
            self.authority_lane,
            &self.explanation,
            self.ordinary_runtime_posture.as_ref(),
            self.async_result_state.as_ref(),
            self.remask_posture.as_ref(),
        )?;
        Ok(self)
    }

    pub fn with_remask_posture(
        mut self,
        remask_posture: WorthQueryRuntimeRemaskPosture,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        self.kind = remask_posture.disposition_kind().state_kind();
        self.remask_posture = Some(remask_posture);
        self.state_digest = compute_state_digest(
            self.kind,
            &self.basis_identity,
            &self.result_shape_identity,
            self.authority_lane,
            &self.explanation,
            self.ordinary_runtime_posture.as_ref(),
            self.async_result_state.as_ref(),
            self.remask_posture.as_ref(),
        )?;
        Ok(self)
    }

    fn new(
        kind: WorthQueryRuntimeStateKind,
        basis_identity: WorthQueryEvidenceIdentity,
        result_shape_identity: WorthQueryEvidenceIdentity,
        authority_lane: WorthQueryAuthorityLane,
        explanation: impl Into<String>,
    ) -> Result<Self, crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial> {
        let explanation = explanation.into();
        let ordinary_runtime_posture = None;
        let async_result_state = None;
        let remask_posture = None;
        let state_digest = compute_state_digest(
            kind,
            &basis_identity,
            &result_shape_identity,
            authority_lane,
            &explanation,
            ordinary_runtime_posture.as_ref(),
            async_result_state.as_ref(),
            remask_posture.as_ref(),
        )?;
        Ok(Self {
            kind,
            basis_identity,
            result_shape_identity,
            authority_lane,
            explanation,
            ordinary_runtime_posture,
            async_result_state,
            remask_posture,
            state_digest,
        })
    }

    pub fn kind(&self) -> WorthQueryRuntimeStateKind {
        self.kind
    }

    pub fn basis_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.basis_identity
    }

    pub fn basis_for_reporting(&self) -> &str {
        self.basis_identity.as_str()
    }

    pub fn result_shape_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.result_shape_identity
    }

    pub fn result_shape_for_reporting(&self) -> &str {
        self.result_shape_identity.as_str()
    }

    pub fn authority_lane(&self) -> WorthQueryAuthorityLane {
        self.authority_lane
    }

    pub fn explanation(&self) -> &str {
        &self.explanation
    }

    pub fn ordinary_runtime_posture(&self) -> Option<&WorthQueryOrdinaryRuntimePosture> {
        self.ordinary_runtime_posture.as_ref()
    }

    pub fn async_result_state(&self) -> Option<&WorthQueryRuntimeAsyncResultState> {
        self.async_result_state.as_ref()
    }

    pub fn remask_posture(&self) -> Option<&WorthQueryRuntimeRemaskPosture> {
        self.remask_posture.as_ref()
    }

    pub fn state_digest(&self) -> &WorthQueryEvidenceIdentity {
        &self.state_digest
    }
}

fn compute_state_digest(
    kind: WorthQueryRuntimeStateKind,
    basis_identity: &WorthQueryEvidenceIdentity,
    result_shape_identity: &WorthQueryEvidenceIdentity,
    authority_lane: WorthQueryAuthorityLane,
    explanation: &str,
    ordinary_runtime_posture: Option<&WorthQueryOrdinaryRuntimePosture>,
    async_result_state: Option<&WorthQueryRuntimeAsyncResultState>,
    remask_posture: Option<&WorthQueryRuntimeRemaskPosture>,
) -> Result<
    WorthQueryEvidenceIdentity,
    crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial,
> {
    Ok(
        worth_query_evidence_identity(WorthQueryEvidenceScope::RuntimeStateSnapshot)
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis_digest"), basis_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("result_shape_digest"),
                result_shape_identity,
            )
            .field_shape(
                WorthQueryEvidenceTag::new("authority_lane"),
                authority_lane.as_str(),
            )
            .field_value(WorthQueryEvidenceTag::new("explanation"), explanation)
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("ordinary_runtime_posture"),
                ordinary_runtime_posture.map(WorthQueryOrdinaryRuntimePosture::evidence_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("async_result_state"),
                async_result_state.map(WorthQueryRuntimeAsyncResultState::result_state_identity),
            )
            .optional_evidence_identity(
                WorthQueryEvidenceTag::new("remask_posture"),
                remask_posture.map(WorthQueryRuntimeRemaskPosture::remask_identity),
            )
            .seal()?,
    )
}
