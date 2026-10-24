use super::*;

impl DurabilityAuthority<'_> {
    /// Record one settled commit in the runtime's in-memory canonical log.
    ///
    /// The durable medium, when one is attached, already holds the commit:
    /// it was published there before the live cutover.
    pub(crate) fn append_commit(
        &self,
        authority: crate::durability::authority::DurableAppendAuthority,
        positioned: &crate::history::data::PositionedCanonicalCommit,
    ) -> Result<(), DurabilityError> {
        authority.validate(self.runtime.runtime_instance_id(), positioned.envelope())?;
        if self.runtime.durability.take_armed_append_failure_for_test() {
            return Err(DurabilityError::new(
                crate::durability::data::RecoveryFailureClass::DurableIoFailure,
                "test-injected durable append failure",
            ));
        }
        self.runtime
            .durability
            .push_log_envelope(positioned.clone());
        Ok(())
    }
}
