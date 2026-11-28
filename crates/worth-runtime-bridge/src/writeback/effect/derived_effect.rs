use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::identity::{BridgeIdentity, BridgeIdentityEvidence, WritebackEffectIdentityTag};
use crate::writeback::{
    AdmittedBridgeWritebackContract, BridgeWritebackEffectClass, BridgeWritebackEffectIntent,
    BridgeWritebackNativeCausalityInputs, BridgeWritebackStrategyDescriptorBasis,
};

pub type BridgeWritebackEffectIdentity = BridgeIdentity<WritebackEffectIdentityTag>;

impl BridgeWritebackEffectIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self::admit_bridge_owned(format!(
            "bridge-writeback-effect:external-authority-evidence:{}",
            evidence_identity.as_str()
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeDerivedWritebackEffect {
    effect_identity: BridgeWritebackEffectIdentity,
    contract_digest: Arc<str>,
    family_kind: crate::writeback::BridgeWritebackFamilyKind,
    effect_class: BridgeWritebackEffectClass,
    strategy_class: crate::writeback::BridgeWritebackStrategyClass,
    strategy_descriptor_basis: BridgeWritebackStrategyDescriptorBasis,
    causality_digest: Arc<str>,
    effect_intent: BridgeWritebackEffectIntent,
    effect_intent_digest: Arc<str>,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeDerivedWritebackEffect {
    pub(crate) fn new(
        effect_identity: BridgeWritebackEffectIdentity,
        contract: &AdmittedBridgeWritebackContract,
        causality: &BridgeWritebackNativeCausalityInputs,
        effect_intent: BridgeWritebackEffectIntent,
    ) -> Self {
        let declaration = contract.validated_declaration().declaration();
        let family_kind = declaration
            .family_kind()
            .expect("writeback contract admits a family");
        let strategy_class = declaration
            .strategy_class()
            .expect("writeback contract admits a strategy");
        let strategy_descriptor_basis = declaration
            .strategy_descriptor_basis()
            .expect("writeback contract admits a descriptor")
            .clone();
        let effect_class = effect_intent.effect_class();
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-derived-writeback-effect|id={}|contract={}|causality={}|intent={}",
            effect_identity.as_str(),
            contract.digest(),
            causality.digest(),
            effect_intent.digest(),
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            effect_identity,
            contract_digest: Arc::from(contract.digest()),
            family_kind,
            effect_class,
            strategy_class,
            strategy_descriptor_basis,
            causality_digest: Arc::from(causality.digest()),
            effect_intent_digest: Arc::from(effect_intent.digest()),
            effect_intent,
            canonical_basis,
            digest: Arc::from(format!("bridge-derived-writeback-effect:sha256:{digest:x}")),
        }
    }

    pub fn strategy_descriptor_digest(&self) -> &str {
        self.strategy_descriptor_basis.digest()
    }

    pub fn strategy_descriptor_basis(&self) -> &BridgeWritebackStrategyDescriptorBasis {
        &self.strategy_descriptor_basis
    }

    pub fn effect_identity(&self) -> &BridgeWritebackEffectIdentity {
        &self.effect_identity
    }

    pub fn contract_digest(&self) -> &str {
        self.contract_digest.as_ref()
    }

    pub fn effect_class(&self) -> BridgeWritebackEffectClass {
        self.effect_class
    }

    pub fn family_kind(&self) -> crate::writeback::BridgeWritebackFamilyKind {
        self.family_kind
    }

    pub fn strategy_class(&self) -> crate::writeback::BridgeWritebackStrategyClass {
        self.strategy_class
    }

    pub fn effect_intent_digest(&self) -> &str {
        self.effect_intent_digest.as_ref()
    }

    pub fn effect_intent(&self) -> &BridgeWritebackEffectIntent {
        &self.effect_intent
    }

    pub fn causality_digest(&self) -> &str {
        self.causality_digest.as_ref()
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}
