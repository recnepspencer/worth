use super::*;

pub(super) fn map_writeback_failure_class(
    failure_class: BridgeWritebackFailureClass,
) -> BridgeWritebackErrorKind {
    match failure_class {
        BridgeWritebackFailureClass::WritebackNotRequested => {
            BridgeWritebackErrorKind::WritebackNotRequested
        }
        BridgeWritebackFailureClass::PolicyRejected => BridgeWritebackErrorKind::PolicyRejected,
        BridgeWritebackFailureClass::FamilyBindingMismatch => {
            BridgeWritebackErrorKind::FamilyBindingMismatch
        }
        BridgeWritebackFailureClass::StrategyDescriptorMismatch => {
            BridgeWritebackErrorKind::StrategyDescriptorMismatch
        }
        BridgeWritebackFailureClass::IdempotenceBasisMismatch => {
            BridgeWritebackErrorKind::IdempotenceBasisMismatch
        }
        BridgeWritebackFailureClass::StaleTruthBasis => BridgeWritebackErrorKind::StaleTruthBasis,
        BridgeWritebackFailureClass::InvariantRejected => {
            BridgeWritebackErrorKind::InvariantRejected
        }
        BridgeWritebackFailureClass::StrategyFailed => BridgeWritebackErrorKind::StrategyFailed,
        BridgeWritebackFailureClass::StrategyPanicked => BridgeWritebackErrorKind::StrategyPanicked,
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
