use super::authority_loop_resolution::resolve_terminal_writeback_loop_disposition;
use super::*;

impl RuntimeBridge {
    /// Executes writeback authority without supplying upstream feedback context.
    pub fn execute_writeback_authority(
        &self,
        contract: &AdmittedBridgeWritebackContract,
        effect: &BridgeDerivedWritebackEffect,
        idempotence: &BridgeWritebackIdempotenceBasis,
    ) -> Result<(BridgeWritebackAuthorityOutcome, TruthWritebackReceipt), BridgeWritebackError>
    {
        let (_, outcome, receipt) = self.execute_writeback_authority_with_feedback_context(
            contract,
            effect,
            idempotence,
            None,
        )?;
        Ok((
            outcome,
            receipt.expect("authority attempt must return a receipt"),
        ))
    }

    /// Classifies incoming feedback before dispatching an admitted mutation.
    pub fn execute_writeback_authority_with_feedback_context(
        &self,
        contract: &AdmittedBridgeWritebackContract,
        effect: &BridgeDerivedWritebackEffect,
        idempotence: &BridgeWritebackIdempotenceBasis,
        incoming_feedback_context: Option<&BridgeWritebackFeedbackContext>,
    ) -> Result<
        (
            BridgeWritebackLoopPreventionReport,
            BridgeWritebackAuthorityOutcome,
            Option<TruthWritebackReceipt>,
        ),
        BridgeWritebackError,
    > {
        let loop_prevention =
            self.classify_writeback_loop_prevention(effect, idempotence, incoming_feedback_context);
        if let Some(outcome) =
            resolve_terminal_writeback_loop_disposition(idempotence, &loop_prevention)?
        {
            return Ok((loop_prevention, outcome, None));
        }

        crate::writeback::validate_authority_inputs(contract, effect, idempotence)?;
        let request = TruthWritebackRequest::from_evidence(contract, effect, &loop_prevention);
        let receipt = self.dispatch_writeback_authority(request)?;
        let outcome = match receipt.outcome_class() {
            BridgeWritebackOutcomeClass::CanonicalNoop => {
                BridgeWritebackAuthorityOutcome::canonical_noop(idempotence)
            }
            BridgeWritebackOutcomeClass::AuthoritativeCommit => {
                BridgeWritebackAuthorityOutcome::authoritative_commit(idempotence, &receipt)
            }
            BridgeWritebackOutcomeClass::Rejected => {
                unreachable!("dispatch rejects failed receipts")
            }
        };
        Ok((loop_prevention, outcome, Some(receipt)))
    }
}
