use branch::history::access::relevant_writes::generation::AuthoredWriteGeneration;
use branch::history::access::relevant_writes::RelationalAuthoredWriteDenial as Denial;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};
use worth_relational_branch::internal_port as branch;
use worth_relational_contracts::internal_port::history::data::coordinates::CommitId;

impl super::super::RelationalCanonicalPublicationRoutes {
    pub(crate) fn discard_authored_generation(&self, selected: CommitId) -> Result<(), Denial> {
        let owner = self
            .authored_generation_owner(selected)
            .ok_or(Denial::CanonicalEvidenceUnavailable)?;
        owner.replace_authored_write_generation(None);
        Ok(())
    }

    /// Explicit cold reconstruction. No other source lineage is traversed and
    /// no generation becomes visible until the complete selected prefix exists.
    pub(crate) fn rebuild_authored_generation(
        &self,
        _lane: &worth_relational_contracts::internal_port::storage::substrate::maintenance::reconstruction::ReconstructionLane<'_>,
        execution: ExecutionRequest<'_, '_>,
        selected: CommitId,
    ) -> Result<usize, Denial> {
        let (generation, count) =
            branch::history::access::relevant_writes::run(execution, |context| {
                let mut chain = ExecutionOrderedMap::<u64, CommitId>::default();
                let mut visited = ExecutionOrderedMap::<CommitId, ()>::default();
                let mut next = Some(selected);
                let mut count = 0u64;
                while let Some(commit) = next {
                    branch::history::access::relevant_writes::visit(context)?;
                    branch::history::access::relevant_writes::ordered_visit(
                        context,
                        visited.len(),
                    )?;
                    if visited.contains_key(&commit) {
                        return Err(Denial::NotSelectedPrefix);
                    }
                    let envelope = self
                        .demand_envelope(&execution, commit)
                        .map_err(|cause| Denial::Execution(cause.into()))?
                        .ok_or(Denial::CanonicalEvidenceUnavailable)?;
                    next = match envelope.commit.parents.as_slice() {
                        [] => None,
                        [parent] => Some(*parent),
                        _ => return Err(Denial::UnsupportedMergeInterval),
                    };
                    branch::history::access::relevant_writes::ordered_visit(context, chain.len())?;
                    chain.insert_cloned_native_for_request(&count, &commit, execution)?;
                    branch::history::access::relevant_writes::ordered_visit(
                        context,
                        visited.len(),
                    )?;
                    visited.insert_cloned_native_for_request(&commit, &(), execution)?;
                    count = count.checked_add(1).ok_or(Denial::SizeOverflow)?;
                }
                let mut generation = AuthoredWriteGeneration::default();
                for rank in (0..chain.len()).rev() {
                    branch::history::access::relevant_writes::visit(context)?;
                    branch::history::access::relevant_writes::ordered_visit(context, chain.len())?;
                    let (_, commit) = chain.entry_at(rank).expect("retained chain rank");
                    let envelope = self
                        .demand_envelope(&execution, *commit)
                        .map_err(|cause| Denial::Execution(cause.into()))?
                        .ok_or(Denial::CanonicalEvidenceUnavailable)?;
                    super::extend_generation(&mut generation, execution, context, &envelope)?;
                }
                Ok((
                    generation,
                    usize::try_from(count).map_err(|_| Denial::SizeOverflow)?,
                ))
            })?;
        // The original request's final handoff precedes the only mutation.
        // No control refusal after publication can hide an installed generation.
        let owner = self
            .authored_generation_owner(selected)
            .ok_or(Denial::CanonicalEvidenceUnavailable)?;
        owner.replace_authored_write_generation(Some(generation));
        Ok(count)
    }
}
