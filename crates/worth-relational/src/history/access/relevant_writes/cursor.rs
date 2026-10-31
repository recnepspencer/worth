use super::page::{
    RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
    RelationalAuthoredWritePageLimits, RelationalPreparedAuthoredWritePage,
};
use super::RelationalAuthoredWriteDenial as Denial;
use crate::history::AuthoredWritePostingKey as Key;
mod admission;
use crate::branch::AdmittedRelationalBranchBasis;
use crate::change_source::RelationalSelectedCommit;
use crate::history::data::CommitId;
use crate::runtime::{AuthoredWriteGeneration, RelationalRuntime};
use std::sync::Arc;
use worth_execution::{
    ChargedBytes, ExecutionRequest, ExecutionRequestArrayBuilder, MapKernelContext,
};

/// A finite continuation over one owner-selected canonical single-parent prefix.
#[derive(Debug)]
pub struct RelationalAuthoredWriteCursor {
    source: Arc<RetainedAuthoredWriteSelection>,
    next: u64,
}
#[derive(Debug)]
pub(super) struct RetainedAuthoredWriteSelection {
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

impl RelationalAuthoredWriteCursor {
    /// Each page probes the selected keys in the key tree and their per-key
    /// postings, then resolves only emitted canonical occurrences. Prefix proof
    /// uses its own logarithmic lookup. No unrelated records or sibling branches
    /// are enumerated. Refusal changes no position.
    pub fn prepare_next_page<'request, 'authority>(
        &self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'request, 'authority>,
        limits: RelationalAuthoredWritePageLimits,
    ) -> Result<RelationalPreparedAuthoredWritePage<'request, 'authority>, Denial> {
        if self
            .source
            .target
            .observation()
            .identity()
            .runtime_instance_id()
            != runtime.runtime_instance_id()
        {
            return Err(Denial::ForeignOwner);
        }
        let page = super::run(execution, |context| {
            self.prepare(runtime, execution, context, limits)
        })?;
        Ok(RelationalPreparedAuthoredWritePage {
            source: Arc::clone(&self.source),
            expected: self.next,
            page,
            execution,
        })
    }

    /// Final handoff polls the same original request retained by the preparation.
    pub fn accept_page(
        &self,
        prepared: RelationalPreparedAuthoredWritePage<'_, '_>,
    ) -> Result<(RelationalAuthoredWritePage, Self), Denial> {
        if !Arc::ptr_eq(&self.source, &prepared.source) || self.next != prepared.expected {
            return Err(Denial::InvalidContinuation);
        }
        prepared.execution.consult().map_err(Denial::Request)?;
        let successor = Self {
            source: Arc::clone(&self.source),
            next: prepared.page.next,
        };
        Ok((prepared.page, successor))
    }

    pub fn next_page(
        &mut self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'_, '_>,
        limits: RelationalAuthoredWritePageLimits,
    ) -> Result<RelationalAuthoredWritePage, Denial> {
        let prepared = self.prepare_next_page(runtime, execution, limits)?;
        let (page, successor) = self.accept_page(prepared)?;
        *self = successor;
        Ok(page)
    }

    fn prepare(
        &self,
        runtime: &RelationalRuntime,
        execution: ExecutionRequest<'_, '_>,
        context: &mut MapKernelContext<'_, '_>,
        limits: RelationalAuthoredWritePageLimits,
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
        let (mut next, mut complete) = (self.next, false);
        while slots.len() < capacity {
            let Some((ordinal, commit)) = self.source.next_occurrence(next, context)? else {
                complete = true;
                break;
            };
            let canonical = runtime
                .history_access()
                .demand_commit_envelope(&execution, commit)
                .map_err(Denial::Execution)?
                .ok_or(Denial::CanonicalEvidenceUnavailable)?;
            if let Some(catalog) = &self.source.managed_catalog {
                super::managed_source::require_packet(
                    &canonical,
                    catalog,
                    &self.source.keys,
                    runtime.runtime_instance_id(),
                    context,
                )?;
            }
            super::ordered_visit(context, self.source.generation.prefix.len(), 8)?;
            if self
                .source
                .generation
                .prefix
                .get(&canonical.commit.commit_id)
                != Some(&ordinal)
            {
                return Err(Denial::CanonicalEvidenceUnavailable);
            }
            slots.push(Some(RelationalAuthoredWriteOccurrence { canonical }))?;
            next = ordinal.checked_add(1).ok_or(Denial::SizeOverflow)?;
        }
        if !complete {
            complete = self.source.next_occurrence(next, context)?.is_none();
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
}

impl RetainedAuthoredWriteSelection {
    fn next_occurrence(
        &self,
        next: u64,
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<Option<(u64, CommitId)>, Denial> {
        let mut first = None;
        for key in self.keys.iter() {
            let width = selector_bytes(context, key)?
                .checked_add(std::mem::size_of::<Key>() as u64)
                .ok_or(Denial::Interrupted(
                    worth_execution::MapKernelStop::WorkCounterOverflow,
                ))?;
            super::ordered_visit(context, self.generation.writes.len(), width)?;
            if let Some(postings) = self.generation.writes.get(key) {
                super::ordered_visit(context, postings.0.len(), 8)?;
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

fn selector_bytes(context: &mut MapKernelContext<'_, '_>, key: &Key) -> Result<u64, Denial> {
    let fields = match key {
        Key::Ordinary(super::RelationalAuthoredWriteSelector::Record {
            scope: super::RelationalAuthoredWriteScope::Field(_, path),
            ..
        }) => path.fields().len(),
        Key::Ordinary(super::RelationalAuthoredWriteSelector::EntityFieldIndex {
            definition,
            ..
        }) => definition.locator.field_path().fields().len(),
        _ => 0,
    };
    super::ordered_visit(context, 0, fields as u64)?;
    Ok(key.additional_charged_bytes())
}
