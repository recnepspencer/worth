//! Heap custody of the complete authorization denial payload.
use super::WorthQueryOperationAuthorizationDenial;
use worth_execution::ChargedBytes;
impl ChargedBytes for WorthQueryOperationAuthorizationDenial {
    fn additional_charged_bytes(&self) -> u64 {
        // These four metadata fields are inline; new fields must be reviewed.
        let Self {
            identity: _identity,
            kind: _kind,
            causes,
            explanation_cause: _explanation_cause,
            subject,
        } = self;
        (causes.capacity() as u64)
            .saturating_mul(
                std::mem::size_of::<super::WorthQueryOperationAuthorizationDenialKind>() as u64,
            )
            .saturating_add(subject.capacity() as u64)
    }
}
