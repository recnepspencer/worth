//! Read-only selection of an actual part; appending still admits one writer per route.
use super::*;
impl RelationalAuthoredCoupledCandidate<'_> {
    /// Select an already appended part by the admitted full-generation route.
    /// This index is an observation, never publication or writer authority.
    pub fn prepared_route_index(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<Option<usize>, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                for (index, part) in self.parts.iter().enumerate() {
                    let meaning = part
                        .publication
                        .producing_meaning()
                        .expect("admitted producing part");
                    checkpoint(context, (family.len() + meaning.family.len() + 65) as u64)
                        .map_err(custody::execution_stop)?;
                    if meaning.family == family
                        && meaning.source_scope == source
                        && &meaning.source_partition == partition
                    {
                        return Ok(Some(index));
                    }
                }
                Ok(None)
            })
            .map_err(custody::execution_stop)?
            .0
    }
}
