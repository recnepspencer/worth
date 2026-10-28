use super::page::{
    RelationalAuthoredWriteOccurrence, RelationalAuthoredWritePage,
    RelationalAuthoredWritePageLimits, RelationalPreparedAuthoredWritePage,
};
use super::{RelationalAuthoredWriteDenial as Denial, RelationalAuthoredWriteSelector as Key};
use crate::branch::AdmittedRelationalBranchBasis;
use crate::change_source::{RelationalRetainedObservation, RelationalSelectedCommit};
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
    _key_memory: Option<worth_execution::ExecutionMemoryReservation>,
    _original: RelationalRetainedObservation,
    target: RelationalRetainedObservation,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalRuntime {
    /// Select relevant authored writes strictly after the genuine source root.
    /// Prefix membership is taken from the target's canonical parent-derived
    /// generation, never from branch names or a supplied revision equality.
    /// Multi-parent histories and unavailable accelerators fail closed. Index
    /// predicates additionally require actual installed-definition coverage
    /// throughout the open source-to-target canonical interval.
    pub fn authored_write_history_cursor(
        &self,
        execution: ExecutionRequest<'_, '_>,
        original: &AdmittedRelationalBranchBasis,
        target: RelationalSelectedCommit,
        selectors: &[Key],
    ) -> Result<RelationalAuthoredWriteCursor, Denial> {
        if original.identity().runtime_instance_id() != self.runtime_instance_id()
            || target.runtime_instance_id() != self.runtime_instance_id()
        {
            return Err(Denial::ForeignOwner);
        }
        if target.observation().commit_id() != Some(target.commit_id()) {
            return Err(Denial::NotExactSelection);
        }
        if selectors.is_empty() {
            return Err(Denial::EmptySelection);
        }
        super::run(execution, |context| {
            for key in selectors {
                super::visit(context)?;
                if let Key::EntityFieldIndex { definition, .. } = key
                    && !super::RelationalAuthoredEntityIndexDefinition::supports_locator(
                        &definition.locator,
                    )
                {
                    return Err(Denial::UnsupportedIndexSelector);
                }
            }
            let original_commit = original
                .observation()
                .commit_id()
                .ok_or(Denial::NoSelectedCommit)?;
            if self
                .history
                .demand_canonical_envelope(&execution, original_commit)
                .map_err(|cause| Denial::NativeRead(cause.into()))?
                .is_none()
            {
                return Err(Denial::SourceUnavailable);
            }
            let generation = self
                .history
                .authored_write_generation(target.commit_id())
                .ok_or(Denial::GenerationUnavailable)?;
            if generation.unsupported_merge {
                return Err(Denial::UnsupportedMergeInterval);
            }
            super::ordered_visit(context, generation.prefix.len(), 8)?;
            let original_ordinal = *generation
                .prefix
                .get(&original_commit)
                .ok_or(Denial::NotSelectedPrefix)?;
            let next = original_ordinal
                .checked_add(1)
                .ok_or(Denial::SizeOverflow)?;
            super::ordered_visit(context, 0, selectors.len() as u64)?;
            if selectors
                .iter()
                .any(|key| matches!(key, Key::EntityFieldIndex { .. }))
            {
                super::ordered_visit(context, generation.incomplete_indexes.len(), 8)?;
                if generation
                    .incomplete_indexes
                    .first_at_or_after(&next)
                    .is_some()
                {
                    return Err(Denial::IncompleteIndexHistory);
                }
            }
            // Resolve at the real receiving catalog. A descriptive tuple is
            // insufficient even when older packets were complete for a smaller
            // installed roster. The first coverage and monotone-run boundary
            // prove every packet in this open selected interval.
            let catalog = self.indexes.definition_read_binding().snapshot();
            for key in selectors {
                super::visit(context)?;
                let Key::EntityFieldIndex { definition, .. } = key else {
                    continue;
                };
                let introduction = catalog
                    .introduction(definition, context)?
                    .ok_or(Denial::IncompleteIndexHistory)?;
                if next > generation.ordinal {
                    continue;
                }
                if next < generation.index_run_start {
                    return Err(Denial::IncompleteIndexHistory);
                }
                super::ordered_visit(context, generation.index_coverage.len(), 8)?;
                let Some((_, coverage)) = generation.index_coverage.first_at_or_after(&next) else {
                    return Err(Denial::IncompleteIndexHistory);
                };
                if !coverage.extends(introduction) {
                    return Err(Denial::IncompleteIndexHistory);
                }
            }
            let memory = execution
                .reserve_retained_memory(
                    (std::mem::size_of::<RetainedAuthoredWriteSelection>()
                        + 2 * std::mem::size_of::<usize>()) as u64,
                )
                .map_err(Denial::Request)?;
            let bytes = selectors.iter().try_fold(0u64, |bytes, key| {
                super::visit(context)?;
                bytes
                    .checked_add(selector_bytes(context, key)?)
                    .ok_or(Denial::SizeOverflow)
            })?;
            let key_memory = execution
                .reserve_retained_memory(bytes)
                .map_err(Denial::Request)?;
            let mut keys = ExecutionRequestArrayBuilder::allocate(execution, selectors.len())?;
            for key in selectors {
                let bytes = selector_bytes(context, key)?;
                super::ordered_visit(context, 0, bytes)?;
                keys.push(key.clone())?;
            }
            let original = self
                .retain_observation_snapshot(original)
                .map_err(Denial::Retention)?;
            let target = self
                .retain_observation_snapshot(&target.observation().admitted_basis())
                .map_err(Denial::Retention)?;
            Ok(RelationalAuthoredWriteCursor {
                source: Arc::new(RetainedAuthoredWriteSelection {
                    generation,
                    keys: keys.seal()?,
                    _key_memory: key_memory,
                    _original: original,
                    target,
                    _memory: memory,
                }),
                next,
            })
        })
    }
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
                .history
                .demand_canonical_envelope(&execution, commit)
                .map_err(|cause| Denial::NativeRead(cause.into()))?
                .ok_or(Denial::CanonicalEvidenceUnavailable)?;
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
        Key::Record {
            scope: super::RelationalAuthoredWriteScope::Field(_, path),
            ..
        } => path.fields().len(),
        Key::EntityFieldIndex { definition, .. } => definition.locator.field_path().fields().len(),
        _ => 0,
    };
    super::ordered_visit(context, 0, fields as u64)?;
    Ok(key.additional_charged_bytes())
}
