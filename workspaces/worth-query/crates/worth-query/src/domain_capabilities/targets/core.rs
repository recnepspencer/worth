use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::intent_admission::{
    WorthQueryIntentAdmissionCoveredEntrypoint, WorthQueryIntentAdmissionFamily,
};
use crate::lower_runtime_routing::{
    WorthQueryLowerRuntimeCrossingClassification, WorthQueryLowerRuntimeRouteKind,
    WorthQueryLowerRuntimeSeamKey, WorthQueryLowerRuntimeSupportPosture,
};
use crate::target_binding::{
    WorthQueryBindingTarget, WorthQueryBindingTargetKind, WorthQueryBindingTargetSemantics,
};
use crate::WorthQueryEvidenceIdentity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilityTargetKind {
    IntentDeclaration,
    AdmittedIntentPlan,
    LowerRuntimeBoundaryEnvelope,
}

impl WorthQueryDomainCapabilityTargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::IntentDeclaration => "intent-declaration",
            Self::AdmittedIntentPlan => "admitted-intent-plan",
            Self::LowerRuntimeBoundaryEnvelope => "lower-runtime-boundary-envelope",
        }
    }

    fn from_shared(kind: WorthQueryBindingTargetKind) -> Option<Self> {
        match kind {
            WorthQueryBindingTargetKind::IntentDeclaration => Some(Self::IntentDeclaration),
            WorthQueryBindingTargetKind::AdmittedIntentPlan => Some(Self::AdmittedIntentPlan),
            WorthQueryBindingTargetKind::LowerRuntimeBoundaryEnvelope => {
                Some(Self::LowerRuntimeBoundaryEnvelope)
            }
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryDomainCapabilityTargetSemantics {
    IntentDeclaration {
        name: String,
        strategy_name: String,
        strategy_version: String,
        input_contract: String,
        source_lane: crate::runtime::WorthQueryIntentSourceLane,
        target_lane: crate::runtime::WorthQueryAuthorityLane,
    },
    AdmittedIntentPlan {
        family: WorthQueryIntentAdmissionFamily,
        entrypoint: WorthQueryIntentAdmissionCoveredEntrypoint,
        request_digest: String,
        eligibility_digest: String,
        decision_digest: String,
    },
    LowerRuntimeBoundaryEnvelope {
        seam_key: WorthQueryLowerRuntimeSeamKey,
        capability_label: &'static str,
        crossing_classification: WorthQueryLowerRuntimeCrossingClassification,
        route_kind: WorthQueryLowerRuntimeRouteKind,
        support_posture: WorthQueryLowerRuntimeSupportPosture,
        envelope_digest: String,
    },
}

impl WorthQueryDomainCapabilityTargetSemantics {
    fn from_shared(shared: &WorthQueryBindingTargetSemantics) -> Option<Self> {
        match shared {
            WorthQueryBindingTargetSemantics::IntentDeclaration {
                name,
                strategy_name,
                strategy_version,
                input_contract,
                source_lane,
                target_lane,
                ..
            } => Some(Self::IntentDeclaration {
                name: name.clone(),
                strategy_name: strategy_name.clone(),
                strategy_version: strategy_version.clone(),
                input_contract: input_contract.clone(),
                source_lane: *source_lane,
                target_lane: *target_lane,
            }),
            WorthQueryBindingTargetSemantics::AdmittedIntentPlan {
                family,
                entrypoint,
                request_digest,
                eligibility_digest,
                decision_digest,
            } => Some(Self::AdmittedIntentPlan {
                family: *family,
                entrypoint: *entrypoint,
                request_digest: request_digest.clone(),
                eligibility_digest: eligibility_digest.clone(),
                decision_digest: decision_digest.clone(),
            }),
            WorthQueryBindingTargetSemantics::LowerRuntimeBoundaryEnvelope {
                seam_key,
                capability_label,
                crossing_classification,
                route_kind,
                support_posture,
                envelope_digest,
            } => Some(Self::LowerRuntimeBoundaryEnvelope {
                seam_key: *seam_key,
                capability_label,
                crossing_classification: *crossing_classification,
                route_kind: *route_kind,
                support_posture: *support_posture,
                envelope_digest: envelope_digest.clone(),
            }),
            _ => None,
        }
    }

    pub fn intent_declaration(
        &self,
    ) -> Option<(
        &str,
        &str,
        &str,
        &str,
        crate::runtime::WorthQueryIntentSourceLane,
        crate::runtime::WorthQueryAuthorityLane,
    )> {
        match self {
            Self::IntentDeclaration {
                name,
                strategy_name,
                strategy_version,
                input_contract,
                source_lane,
                target_lane,
            } => Some((
                name.as_str(),
                strategy_name.as_str(),
                strategy_version.as_str(),
                input_contract.as_str(),
                *source_lane,
                *target_lane,
            )),
            _ => None,
        }
    }

    pub fn admitted_intent_plan(
        &self,
    ) -> Option<(
        WorthQueryIntentAdmissionFamily,
        WorthQueryIntentAdmissionCoveredEntrypoint,
        &str,
        &str,
        &str,
    )> {
        match self {
            Self::AdmittedIntentPlan {
                family,
                entrypoint,
                request_digest,
                eligibility_digest,
                decision_digest,
            } => Some((
                *family,
                *entrypoint,
                request_digest.as_str(),
                eligibility_digest.as_str(),
                decision_digest.as_str(),
            )),
            _ => None,
        }
    }

    pub fn lower_runtime_boundary(
        &self,
    ) -> Option<(
        WorthQueryLowerRuntimeSeamKey,
        &'static str,
        WorthQueryLowerRuntimeCrossingClassification,
        WorthQueryLowerRuntimeRouteKind,
        WorthQueryLowerRuntimeSupportPosture,
        &str,
    )> {
        match self {
            Self::LowerRuntimeBoundaryEnvelope {
                seam_key,
                capability_label,
                crossing_classification,
                route_kind,
                support_posture,
                envelope_digest,
            } => Some((
                *seam_key,
                *capability_label,
                *crossing_classification,
                *route_kind,
                *support_posture,
                envelope_digest.as_str(),
            )),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryDomainCapabilityTarget {
    shared: WorthQueryBindingTarget,
    semantics: WorthQueryDomainCapabilityTargetSemantics,
}

impl WorthQueryDomainCapabilityTarget {
    pub(crate) fn from_shared(shared: WorthQueryBindingTarget) -> Option<Self> {
        let semantics = WorthQueryDomainCapabilityTargetSemantics::from_shared(shared.semantics())?;
        Some(Self { shared, semantics })
    }

    pub fn kind(&self) -> WorthQueryDomainCapabilityTargetKind {
        WorthQueryDomainCapabilityTargetKind::from_shared(self.shared.kind())
            .expect("domain-capability target must wrap a compatible shared target kind")
    }

    pub fn target_digest(&self) -> &str {
        self.shared.target_digest()
    }

    pub fn binding_digest(&self) -> &str {
        self.shared.binding_digest()
    }

    pub fn target_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.shared.target_identity()
    }

    pub fn binding_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.shared.binding_identity()
    }

    pub fn semantics(&self) -> &WorthQueryDomainCapabilityTargetSemantics {
        &self.semantics
    }

    pub(crate) fn shared(&self) -> &WorthQueryBindingTarget {
        &self.shared
    }

    pub(crate) fn into_shared(self) -> WorthQueryBindingTarget {
        self.shared
    }
}

pub trait WorthQueryDomainCapabilityTargetBinding: Clone {
    fn erased_target(&self) -> &WorthQueryDomainCapabilityTarget;
    fn into_erased_target(self) -> WorthQueryDomainCapabilityTarget;

    fn kind(&self) -> WorthQueryDomainCapabilityTargetKind {
        self.erased_target().kind()
    }

    fn target_digest(&self) -> &str {
        self.erased_target().target_digest()
    }

    fn binding_digest(&self) -> &str {
        self.erased_target().binding_digest()
    }

    fn target_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.erased_target().target_identity()
    }

    fn binding_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        self.erased_target().binding_identity()
    }

    fn semantics(&self) -> &WorthQueryDomainCapabilityTargetSemantics {
        self.erased_target().semantics()
    }

    fn installed_authority(
        &self,
    ) -> Option<&crate::domain_installation::WorthQueryInstalledDomainAuthority> {
        None
    }
}
