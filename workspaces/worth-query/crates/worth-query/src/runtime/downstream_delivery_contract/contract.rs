use super::*;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeDownstreamDeliveryClass {
    TruthPatch,
    TimeOnly,
    AsyncBacked,
    MixedCause,
}

impl WorthQueryRuntimeDownstreamDeliveryClass {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::TruthPatch => "truth-patch",
            Self::TimeOnly => "time-only",
            Self::AsyncBacked => "async-backed",
            Self::MixedCause => "mixed-cause",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryRuntimeDownstreamSupportPosture {
    Supported,
    Remasked,
    Denied,
}

impl WorthQueryRuntimeDownstreamSupportPosture {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Supported => "supported",
            Self::Remasked => "remasked",
            Self::Denied => "denied",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryRuntimeDownstreamDeliveryContract {
    backend_posture: WorthQueryRuntimeBackendPosture,
    runtime_resume_support_status: WorthQueryLowerRuntimeSupportPosture,
    runtime_resume_support_identity: WorthQueryEvidenceIdentity,
    durable_resume_support_status: WorthQueryLowerRuntimeSupportPosture,
    durable_resume_support_identity: WorthQueryEvidenceIdentity,
    contract_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryRuntimeDownstreamDeliveryContract {
    pub fn from_support_profile(
        profile: &WorthQueryRuntimeSupportProfile,
    ) -> Result<Self, IdentityDerivationDenial> {
        Self::from_backend_posture(profile.posture())
    }

    pub fn from_backend_posture(
        backend_posture: WorthQueryRuntimeBackendPosture,
    ) -> Result<Self, IdentityDerivationDenial> {
        let support = worth_query_lower_runtime_support_matrix();
        let runtime_resume = support
            .support_for(WorthQueryLowerRuntimeSeamKey::BasisReadmissionFromSubscriptionEvidence)
            .expect("subscription basis readmission support row must exist");
        let durable_resume_rows = [
            WorthQueryLowerRuntimeSeamKey::DurableRouteReplayNeighbor,
            WorthQueryLowerRuntimeSeamKey::PersistedBoundaryExecutionReceiptNeighbor,
            WorthQueryLowerRuntimeSeamKey::RestartStableBoundaryEnvelopeReloadNeighbor,
        ]
        .map(|seam| {
            support
                .support_for(seam)
                .expect("durable resume debt row must exist")
        });
        let durable_resume_support_status =
            aggregate_support_posture(durable_resume_rows.iter().map(|row| row.posture()));
        let runtime_resume_support_identity = lower_runtime_support_row_identity(runtime_resume)?;
        let durable_resume_support_identity =
            lower_runtime_support_rows_aggregate_identity(durable_resume_rows)?;
        let runtime_resume_support_status = runtime_resume.posture();
        let contract_identity = runtime_downstream_delivery_contract_identity(
            backend_posture,
            runtime_resume_support_status,
            &runtime_resume_support_identity,
            durable_resume_support_status,
            &durable_resume_support_identity,
        );
        Ok(Self {
            backend_posture,
            runtime_resume_support_status,
            runtime_resume_support_identity,
            durable_resume_support_status,
            durable_resume_support_identity,
            contract_identity,
        })
    }

    pub fn backend_posture(&self) -> WorthQueryRuntimeBackendPosture {
        self.backend_posture
    }

    pub fn runtime_backed_resume_supported(&self) -> bool {
        self.runtime_resume_support_status == WorthQueryLowerRuntimeSupportPosture::Admitted
    }

    pub fn runtime_resume_support_posture(&self) -> WorthQueryLowerRuntimeSupportPosture {
        self.runtime_resume_support_status
    }

    pub fn runtime_resume_support_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.runtime_resume_support_identity
    }

    pub fn runtime_resume_support_for_reporting(&self) -> &str {
        self.runtime_resume_support_identity.as_str()
    }

    pub fn durable_resume_deferred(&self) -> bool {
        self.durable_resume_support_status == WorthQueryLowerRuntimeSupportPosture::Deferred
    }

    pub fn durable_resume_support_posture(&self) -> WorthQueryLowerRuntimeSupportPosture {
        self.durable_resume_support_status
    }

    pub fn durable_resume_support_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.durable_resume_support_identity
    }

    pub fn durable_resume_support_for_reporting(&self) -> &str {
        self.durable_resume_support_identity.as_str()
    }

    pub fn contract_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.contract_identity
    }

    pub fn contract_for_reporting(&self) -> &str {
        self.contract_identity.as_str()
    }
}
