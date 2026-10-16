//! Heap custody of the complete authorization denial payload.
use super::WorthQueryOperationAuthorizationDenial;
use worth_execution::ChargedBytes;
impl ChargedBytes for WorthQueryOperationAuthorizationDenial {
    fn additional_charged_bytes(&self) -> u64 {
        // Metadata is inline; the actor cause owns a shared native denial allocation.
        let Self {
            identity: _identity,
            kind,
            causes,
            explanation_cause: _explanation_cause,
            subject,
            workflow_actor_cause,
            lifecycle_storage_cause,
        } = self;
        (std::mem::size_of_val(kind.as_ref()) as u64)
            .saturating_add(
                (causes.capacity() as u64).saturating_mul(std::mem::size_of::<
                    super::WorthQueryOperationAuthorizationDenialKind,
                >() as u64),
            )
            .saturating_add(subject.capacity() as u64)
            .saturating_add(lifecycle_storage_cause.as_ref().map_or(0, |cause| {
                (std::mem::size_of_val(cause.as_ref()) + 2 * std::mem::size_of::<usize>()) as u64
            }))
            .saturating_add(
                workflow_actor_cause
                    .as_ref()
                    .map_or(0, |cause| cause.additional_charged_bytes()),
            )
    }
}
