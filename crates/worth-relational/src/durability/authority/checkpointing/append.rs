use super::*;

impl DurabilityAuthority<'_> {
    /// Record one settled commit in the runtime's in-memory canonical log.
    ///
    /// The durable medium, when one is attached, already holds the commit:
    /// it was published there before the live cutover. This record is how the
    /// runtime retains its own history, and it cannot fail.
    pub(crate) fn append_commit(
        &self,
        positioned: &crate::history::data::PositionedCanonicalCommit,
    ) {
        self.runtime
            .durability
            .push_log_envelope(positioned.clone());
    }
}
