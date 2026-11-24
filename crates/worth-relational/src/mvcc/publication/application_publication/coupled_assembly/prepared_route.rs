//! Read-only selection of an actual part; appending still admits one writer per route.
use super::*;
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Only an Edit may reuse an appended participant in another producer round.
    /// Retirement retains every mandatory Preserve participant.
    pub fn permits_unchanged_output_selection(&self) -> bool {
        self.binding.owners.source.is_edit()
    }
    /// Select an already appended part by the admitted full-generation route.
    /// This index is an observation, never publication or writer authority.
    pub fn prepared_route_index(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<usize>, TransactionCommitError> {
        super::route_selection::prepared_index(execution, &self.parts, family, source, partition)
    }
}
