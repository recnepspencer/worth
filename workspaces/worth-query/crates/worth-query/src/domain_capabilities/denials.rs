use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use worth_proof::TransitionOutcome;

use super::targets::WorthQueryDomainCapabilityTargetKind;
use crate::domain_capabilities::identity::domain_capability_scope_encoder;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilityProgressionDenialKind {
    EmptySemanticCode,
    EmptyDetail,
    StaleInstallationGeneration,
    ContributionCategoryNotInstalled,
    UnsupportedCanonicalMaterializationPosture,
    MissingCanonicalMaterializationSemantics,
    InconsistentCanonicalMaterializationSemantics,
}

impl WorthQueryDomainCapabilityProgressionDenialKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::EmptySemanticCode => "empty-semantic-code",
            Self::EmptyDetail => "empty-detail",
            Self::StaleInstallationGeneration => "stale-installation-generation",
            Self::ContributionCategoryNotInstalled => "contribution-category-not-installed",
            Self::UnsupportedCanonicalMaterializationPosture => {
                "unsupported-canonical-materialization-posture"
            }
            Self::MissingCanonicalMaterializationSemantics => {
                "missing-canonical-materialization-semantics"
            }
            Self::InconsistentCanonicalMaterializationSemantics => {
                "inconsistent-canonical-materialization-semantics"
            }
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityProgressionDenial {
    kind: WorthQueryDomainCapabilityProgressionDenialKind,
    category: &'static str,
    target_kind: WorthQueryDomainCapabilityTargetKind,
    request_identity: WorthQueryEvidenceIdentity,
    message: String,
    failure_identity: Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial>,
}

impl WorthQueryDomainCapabilityProgressionDenial {
    pub(crate) fn new(
        kind: WorthQueryDomainCapabilityProgressionDenialKind,
        category: &'static str,
        target_kind: WorthQueryDomainCapabilityTargetKind,
        request_identity: WorthQueryEvidenceIdentity,
        message: impl Into<String>,
    ) -> Self {
        let message = message.into();
        let failure_identity = (|| {
            Ok(compose_progression_denial_failure_identity(
                kind,
                category,
                target_kind,
                &request_identity,
            )?)
        })();
        Self {
            kind,
            category,
            target_kind,
            request_identity,
            message,
            failure_identity,
        }
    }

    pub fn kind(&self) -> WorthQueryDomainCapabilityProgressionDenialKind {
        self.kind
    }

    pub fn category(&self) -> &str {
        self.category
    }

    pub fn target_kind(&self) -> WorthQueryDomainCapabilityTargetKind {
        self.target_kind
    }

    pub fn request_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.request_identity
    }

    pub fn request_for_reporting(&self) -> &str {
        self.request_identity.as_str()
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn failure_identity(
        &self,
    ) -> Result<&WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        Ok(self.failure_identity.as_ref().map_err(|cause| *cause)?)
    }

    pub fn failure_for_reporting(&self) -> Result<&str, IdentityDerivationDenial> {
        Ok(self
            .failure_identity
            .as_ref()
            .map_err(|cause| *cause)?
            .as_str())
    }
}

fn compose_progression_denial_failure_identity(
    kind: WorthQueryDomainCapabilityProgressionDenialKind,
    category: &'static str,
    target_kind: WorthQueryDomainCapabilityTargetKind,
    request_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    Ok(
        domain_capability_scope_encoder("worth_query_domain_capability_progression_denial_v1")
            .field_shape(WorthQueryEvidenceTag::new("kind"), kind.as_str())
            .field_shape(WorthQueryEvidenceTag::new("category"), category)
            .field_shape(
                WorthQueryEvidenceTag::new("target_kind"),
                target_kind.as_str(),
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("request"), request_identity)
            .seal()?,
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityProgressionFailure {
    message: String,
}

impl WorthQueryDomainCapabilityProgressionFailure {
    pub fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityStale {
    category: &'static str,
    bound_target_identity: WorthQueryEvidenceIdentity,
    current_target_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryDomainCapabilityStale {
    pub(crate) fn new(
        category: &'static str,
        bound_target_identity: WorthQueryEvidenceIdentity,
        current_target_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self {
            category,
            bound_target_identity,
            current_target_identity,
        }
    }

    pub fn category(&self) -> &str {
        self.category
    }

    pub fn bound_target_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bound_target_identity
    }

    pub fn current_target_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.current_target_identity
    }

    pub fn bound_target_for_reporting(&self) -> &str {
        self.bound_target_identity.as_str()
    }

    pub fn current_target_for_reporting(&self) -> &str {
        self.current_target_identity.as_str()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityRebindRequired {
    category: &'static str,
    bound_target_identity: WorthQueryEvidenceIdentity,
    current_target_identity: WorthQueryEvidenceIdentity,
}

impl WorthQueryDomainCapabilityRebindRequired {
    pub(crate) fn new(
        category: &'static str,
        bound_target_identity: WorthQueryEvidenceIdentity,
        current_target_identity: WorthQueryEvidenceIdentity,
    ) -> Self {
        Self {
            category,
            bound_target_identity,
            current_target_identity,
        }
    }

    pub fn category(&self) -> &str {
        self.category
    }

    pub fn bound_target_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.bound_target_identity
    }

    pub fn current_target_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.current_target_identity
    }

    pub fn bound_target_for_reporting(&self) -> &str {
        self.bound_target_identity.as_str()
    }

    pub fn current_target_for_reporting(&self) -> &str {
        self.current_target_identity.as_str()
    }
}

pub type WorthQueryDomainCapabilityTransitionOutcome<S> = TransitionOutcome<
    S,
    WorthQueryDomainCapabilityProgressionDenial,
    std::convert::Infallible,
    WorthQueryDomainCapabilityStale,
    WorthQueryDomainCapabilityRebindRequired,
    WorthQueryDomainCapabilityProgressionFailure,
>;
