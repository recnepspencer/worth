use super::{
    WorthQueryConsumerSupportAdmissionCounters, WorthQueryConsumerSupportDimension,
    WorthQueryConsumerSupportPosture,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    worth_query_evidence_identity, WorthQueryEvidenceIdentity, WorthQueryEvidenceScope,
    WorthQueryEvidenceTag,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryConsumerSupportCompatibilityDenial {
    dimension: WorthQueryConsumerSupportDimension,
    runtime_posture: WorthQueryConsumerSupportPosture,
    counters: WorthQueryConsumerSupportAdmissionCounters,
    evidence_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
}

impl WorthQueryConsumerSupportCompatibilityDenial {
    pub(crate) fn new(
        dimension: WorthQueryConsumerSupportDimension,
        runtime_posture: WorthQueryConsumerSupportPosture,
        counters: WorthQueryConsumerSupportAdmissionCounters,
    ) -> Self {
        let evidence_identity = worth_query_evidence_identity(
            WorthQueryEvidenceScope::ConsumerProjectionContractDenial,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "worth_query_consumer_support_compatibility_denial_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("dimension"), dimension.as_str())
        .field_shape(
            WorthQueryEvidenceTag::new("runtime_posture"),
            runtime_posture.as_str(),
        )
        .seal();
        Self {
            dimension,
            runtime_posture,
            counters,
            evidence_identity,
        }
    }

    pub fn dimension(&self) -> WorthQueryConsumerSupportDimension {
        self.dimension
    }

    pub fn runtime_posture(&self) -> WorthQueryConsumerSupportPosture {
        self.runtime_posture
    }

    pub fn counters(&self) -> WorthQueryConsumerSupportAdmissionCounters {
        self.counters
    }

    pub fn evidence_identity(
        &self,
    ) -> Result<&WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.evidence_identity.as_ref().map_err(|cause| *cause)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryConsumerProjectionContractDenial {
    StaleInstallationGeneration {
        counters: WorthQueryConsumerSupportAdmissionCounters,
    },
    AlreadyMinted {
        counters: WorthQueryConsumerSupportAdmissionCounters,
    },
    Compatibility(WorthQueryConsumerSupportCompatibilityDenial),
}

impl WorthQueryConsumerProjectionContractDenial {
    pub fn counters(&self) -> WorthQueryConsumerSupportAdmissionCounters {
        match self {
            Self::StaleInstallationGeneration { counters } | Self::AlreadyMinted { counters } => {
                *counters
            }
            Self::Compatibility(denial) => denial.counters(),
        }
    }
}

impl From<WorthQueryConsumerSupportCompatibilityDenial>
    for WorthQueryConsumerProjectionContractDenial
{
    fn from(denial: WorthQueryConsumerSupportCompatibilityDenial) -> Self {
        Self::Compatibility(denial)
    }
}
