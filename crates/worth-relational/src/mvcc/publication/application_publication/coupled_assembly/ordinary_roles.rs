//! Real allocation bindings resolve ordinary output correspondence without ownership.
use super::producer_posture::resolve_applied_roles;
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
        for part in &self.parts {
            execution
                .run_native_with_checkpoint(|context| checkpoint(context, 1))
                .map_err(custody::execution_stop)?
                .0
                .map_err(custody::execution_stop)?;
            if part
                .publication
                .lineage_attribution()
                .is_some_and(|(actual, _)| actual == producer)
                && part.publication.producing_meaning() == Some(meaning)
            {
                return resolve_applied_roles(
                    execution,
                    part.publication.roles(),
                    &self.mutation.outcome.created_entities,
                )
                .map(Some);
            }
        }
        Ok(None)
    }
}
