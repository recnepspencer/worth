use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};
use crate::identity::hash_parts;

use super::semantics::WorthQueryBindingTargetSemantics;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryBindingTargetKind {
    IntentDeclaration,
    AdmittedIntentPlan,
    LowerRuntimeBoundaryEnvelope,
    AdmittedDeclarationProgression,
    DeclarationRoutePlan,
    DeclarationReceipt,
    DeclarationEnvelope,
}

impl WorthQueryBindingTargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::IntentDeclaration => "intent-declaration",
            Self::AdmittedIntentPlan => "admitted-intent-plan",
            Self::LowerRuntimeBoundaryEnvelope => "lower-runtime-boundary-envelope",
            Self::AdmittedDeclarationProgression => "admitted-declaration-progression",
            Self::DeclarationRoutePlan => "declaration-route-plan",
            Self::DeclarationReceipt => "declaration-receipt",
            Self::DeclarationEnvelope => "declaration-envelope",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryBindingTarget {
    kind: WorthQueryBindingTargetKind,
    target_digest: String,
    binding_digest: String,
    semantics: WorthQueryBindingTargetSemantics,
}

impl WorthQueryBindingTarget {
    pub fn kind(&self) -> WorthQueryBindingTargetKind {
        self.kind
    }

    pub fn target_digest(&self) -> &str {
        &self.target_digest
    }

    pub fn binding_digest(&self) -> &str {
        &self.binding_digest
    }

    pub fn target_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_binding_target_target_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), self.kind.as_str())
            .field_value(WorthQueryEvidenceTag::new("target"), &self.target_digest)
            .seal()
    }

    pub fn binding_identity(&self) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
        WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::LowerRuntimeBoundaryEvidence)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "worth_query_binding_target_binding_v1",
            )
            .field_shape(WorthQueryEvidenceTag::new("kind"), self.kind.as_str())
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("target"),
                &self.target_identity()?,
            )
            .field_value(WorthQueryEvidenceTag::new("binding"), &self.binding_digest)
            .seal()
    }

    pub fn semantics(&self) -> &WorthQueryBindingTargetSemantics {
        &self.semantics
    }

    pub(crate) fn new(
        kind: WorthQueryBindingTargetKind,
        target_digest: String,
        semantics: WorthQueryBindingTargetSemantics,
    ) -> Self {
        let binding_digest = hash_parts(&[
            "worth_query_binding_target_v1".to_string(),
            format!("kind:{}", kind.as_str()),
            format!("target:{target_digest}"),
            format!("semantics:{}", semantics.binding_digest_material()),
        ]);
        Self {
            kind,
            target_digest,
            binding_digest,
            semantics,
        }
    }
}

pub(crate) mod sealed {
    pub trait Sealed {}
}

pub trait WorthQueryBindingTargetWitness: Clone + sealed::Sealed {
    fn erased_target(&self) -> &WorthQueryBindingTarget;
    fn into_erased_target(self) -> WorthQueryBindingTarget;

    fn kind(&self) -> WorthQueryBindingTargetKind {
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

    fn semantics(&self) -> &WorthQueryBindingTargetSemantics {
        self.erased_target().semantics()
    }
}
