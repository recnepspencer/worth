use super::*;

pub(super) fn map_writeback_failure_class(
    failure_class: BridgeWritebackFailureClass,
) -> BridgeWritebackErrorKind {
    match failure_class {
        BridgeWritebackFailureClass::WritebackNotRequested => {
            BridgeWritebackErrorKind::WritebackNotRequested
        }
        BridgeWritebackFailureClass::PolicyRejected => BridgeWritebackErrorKind::PolicyRejected,
        BridgeWritebackFailureClass::StrategyUnavailable => {
            BridgeWritebackErrorKind::StrategyUnavailable
        }
        BridgeWritebackFailureClass::FamilyBindingMismatch => {
            BridgeWritebackErrorKind::FamilyBindingMismatch
        }
        BridgeWritebackFailureClass::StrategyDescriptorMismatch => {
            BridgeWritebackErrorKind::StrategyDescriptorMismatch
        }
        BridgeWritebackFailureClass::IdempotenceBasisMismatch => {
            BridgeWritebackErrorKind::IdempotenceBasisMismatch
        }
        BridgeWritebackFailureClass::CausalityEffectMismatch => {
            BridgeWritebackErrorKind::CausalityEffectMismatch
        }
        BridgeWritebackFailureClass::StaleTruthBasis => BridgeWritebackErrorKind::StaleTruthBasis,
        BridgeWritebackFailureClass::InvariantRejected => {
            BridgeWritebackErrorKind::InvariantRejected
        }
        BridgeWritebackFailureClass::StrategyFailed => BridgeWritebackErrorKind::StrategyFailed,
        BridgeWritebackFailureClass::StrategyPanicked => BridgeWritebackErrorKind::StrategyPanicked,
        BridgeWritebackFailureClass::ReplayMismatch => BridgeWritebackErrorKind::ReplayMismatch,
        BridgeWritebackFailureClass::AuthorityDenied => BridgeWritebackErrorKind::AuthorityDenied,
        BridgeWritebackFailureClass::PreviewWritebackRejected => {
            BridgeWritebackErrorKind::PreviewWritebackRejected
        }
    }
}

pub(super) fn panic_content_message(content: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = content.downcast_ref::<&str>() {
        (*message).to_string()
    } else if let Some(message) = content.downcast_ref::<String>() {
        message.clone()
    } else {
        "non-string panic content".to_string()
    }
}

pub(super) fn map_writeback_error_kind_to_failure_class(
    error_kind: BridgeWritebackErrorKind,
) -> BridgeWritebackFailureClass {
    match error_kind {
        BridgeWritebackErrorKind::WritebackNotRequested => {
            BridgeWritebackFailureClass::WritebackNotRequested
        }
        BridgeWritebackErrorKind::PolicyRejected => BridgeWritebackFailureClass::PolicyRejected,
        BridgeWritebackErrorKind::StrategyUnavailable => {
            BridgeWritebackFailureClass::StrategyUnavailable
        }
        BridgeWritebackErrorKind::FamilyBindingMismatch => {
            BridgeWritebackFailureClass::FamilyBindingMismatch
        }
        BridgeWritebackErrorKind::StrategyDescriptorMismatch => {
            BridgeWritebackFailureClass::StrategyDescriptorMismatch
        }
        BridgeWritebackErrorKind::IdempotenceBasisMismatch => {
            BridgeWritebackFailureClass::IdempotenceBasisMismatch
        }
        BridgeWritebackErrorKind::CausalityEffectMismatch => {
            BridgeWritebackFailureClass::CausalityEffectMismatch
        }
        BridgeWritebackErrorKind::StaleTruthBasis => BridgeWritebackFailureClass::StaleTruthBasis,
        BridgeWritebackErrorKind::InvariantRejected => {
            BridgeWritebackFailureClass::InvariantRejected
        }
        BridgeWritebackErrorKind::StrategyFailed => BridgeWritebackFailureClass::StrategyFailed,
        BridgeWritebackErrorKind::StrategyPanicked => BridgeWritebackFailureClass::StrategyPanicked,
        BridgeWritebackErrorKind::ReplayMismatch => BridgeWritebackFailureClass::ReplayMismatch,
        BridgeWritebackErrorKind::AuthorityDenied => BridgeWritebackFailureClass::AuthorityDenied,
        BridgeWritebackErrorKind::PreviewWritebackRejected => {
            BridgeWritebackFailureClass::PreviewWritebackRejected
        }
    }
}

pub(super) fn writeback_failure_digest(
    error: &BridgeWritebackError,
    contract: &AdmittedBridgeWritebackContract,
    effect: &BridgeDerivedWritebackEffect,
    idempotence: &BridgeWritebackIdempotenceBasis,
) -> std::sync::Arc<str> {
    use sha2::{Digest, Sha256};

    let canonical_basis = format!(
        "bridge-writeback-execution-failure|kind:{:?}|contract={}|effect={}|idempotence={}|message={}",
        error.kind(),
        contract.digest(),
        effect.digest(),
        idempotence.digest(),
        error
    );
    let digest = Sha256::digest(canonical_basis.as_bytes());
    std::sync::Arc::from(format!(
        "bridge-writeback-execution-failure:sha256:{digest:x}"
    ))
}

pub(super) fn validate_writeback_receipt_contract(
    request: &TruthWritebackRequest,
    receipt: &TruthWritebackReceipt,
) -> Result<(), BridgeWritebackError> {
    if receipt.request_digest() != request.digest() {
        return Err(BridgeWritebackError::new(
            BridgeWritebackErrorKind::InvariantRejected,
            format!(
                "truth writeback authority returned receipt `{}` for request `{}`",
                receipt.request_digest(),
                request.digest()
            ),
        ));
    }

    match (receipt.outcome_class(), receipt.failure_class()) {
        (BridgeWritebackOutcomeClass::Rejected, None) => Err(BridgeWritebackError::new(
            BridgeWritebackErrorKind::InvariantRejected,
            format!(
                "truth writeback authority returned rejected receipt `{}` without a failure class",
                receipt.digest()
            ),
        )),
        (BridgeWritebackOutcomeClass::CanonicalNoop, Some(_))
        | (BridgeWritebackOutcomeClass::AuthoritativeCommit, Some(_)) => {
            Err(BridgeWritebackError::new(
                BridgeWritebackErrorKind::InvariantRejected,
                format!(
                    "truth writeback authority returned non-rejected receipt `{}` with a failure class",
                    receipt.digest()
                ),
            ))
        }
        _ => Ok(()),
    }
}
