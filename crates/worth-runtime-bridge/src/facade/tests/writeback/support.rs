pub(super) use super::super::*;
pub(super) use crate::facade::{
    BridgeDerivedWritebackEffect, BridgeExecutionPolicyClass, BridgePolicyDeclaration,
    BridgePolicyDeclarationIdentity, BridgeRequestKind, BridgeRouteIdentity, BridgeRuntimePolicy,
    BridgeWritebackAuthoritativeStateBasis, BridgeWritebackCausalityIdentity,
    BridgeWritebackDeclaration, BridgeWritebackDeclarationIdentity, BridgeWritebackEffectClass,
    BridgeWritebackEffectIdentity, BridgeWritebackEffectIntent, BridgeWritebackErrorKind,
    BridgeWritebackFailureClass, BridgeWritebackFamilyKind, BridgeWritebackIdempotenceClass,
    BridgeWritebackIdempotenceIdentity, BridgeWritebackLoopDisposition,
    BridgeWritebackNativeCausalityInputs, BridgeWritebackRequestMode, BridgeWritebackStrategyClass,
    BridgeWritebackStrategyDescriptorBasis,
};
use std::sync::{Arc, RwLock};
use worth_foundational::facade::{AspectKey, AspectValue};

#[derive(Clone)]
pub(super) struct RejectingWritebackAuthority {
    pub(super) failure_class: BridgeWritebackFailureClass,
}

#[derive(Clone)]
pub(super) struct FailingWritebackAuthority;

#[derive(Clone)]
pub(super) struct PanickingWritebackAuthority;

#[derive(Clone, Default)]
pub(super) struct MismatchedReceiptWritebackAuthority {
    prior_request: Arc<RwLock<Option<crate::adapter::TruthWritebackRequest>>>,
}

#[derive(Clone)]
pub(super) struct MalformedRejectedReceiptWritebackAuthority;

#[derive(Clone)]
pub(super) struct MalformedSuccessfulReceiptWritebackAuthority;

impl crate::adapter::TruthWritebackAuthority for RejectingWritebackAuthority {
    fn execute_writeback(
        &self,
        request: crate::adapter::TruthWritebackRequest,
    ) -> Result<crate::adapter::TruthWritebackReceipt, crate::adapter::TruthWritebackAuthorityError>
    {
        Ok(
            crate::adapter::TruthWritebackReceipt::new_with_failure_class(
                crate::facade::BridgeWritebackOutcomeClass::Rejected,
                Some(self.failure_class),
                &request,
            ),
        )
    }
}

impl crate::adapter::TruthWritebackAuthority for FailingWritebackAuthority {
    fn execute_writeback(
        &self,
        _request: crate::adapter::TruthWritebackRequest,
    ) -> Result<crate::adapter::TruthWritebackReceipt, crate::adapter::TruthWritebackAuthorityError>
    {
        Err(crate::adapter::TruthWritebackAuthorityError::new(
            "writeback authority transport failure",
        ))
    }
}

impl crate::adapter::TruthWritebackAuthority for PanickingWritebackAuthority {
    fn execute_writeback(
        &self,
        _request: crate::adapter::TruthWritebackRequest,
    ) -> Result<crate::adapter::TruthWritebackReceipt, crate::adapter::TruthWritebackAuthorityError>
    {
        panic!("writeback strategy panic");
    }
}

impl crate::adapter::TruthWritebackAuthority for MismatchedReceiptWritebackAuthority {
    fn execute_writeback(
        &self,
        request: crate::adapter::TruthWritebackRequest,
    ) -> Result<crate::adapter::TruthWritebackReceipt, crate::adapter::TruthWritebackAuthorityError>
    {
        let mut prior_request = self
            .prior_request
            .write()
            .expect("mismatched receipt authority lock poisoned");
        let receipt_request = prior_request.as_ref().unwrap_or(&request);
        let receipt = crate::adapter::TruthWritebackReceipt::new(
            crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
            receipt_request,
        );
        *prior_request = Some(request);
        Ok(receipt)
    }
}

impl crate::adapter::TruthWritebackAuthority for MalformedRejectedReceiptWritebackAuthority {
    fn execute_writeback(
        &self,
        request: crate::adapter::TruthWritebackRequest,
    ) -> Result<crate::adapter::TruthWritebackReceipt, crate::adapter::TruthWritebackAuthorityError>
    {
        Ok(crate::adapter::TruthWritebackReceipt::new(
            crate::facade::BridgeWritebackOutcomeClass::Rejected,
            &request,
        ))
    }
}

impl crate::adapter::TruthWritebackAuthority for MalformedSuccessfulReceiptWritebackAuthority {
    fn execute_writeback(
        &self,
        request: crate::adapter::TruthWritebackRequest,
    ) -> Result<crate::adapter::TruthWritebackReceipt, crate::adapter::TruthWritebackAuthorityError>
    {
        Ok(
            crate::adapter::TruthWritebackReceipt::new_with_failure_class(
                crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
                Some(BridgeWritebackFailureClass::StrategyFailed),
                &request,
            ),
        )
    }
}

