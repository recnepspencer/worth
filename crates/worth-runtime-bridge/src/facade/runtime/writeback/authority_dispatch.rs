use super::authority_failure_mapping::{
    map_writeback_failure_class, panic_content_message, validate_writeback_receipt_contract,
};
use super::*;

impl RuntimeBridge {
    pub(super) fn dispatch_writeback_authority(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, BridgeWritebackError> {
        let authority = self.writeback_authority.as_ref().ok_or_else(|| {
            BridgeWritebackError::new(
                BridgeWritebackErrorKind::AuthorityDenied,
                "runtime has no truth writeback authority bound",
            )
        })?;
        let receipt = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            authority.execute_writeback(request.clone())
        })) {
            Ok(result) => result.map_err(|transport_error| {
                BridgeWritebackError::new(
                    BridgeWritebackErrorKind::StrategyFailed,
                    format!("truth writeback authority failed: {transport_error}"),
                )
            }),
            Err(panic_content) => Err(BridgeWritebackError::new(
                BridgeWritebackErrorKind::StrategyPanicked,
                format!(
                    "truth writeback authority panicked: {}",
                    panic_content_message(panic_content)
                ),
            )),
        }?;
        validate_writeback_receipt_contract(&request, &receipt)?;
        if receipt.outcome_class() == BridgeWritebackOutcomeClass::Rejected {
            let failure_class = receipt
                .failure_class()
                .expect("validated rejected receipts carry a failure class");
            return Err(BridgeWritebackError::new(
                map_writeback_failure_class(failure_class),
                format!(
                    "truth writeback authority rejected request `{}` with failure `{failure_class:?}`",
                    receipt.request_digest()
                ),
            ));
        }
        Ok(receipt)
    }
}
