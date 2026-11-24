//! Real allocation bindings resolve ordinary output correspondence without ownership.
use super::*;

impl RelationalAuthoredCoupledCandidate<'_> {
    /// Resolve the roles of the actual appended participant on this proposal.
    /// Allocation identity is observation only: it is neither a correspondence
    /// nor permission to append or publish a participant.
    pub fn appended_output_entities(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        producer: &str,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<
        Option<worth_execution::ExecutionArray<crate::identity::data::EntityId>>,
        TransactionCommitError,
    > {
        super::route_selection::output_entities(
            execution,
            &self.parts,
            &self.mutation.outcome.created_entities,
            producer,
            meaning,
        )
    }
}
