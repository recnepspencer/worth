use super::{AuthoredWriteGeneration, Denial};
use crate::history::data::CommitId;
use worth_execution::{ExecutionOrderedMap, ExecutionRequest};

impl super::super::RelationalCanonicalPublicationRoutes {
    pub(crate) fn discard_authored_generation(&self, selected: CommitId) -> Result<(), Denial> {
        let route = self
            .by_commit
            .get(&selected)
            .ok_or(Denial::CanonicalEvidenceUnavailable)?;
        if !route.is_visible() {
            return Err(Denial::CanonicalEvidenceUnavailable);
        }
        route
            .authored_writes
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .take();
        Ok(())
    }

    /// Explicit cold reconstruction. No other source lineage is traversed and
    /// no generation becomes visible until the complete selected prefix exists.
    pub(crate) fn rebuild_authored_generation(
        &self,
        execution: ExecutionRequest<'_, '_>,
        selected: CommitId,
    ) -> Result<usize, Denial> {
        let (generation, count) = crate::history::run_authored_write_stage(execution, |context| {
            let mut chain = ExecutionOrderedMap::<u64, CommitId>::default();
            let mut visited = ExecutionOrderedMap::<CommitId, ()>::default();
            let mut next = Some(selected);
            let mut count = 0u64;
            while let Some(commit) = next {
                crate::history::visit_authored_write(context)?;
                crate::history::visit_authored_write_lookup(context, visited.len(), 8)?;
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
                crate::history::visit_authored_write_lookup(context, chain.len(), 8)?;
                chain.insert_cloned_native_for_request(&count, &commit, execution)?;
                crate::history::visit_authored_write_lookup(context, visited.len(), 8)?;
                visited.insert_cloned_native_for_request(&commit, &(), execution)?;
                count = count.checked_add(1).ok_or(Denial::SizeOverflow)?;
            }
            let mut generation = AuthoredWriteGeneration::default();
            for rank in (0..chain.len()).rev() {
                crate::history::visit_authored_write(context)?;
                crate::history::visit_authored_write_lookup(context, chain.len(), 8)?;
                let (_, commit) = chain.entry_at(rank).expect("retained chain rank");
                let envelope = self
                    .demand_envelope(&execution, *commit)
                    .map_err(|cause| Denial::Execution(cause.into()))?
                    .ok_or(Denial::CanonicalEvidenceUnavailable)?;
                generation.extend(execution, context, &envelope)?;
            }
            Ok((
                generation,
                usize::try_from(count).map_err(|_| Denial::SizeOverflow)?,
            ))
        })?;
        // The original request's final handoff precedes the only mutation.
        // No control refusal after publication can hide an installed generation.
        let route = self
            .by_commit
            .get(&selected)
            .ok_or(Denial::CanonicalEvidenceUnavailable)?;
        if !route.is_visible() {
            return Err(Denial::CanonicalEvidenceUnavailable);
        }
        *route
            .authored_writes
            .lock()
            .unwrap_or_else(|p| p.into_inner()) = Some(generation);
        Ok(count)
    }
}