pub(super) fn lowered_policy(
    runtime: &RuntimeBridge,
) -> crate::facade::LoweredBridgeExecutionPolicy {
    let contract = runtime
        .admit_policy_declaration(BridgePolicyDeclaration::new(
            BridgePolicyDeclarationIdentity::admit_bridge_owned("policy:writeback"),
            BridgeRequestKind::Authoritative,
            BridgeExecutionPolicyClass::DeterministicCanonical,
            BridgeDiagnosticsTier::Standard,
            true,
            true,
        ))
        .expect("authoritative writeback policy should admit");
    runtime.lower_admitted_policy(&contract)
}

pub(super) fn writeback_declaration(
    declaration_identity: BridgeWritebackDeclarationIdentity,
    request_kind: BridgeRequestKind,
    request_mode: BridgeWritebackRequestMode,
    _strategy_descriptor_evidence_text: &str,
) -> BridgeWritebackDeclaration {
    match request_mode {
        BridgeWritebackRequestMode::ReadOnly => BridgeWritebackDeclaration::read_only(
            declaration_identity,
            request_kind,
            BridgeWritebackEffectClass::ProjectedStateDiff,
            BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
        ),
        BridgeWritebackRequestMode::WritebackCapable => {
            BridgeWritebackDeclaration::writeback_capable(
                declaration_identity,
                request_kind,
                BridgeWritebackFamilyKind::ProjectedStateDiff,
                BridgeWritebackEffectClass::ProjectedStateDiff,
                BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
                BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
            )
        }
    }
}

pub(super) fn causality_basis(
    identity: BridgeWritebackCausalityIdentity,
    truth_trigger_evidence_text: &str,
) -> BridgeWritebackNativeCausalityInputs {
    BridgeWritebackNativeCausalityInputs::new(
        identity.clone(),
        crate::truth_identity_fixtures::truth_commit_fixture(truth_trigger_evidence_text),
        BridgeRouteIdentity::admit_bridge_owned(identity.as_str()),
        crate::truth_identity_fixtures::truth_snapshot_fixture(identity.as_str()),
        crate::truth_identity_fixtures::truth_snapshot_fixture(identity.as_str()),
    )
}

pub(super) fn writeback_effect_intent(
    effect_class: BridgeWritebackEffectClass,
    marker: impl Into<String>,
) -> BridgeWritebackEffectIntent {
    let marker = marker.into();
    let aspect_key = match effect_class {
        BridgeWritebackEffectClass::ProjectedStateDiff => "bridge.writeback.projected-state-diff",
        BridgeWritebackEffectClass::AspectReconciliation => {
            "bridge.writeback.aspect-reconciliation"
        }
    };
    BridgeWritebackEffectIntent::validated_scalar_patch(
        effect_class,
        AspectKey::new(aspect_key).expect("static writeback effect aspect key is valid"),
        AspectValue::String(marker.into()),
    )
    .expect("writeback effect test intent should validate as a foundational scalar patch")
}

pub(super) fn truth_state_basis(
    effect: &BridgeDerivedWritebackEffect,
) -> BridgeWritebackAuthoritativeStateBasis {
    BridgeWritebackAuthoritativeStateBasis::from_effect(effect)
}

pub(super) fn writeback_declaration_with_shape(
    declaration_identity: BridgeWritebackDeclarationIdentity,
    request_kind: BridgeRequestKind,
    request_mode: BridgeWritebackRequestMode,
    effect_class: BridgeWritebackEffectClass,
    _strategy_descriptor_evidence_text: &str,
    idempotence_class: BridgeWritebackIdempotenceClass,
) -> BridgeWritebackDeclaration {
    match request_mode {
        BridgeWritebackRequestMode::ReadOnly => BridgeWritebackDeclaration::read_only(
            declaration_identity,
            request_kind,
            effect_class,
            idempotence_class,
        ),
        BridgeWritebackRequestMode::WritebackCapable => {
            BridgeWritebackDeclaration::writeback_capable(
                declaration_identity,
                request_kind,
                match effect_class {
                    BridgeWritebackEffectClass::ProjectedStateDiff => {
                        BridgeWritebackFamilyKind::ProjectedStateDiff
                    }
                    BridgeWritebackEffectClass::AspectReconciliation => {
                        BridgeWritebackFamilyKind::AspectReconciliation
                    }
                },
                effect_class,
                match effect_class {
                    BridgeWritebackEffectClass::ProjectedStateDiff => {
                        BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation
                    }
                    BridgeWritebackEffectClass::AspectReconciliation => {
                        BridgeWritebackStrategyClass::AspectReconciliationCommit
                    }
                },
                idempotence_class,
            )
        }
    }
}

pub(super) fn projected_strategy_descriptor_basis() -> BridgeWritebackStrategyDescriptorBasis {
    BridgeWritebackStrategyDescriptorBasis::for_writeback_contract(
        BridgeWritebackFamilyKind::ProjectedStateDiff,
        BridgeWritebackEffectClass::ProjectedStateDiff,
        BridgeWritebackStrategyClass::ProjectedStateDiffReconciliation,
        BridgeWritebackIdempotenceClass::RequireSemanticNoopSuppression,
    )
}
