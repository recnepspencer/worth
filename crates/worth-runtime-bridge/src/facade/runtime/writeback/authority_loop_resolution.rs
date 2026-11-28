use super::*;

pub(super) fn resolve_terminal_writeback_loop_disposition(
    idempotence: &BridgeWritebackIdempotenceBasis,
    loop_prevention: &BridgeWritebackLoopPreventionReport,
) -> Result<Option<BridgeWritebackAuthorityOutcome>, BridgeWritebackError> {
    match loop_prevention.disposition() {
        BridgeWritebackLoopDisposition::AllowAuthoritativeAttempt => Ok(None),
        BridgeWritebackLoopDisposition::CanonicalNoop => Ok(Some(
            BridgeWritebackAuthorityOutcome::canonical_noop(idempotence),
        )),
        BridgeWritebackLoopDisposition::RejectAsUnsafeFeedback => Err(BridgeWritebackError::new(
            BridgeWritebackErrorKind::InvariantRejected,
            format!(
                "unsafe bridge feedback suppressed before authority execution: {}",
                loop_prevention.digest()
            ),
        )),
    }
}
