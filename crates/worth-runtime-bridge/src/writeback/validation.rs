use std::sync::Arc;

use crate::facade::BridgeRequestKind;

use super::{
    BridgeWritebackDeclaration, BridgeWritebackEffectClass, BridgeWritebackFamilyKind,
    BridgeWritebackIdempotenceClass, BridgeWritebackRequestMode, BridgeWritebackStrategyClass,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedBridgeWritebackDeclaration {
    declaration: BridgeWritebackDeclaration,
    canonical_basis: Arc<str>,
}

impl ValidatedBridgeWritebackDeclaration {
    pub(crate) fn new(
        declaration: BridgeWritebackDeclaration,
    ) -> Result<Self, crate::error::BridgeWritebackError> {
        reject_preview_writeback(&declaration)?;
        reject_read_only_family_binding(&declaration)?;
        reject_read_only_strategy_binding(&declaration)?;
        reject_read_only_strategy_class_binding(&declaration)?;
        reject_writeback_capable_missing_family_kind(&declaration)?;
        reject_writeback_capable_missing_strategy_descriptor(&declaration)?;
        reject_writeback_capable_missing_strategy_class(&declaration)?;
        reject_strategy_descriptor_basis_mismatch(&declaration)?;
        if let Some(family) = declaration.family_kind() {
            require_family_shape(&declaration, family)?;
        }
        let canonical_basis = Arc::from(format!(
            "validated-bridge-writeback-declaration|declaration={}",
            declaration.digest()
        ));

        Ok(Self {
            declaration,
            canonical_basis,
        })
    }

    pub fn declaration(&self) -> &BridgeWritebackDeclaration {
        &self.declaration
    }

    pub fn canonical_basis(&self) -> &str {
        self.canonical_basis.as_ref()
    }
}

fn reject_preview_writeback(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    if declaration.request_kind() == BridgeRequestKind::Preview
        && declaration.request_mode() == BridgeWritebackRequestMode::WritebackCapable
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::PreviewWritebackRejected,
            format!(
                "Writeback declaration `{}` cannot request truth mutation from preview request kind.",
                declaration.declaration_identity().as_str()
            ),
        ));
    }

    Ok(())
}

fn reject_read_only_family_binding(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    if declaration.request_mode() == BridgeWritebackRequestMode::ReadOnly
        && declaration.family_kind().is_some()
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::WritebackNotRequested,
            format!(
                "Read-only writeback declaration `{}` cannot bind writeback family `{:?}`.",
                declaration.declaration_identity().as_str(),
                declaration.family_kind(),
            ),
        ));
    }

    Ok(())
}

fn reject_read_only_strategy_binding(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    if declaration.request_mode() == BridgeWritebackRequestMode::ReadOnly
        && declaration.strategy_descriptor_basis().is_some()
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::WritebackNotRequested,
            format!(
                "Read-only writeback declaration `{}` cannot bind a native strategy descriptor basis `{}`.",
                declaration.declaration_identity().as_str(),
                declaration
                    .strategy_descriptor_basis()
                    .expect("read-only rejection already proved strategy descriptor basis exists")
                    .digest(),
            ),
        ));
    }

    Ok(())
}

fn reject_read_only_strategy_class_binding(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    if declaration.request_mode() == BridgeWritebackRequestMode::ReadOnly
        && declaration.strategy_class().is_some()
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::WritebackNotRequested,
            format!(
                "Read-only writeback declaration `{}` cannot bind strategy class `{:?}`.",
                declaration.declaration_identity().as_str(),
                declaration
                    .strategy_class()
                    .expect("read-only rejection already proved strategy class exists"),
            ),
        ));
    }

    Ok(())
}

fn reject_writeback_capable_missing_family_kind(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    if declaration.request_mode() == BridgeWritebackRequestMode::WritebackCapable
        && declaration.family_kind().is_none()
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::FamilyBindingMismatch,
            format!(
                "Writeback-capable declaration `{}` must bind an explicit writeback family.",
                declaration.declaration_identity().as_str(),
            ),
        ));
    }

    Ok(())
}

