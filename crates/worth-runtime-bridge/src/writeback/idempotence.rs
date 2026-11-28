use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, BridgeIdentityEvidence, WritebackIdempotenceIdentityTag};

use super::{BridgeDerivedWritebackEffect, BridgeWritebackStrategyDescriptorBasis};

pub type BridgeWritebackIdempotenceIdentity = BridgeIdentity<WritebackIdempotenceIdentityTag>;

impl BridgeWritebackIdempotenceIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self::admit_bridge_owned(format!(
            "bridge-writeback-idempotence:external-authority-evidence:{}",
            evidence_identity.as_str()
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeWritebackAuthoritativeStateBasis {
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeWritebackAuthoritativeStateBasis {
    pub fn from_effect(effect: &BridgeDerivedWritebackEffect) -> Self {
        Self::from_native_basis(format!(
            "bridge-writeback-authoritative-state-basis|effect-intent={}|causality={}|family:{:?}|strategy-class:{:?}|strategy={}",
            effect.effect_intent_digest(),
            effect.causality_digest(),
            effect.family_kind(),
            effect.strategy_class(),
            effect.strategy_descriptor_basis().digest(),
        ))
    }

    pub fn from_effect_intent_and_causality(
        effect_intent: &super::BridgeWritebackEffectIntent,
        causality: &super::BridgeWritebackNativeCausalityInputs,
    ) -> Self {
        Self::from_native_basis(format!(
            "bridge-writeback-authoritative-state-basis|effect-intent={}|effect-intent-basis={}|causality={}",
            effect_intent.digest(),
            effect_intent.patch_canonical_basis(),
            causality.digest(),
        ))
    }

    fn from_native_basis(canonical_basis: String) -> Self {
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            canonical_basis: Arc::from(canonical_basis),
            digest: Arc::from(format!(
                "bridge-writeback-authoritative-state-basis:sha256:{digest:x}"
            )),
        }
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeWritebackIdempotenceBasis {
    idempotence_identity: BridgeWritebackIdempotenceIdentity,
    effect_intent_digest: Arc<str>,
    causality_digest: Arc<str>,
    lowered_policy_digest: Arc<str>,
    family_kind: crate::writeback::BridgeWritebackFamilyKind,
    strategy_class: crate::writeback::BridgeWritebackStrategyClass,
    strategy_descriptor_basis: BridgeWritebackStrategyDescriptorBasis,
    authoritative_state_digest: Arc<str>,
    idempotence_class: BridgeWritebackIdempotenceClass,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeWritebackIdempotenceBasis {
    pub fn new(
        idempotence_identity: BridgeWritebackIdempotenceIdentity,
        effect: &BridgeDerivedWritebackEffect,
        lowered_policy_digest: impl Into<Arc<str>>,
        authoritative_state_basis: &BridgeWritebackAuthoritativeStateBasis,
        idempotence_class: BridgeWritebackIdempotenceClass,
    ) -> Self {
        let lowered_policy_digest = lowered_policy_digest.into();
        let authoritative_state_digest: Arc<str> =
            Arc::from(authoritative_state_basis.digest().to_owned());
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-writeback-idempotence-basis|id={}|effect-intent={}|causality={}|lowered-policy={}|family:{:?}|strategy-class:{:?}|strategy={}|authoritative-state={}|class:{idempotence_class:?}",
            idempotence_identity.as_str(),
            effect.effect_intent_digest(),
            effect.causality_digest(),
            lowered_policy_digest.as_ref(),
            effect.family_kind(),
            effect.strategy_class(),
            effect.strategy_descriptor_basis().digest(),
            authoritative_state_digest.as_ref(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            idempotence_identity,
            effect_intent_digest: Arc::from(effect.effect_intent_digest().to_owned()),
            causality_digest: Arc::from(effect.causality_digest().to_owned()),
            lowered_policy_digest,
            family_kind: effect.family_kind(),
            strategy_class: effect.strategy_class(),
            strategy_descriptor_basis: effect.strategy_descriptor_basis().clone(),
            authoritative_state_digest,
            idempotence_class,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-writeback-idempotence-basis:sha256:{digest:x}"
            )),
        }
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }

    pub fn idempotence_identity(&self) -> &BridgeWritebackIdempotenceIdentity {
        &self.idempotence_identity
    }

    pub fn effect_intent_digest(&self) -> &str {
        self.effect_intent_digest.as_ref()
    }

    pub fn causality_digest(&self) -> &str {
        self.causality_digest.as_ref()
    }

    pub fn lowered_policy_digest(&self) -> &str {
        self.lowered_policy_digest.as_ref()
    }

    pub fn strategy_class(&self) -> crate::writeback::BridgeWritebackStrategyClass {
        self.strategy_class
    }

    pub fn family_kind(&self) -> crate::writeback::BridgeWritebackFamilyKind {
        self.family_kind
    }

    pub fn strategy_descriptor_basis(&self) -> &BridgeWritebackStrategyDescriptorBasis {
        &self.strategy_descriptor_basis
    }

    pub fn strategy_descriptor_digest(&self) -> &str {
        self.strategy_descriptor_basis.digest()
    }

    pub fn authoritative_state_digest(&self) -> &str {
        self.authoritative_state_digest.as_ref()
    }

    pub fn idempotence_class(&self) -> BridgeWritebackIdempotenceClass {
        self.idempotence_class
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackIdempotenceClass {
    RequireSemanticNoopSuppression,
    AllowRepeatedAuthorityAttempt,
}
