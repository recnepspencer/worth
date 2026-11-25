use super::page::{
    RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
    RelationalAuthoredWritePageLimits,
};
use super::RelationalAuthoredWriteDenial as Denial;
pub mod admission;
use crate::branch::basis::AdmittedRelationalBranchBasis;
use crate::history::access::relevant_writes::generation::AuthoredWriteGeneration;
use crate::history::access::relevant_writes::posting_key::AuthoredWritePostingKey as Key;
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use std::sync::Arc;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder, MapKernelContext};

#[derive(Debug)]
pub struct RetainedAuthoredWriteSelection {
    generation: AuthoredWriteGeneration,
    keys: worth_execution::ExecutionArray<Key>,
    managed_catalog: Option<
        Arc<crate::validation::candidate_source::authored_history::catalog::SourceHistoryCatalog>,
    >,
    _key_memory: Option<worth_execution::ExecutionMemoryReservation>,
    _original: AdmittedRelationalBranchBasis,
    target: AdmittedRelationalBranchBasis,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RetainedAuthoredWriteSelection {
    pub fn require_runtime(&self, runtime_instance_id: u64) -> Result<(), Denial> {
        if self.target.identity().runtime_instance_id() != runtime_instance_id {
            return Err(Denial::ForeignOwner);
        }
        Ok(())
    }

    pub fn prepare_page(
        &self,
        next_ordinal: u64,
        execution: ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        limits: RelationalAuthoredWritePageLimits,
        mut demand_envelope: impl FnMut(CommitId) -> Result<Arc<CanonicalCommitEnvelope>, Denial>,
    ) -> Result<RelationalAuthoredWritePage, Denial> {
        let capacity = limits
            .maximum_entries
            .get()
            .min(limits.maximum_source_visits.get())
            .min(
                limits.maximum_owned_bytes.get()
                    / std::mem::size_of::<Option<RelationalAuthoredWriteOccurrence>>(),
            );
        if capacity == 0 {
            return Err(Denial::PageStorageTooSmall);
        }
        let mut slots = ExecutionRequestArrayBuilder::allocate(execution, capacity)?;
        let (mut next, mut complete) = (next_ordinal, false);
        while slots.len() < capacity {
            let Some((ordinal, commit)) = self.next_occurrence(next, context)? else {
                complete = true;
                break;
            };
            let canonical = demand_envelope(commit)?;
            if let Some(catalog) = &self.managed_catalog {
                super::managed_source::require_packet(
                    &canonical,
                    catalog,
                    &self.keys,
                    self.target.identity().runtime_instance_id(),
                    context,
                )?;
            }
            super::ordered_visit(context, self.generation.prefix.len())?;
            if self.generation.prefix.get(&canonical.commit.commit_id) != Some(&ordinal) {
                return Err(Denial::CanonicalEvidenceUnavailable);
            }
            slots.push(Some(RelationalAuthoredWriteOccurrence {
                canonical: crate::history::data::canonical_commit_envelope::shared::SharedCanonicalCommitEnvelopeOwnerPort::from_arc(canonical),
            }))?;
            next = ordinal.checked_add(1).ok_or(Denial::SizeOverflow)?;
        }
        if !complete {
            complete = self.next_occurrence(next, context)?.is_none();
        }
        let count = slots.len();
        while slots.len() < capacity {
            super::visit(context)?;
            slots.push(None)?;
        }
        Ok(RelationalAuthoredWritePage {
            slots: slots.seal()?,
            count,
            next,
            complete,
            visits: count,
        })
    }

    fn next_occurrence(
        &self,
        next: u64,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<(u64, CommitId)>, Denial> {
        let mut first = None;
        for key in self.keys.iter() {
            super::ordered_visit(context, self.generation.writes.len())?;
            if let Some(postings) = self.generation.writes.get(key) {
                super::ordered_visit(context, postings.0.len())?;
                if let Some((&ordinal, &commit)) = postings.0.first_at_or_after(&next)
                    && first.is_none_or(|(previous, _)| ordinal < previous)
                {
                    first = Some((ordinal, commit));
                }
            }
        }
        Ok(first)
    }
}