fn reject_writeback_capable_missing_strategy_descriptor(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    if declaration.request_mode() == BridgeWritebackRequestMode::WritebackCapable
        && declaration.strategy_descriptor_basis().is_none()
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::StrategyDescriptorMismatch,
            format!(
                "Writeback-capable declaration `{}` must bind a non-empty strategy descriptor.",
                declaration.declaration_identity().as_str(),
            ),
        ));
    }

    Ok(())
}

fn reject_writeback_capable_missing_strategy_class(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    if declaration.request_mode() == BridgeWritebackRequestMode::WritebackCapable
        && declaration.strategy_class().is_none()
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::StrategyDescriptorMismatch,
            format!(
                "Writeback-capable declaration `{}` must bind an explicit strategy class.",
                declaration.declaration_identity().as_str(),
            ),
        ));
    }

    Ok(())
}

fn reject_strategy_descriptor_basis_mismatch(
    declaration: &BridgeWritebackDeclaration,
) -> Result<(), crate::error::BridgeWritebackError> {
    let Some(strategy_descriptor_basis) = declaration.strategy_descriptor_basis() else {
        return Ok(());
    };
    if declaration.request_mode() != BridgeWritebackRequestMode::WritebackCapable {
        return Ok(());
    }
    if declaration.family_kind() != Some(strategy_descriptor_basis.family_kind())
        || declaration.effect_class() != strategy_descriptor_basis.effect_class()
        || declaration.strategy_class() != Some(strategy_descriptor_basis.strategy_class())
        || declaration.idempotence_class() != strategy_descriptor_basis.idempotence_class()
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::StrategyDescriptorMismatch,
            format!(
                "Writeback-capable declaration `{}` must bind a strategy descriptor basis matching its native family, effect, strategy, and idempotence tuple.",
                declaration.declaration_identity().as_str(),
            ),
        ));
    }

    Ok(())
}

fn require_family_shape(
    declaration: &BridgeWritebackDeclaration,
    family: BridgeWritebackFamilyKind,
) -> Result<(), crate::error::BridgeWritebackError> {
    let (effect, strategy) = match family {
        BridgeWritebackFamilyKind::ProjectedStateDiff => (
            BridgeWritebackEffectClass::ProjectedStateDiff,
            BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        ),
        BridgeWritebackFamilyKind::AspectReconciliation => (
            BridgeWritebackEffectClass::AspectReconciliation,
            BridgeWritebackStrategyClass::AspectReconciliationCommit,
        ),
    };
    if declaration.effect_class() != effect
        || declaration.strategy_class() != Some(strategy)
        || declaration.idempotence_class()
            != BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression
    {
        return Err(crate::error::BridgeWritebackError::new(
            crate::error::BridgeWritebackErrorKind::FamilyBindingMismatch,
            format!("writeback declaration is incompatible with family {family:?}"),
        ));
    }
    Ok(())
}

pub(crate) fn validate_authority_inputs(
    contract: &super::AdmittedBridgeWritebackContract,
    effect: &super::BridgeDerivedWritebackEffect,
    idempotence: &super::BridgeWritebackIdempotenceBasis,
) -> Result<(), crate::error::BridgeWritebackError> {
    use crate::error::{BridgeWritebackError, BridgeWritebackErrorKind};

    let declaration = contract.validated_declaration().declaration();
    if contract.digest() != effect.contract_digest()
        || declaration.effect_class() != effect.effect_class()
    {
        return Err(BridgeWritebackError::new(
            BridgeWritebackErrorKind::FamilyBindingMismatch,
            "writeback effect does not belong to the admitted contract",
        ));
    }
    if idempotence.idempotence_class() != declaration.idempotence_class()
        || idempotence.lowered_policy_digest() != contract.lowered_policy_digest()
        || idempotence.effect_intent_digest() != effect.effect_intent_digest()
        || idempotence.causality_digest() != effect.causality_digest()
        || idempotence.strategy_descriptor_basis() != effect.strategy_descriptor_basis()
    {
        return Err(BridgeWritebackError::new(
            BridgeWritebackErrorKind::IdempotenceBasisMismatch,
            "writeback idempotence does not match the admitted effect and policy",
        ));
    }
    Ok(())
}
