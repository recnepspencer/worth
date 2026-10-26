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
        let id = positioned.commit.commit_id;
        let cell = self
            .runtime
            .history
            .performed_envelope_residency(id)
            .expect("performed publication installed its exact route");
        self.runtime
            .durability
            .push_log_envelope((id, positioned.position(), cell));
    }
}
