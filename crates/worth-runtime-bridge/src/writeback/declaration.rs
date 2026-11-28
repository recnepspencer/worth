use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::facade::BridgeRequestKind;
use crate::identity::{BridgeIdentity, BridgeIdentityEvidence, WritebackDeclarationIdentityTag};

use super::BridgeWritebackIdempotenceClass;

pub type BridgeWritebackDeclarationIdentity = BridgeIdentity<WritebackDeclarationIdentityTag>;

impl BridgeWritebackDeclarationIdentity {
    pub fn from_bridge_evidence(evidence_identity: &BridgeIdentityEvidence) -> Self {
        Self::admit_bridge_owned(format!(
            "bridge-writeback-declaration:external-authority-evidence:{}",
            evidence_identity.as_str()
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeWritebackStrategyDescriptorBasis {
    family_kind: BridgeWritebackFamilyKind,
    effect_class: BridgeWritebackEffectClass,
    strategy_class: BridgeWritebackStrategyClass,
    idempotence_class: BridgeWritebackIdempotenceClass,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeWritebackStrategyDescriptorBasis {
    pub fn for_writeback_contract(
        family_kind: BridgeWritebackFamilyKind,
        effect_class: BridgeWritebackEffectClass,
        strategy_class: BridgeWritebackStrategyClass,
        idempotence_class: BridgeWritebackIdempotenceClass,
    ) -> Self {
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-writeback-strategy-descriptor|family:{family_kind:?}|effect:{effect_class:?}|strategy-class:{strategy_class:?}|idempotence:{idempotence_class:?}",
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {
            family_kind,
            effect_class,
            strategy_class,
            idempotence_class,
            canonical_basis,
            digest: Arc::from(format!(
                "bridge-writeback-strategy-descriptor:sha256:{digest:x}"
            )),
        }
    }

    pub fn family_kind(&self) -> BridgeWritebackFamilyKind {
        self.family_kind
    }

    pub fn effect_class(&self) -> BridgeWritebackEffectClass {
        self.effect_class
    }

    pub fn strategy_class(&self) -> BridgeWritebackStrategyClass {
        self.strategy_class
    }

    pub fn idempotence_class(&self) -> BridgeWritebackIdempotenceClass {
        self.idempotence_class
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BridgeWritebackDeclaration {
    declaration_identity: BridgeWritebackDeclarationIdentity,
    request_kind: BridgeRequestKind,
    request_mode: BridgeWritebackRequestMode,
    family_kind: Option<BridgeWritebackFamilyKind>,
    effect_class: BridgeWritebackEffectClass,
    strategy_class: Option<BridgeWritebackStrategyClass>,
    strategy_descriptor_basis: Option<BridgeWritebackStrategyDescriptorBasis>,
    idempotence_class: BridgeWritebackIdempotenceClass,
    canonical_basis: Arc<str>,
    digest: Arc<str>,
}

impl BridgeWritebackDeclaration {
    pub(crate) fn new(
        declaration_identity: BridgeWritebackDeclarationIdentity,
        request_kind: BridgeRequestKind,
        request_mode: BridgeWritebackRequestMode,
        family_kind: Option<BridgeWritebackFamilyKind>,
        effect_class: BridgeWritebackEffectClass,
        strategy_class: Option<BridgeWritebackStrategyClass>,
        strategy_descriptor_basis: Option<BridgeWritebackStrategyDescriptorBasis>,
        idempotence_class: BridgeWritebackIdempotenceClass,
    ) -> Self {
        let strategy_descriptor_basis_digest = strategy_descriptor_basis
            .as_ref()
            .map(|basis| basis.digest())
            .unwrap_or("");
        let canonical_basis = Arc::<str>::from(format!(
            "bridge-writeback-declaration|id={}|request-kind:{request_kind:?}|request-mode:{request_mode:?}|family:{}|effect:{effect_class:?}|strategy-class:{}|strategy={}|idempotence:{idempotence_class:?}",
            declaration_identity.as_str(),
            family_kind
                .map(|kind| format!("{kind:?}"))
                .unwrap_or_else(|| "none".to_string()),
            strategy_class
                .map(|class| format!("{class:?}"))
                .unwrap_or_else(|| "none".to_string()),
            strategy_descriptor_basis_digest,
        ));
        let digest = Sha256::digest(canonical_basis.as_bytes());
        Self {
            declaration_identity,
            request_kind,
            request_mode,
            family_kind,
            effect_class,
            strategy_class,
            strategy_descriptor_basis,
            idempotence_class,
            canonical_basis,
            digest: Arc::from(format!("bridge-writeback-declaration:sha256:{digest:x}")),
        }
    }

    pub fn read_only(
        declaration_identity: BridgeWritebackDeclarationIdentity,
        request_kind: BridgeRequestKind,
        effect_class: BridgeWritebackEffectClass,
        idempotence_class: BridgeWritebackIdempotenceClass,
    ) -> Self {
        Self::new(
            declaration_identity,
            request_kind,
            BridgeWritebackRequestMode::ReadOnly,
            None,
            effect_class,
            None,
            None,
            idempotence_class,
        )
    }

    pub fn writeback_capable(
        declaration_identity: BridgeWritebackDeclarationIdentity,
        request_kind: BridgeRequestKind,
        family_kind: BridgeWritebackFamilyKind,
        effect_class: BridgeWritebackEffectClass,
        strategy_class: BridgeWritebackStrategyClass,
        idempotence_class: BridgeWritebackIdempotenceClass,
    ) -> Self {
        let strategy_descriptor_basis =
            BridgeWritebackStrategyDescriptorBasis::for_writeback_contract(
                family_kind,
                effect_class,
                strategy_class,
                idempotence_class,
            );
        Self::new(
            declaration_identity,
            request_kind,
            BridgeWritebackRequestMode::WritebackCapable,
            Some(family_kind),
            effect_class,
            Some(strategy_class),
            Some(strategy_descriptor_basis),
            idempotence_class,
        )
    }

    pub fn declaration_identity(&self) -> &BridgeWritebackDeclarationIdentity {
        &self.declaration_identity
    }

    pub fn request_kind(&self) -> BridgeRequestKind {
        self.request_kind
    }

    pub fn request_mode(&self) -> BridgeWritebackRequestMode {
        self.request_mode
    }

    pub fn effect_class(&self) -> BridgeWritebackEffectClass {
        self.effect_class
    }

    pub fn family_kind(&self) -> Option<BridgeWritebackFamilyKind> {
        self.family_kind
    }

    pub fn strategy_class(&self) -> Option<BridgeWritebackStrategyClass> {
        self.strategy_class
    }

    pub fn strategy_descriptor_basis(&self) -> Option<&BridgeWritebackStrategyDescriptorBasis> {
        self.strategy_descriptor_basis.as_ref()
    }

    pub fn strategy_descriptor_digest(&self) -> &str {
        self.strategy_descriptor_basis
            .as_ref()
            .map(|basis| basis.digest())
            .unwrap_or("")
    }

    pub fn idempotence_class(&self) -> BridgeWritebackIdempotenceClass {
        self.idempotence_class
    }

    pub const fn declaration_field_count(&self) -> usize {
        7
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }

    pub fn digest(&self) -> &str {
        self.digest.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackFamilyKind {
    ProjectedStateDiff,
    AspectReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackRequestMode {
    ReadOnly,
    WritebackCapable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackEffectClass {
    ProjectedStateDiff,
    AspectReconciliation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BridgeWritebackStrategyClass {
    ProjectedStateDiffReconciliation,
    AspectReconciliationCommit,
}
