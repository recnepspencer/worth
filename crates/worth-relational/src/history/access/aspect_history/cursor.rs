use std::sync::Arc;

use worth_foundational::facade::AspectKey;

use crate::change_source::{RelationalRetainedObservation, RelationalSelectedCommit};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;
use crate::runtime::RelationalRuntime;

use super::{
    RelationalPreparedRecordHistoryPage, RelationalRecordHistoryChange,
    RelationalRecordHistoryDenial as Denial, RelationalRecordHistoryPage,
    RelationalRecordHistoryPageLimits, RelationalRecordHistorySelection,
    RelationalRecordHistorySelectionLimits, RelationalRecordHistorySubject,
};

/// Position inside one genuine selected canonical commit. This does not prove
/// product ancestry; a composition owner supplies each selected component commit.
#[derive(Debug)]
pub struct RelationalCanonicalRecordHistoryCursor {
    source: Arc<RetainedCanonicalRecordHistory>,
    next_record: usize,
}

#[derive(Debug)]
pub(super) struct RetainedCanonicalRecordHistory {
    pub(super) envelope: Arc<CanonicalCommitEnvelope>,
    retained: RelationalRetainedObservation,
    pub(super) selection: RelationalRecordHistorySelection,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalRuntime {
    /// A subject may come from an earlier retained root. The selected commit
    /// must independently be exact and owner-issued; neither proves World ancestry.
    pub fn record_aspect_history_cursor(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        selected: RelationalSelectedCommit,
        subjects: &[RelationalRecordHistorySubject],
        aspects: &[AspectKey],
    ) -> Result<RelationalCanonicalRecordHistoryCursor, Denial> {
        use std::num::NonZeroUsize;
        let selection = self.retain_record_history_selection(
            execution,
            subjects,
            aspects,
            RelationalRecordHistorySelectionLimits {
                maximum_subjects: NonZeroUsize::new(subjects.len())
                    .ok_or(Denial::EmptySubjectSelection)?,
                maximum_aspects: NonZeroUsize::new(aspects.len())
                    .ok_or(Denial::EmptyAspectSelection)?,
                maximum_owned_bytes: NonZeroUsize::new(usize::MAX).expect("positive storage bound"),
            },
        )?;
        self.record_aspect_history_cursor_from_selection(execution, selected, &selection)
    }

    /// Open another exact canonical commit while sharing the already funded
    /// sealed selection. Each returned cursor retains its own source identity.
    pub fn record_aspect_history_cursor_from_selection(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        selected: RelationalSelectedCommit,
        selection: &RelationalRecordHistorySelection,
    ) -> Result<RelationalCanonicalRecordHistoryCursor, Denial> {
        if selected.runtime_instance_id() != self.runtime_instance_id()
            || selection.runtime() != self.runtime_instance_id()
        {
            return Err(Denial::ForeignOwner);
        }
        if selected.observation().commit_id() != Some(selected.commit_id()) {
            return Err(Denial::NotExactSelectedCommit);
        }
        super::request_stage::run(execution, |checkpoint| {
            checkpoint.poll()?;
            let memory = execution
                .reserve_retained_memory(
                    RelationalCanonicalRecordHistoryCursor::owned_source_backing_bytes() as u64,
                )
                .map_err(Denial::Request)?;
            let envelope = self
                .history
                .canonical_envelope(selected.commit_id())
                .or_else(|| {
                    self.history
                        .commit_artifact(selected.commit_id())
                        .map(|artifact| Arc::clone(artifact.envelope()))
                })
                .ok_or(Denial::CommitUnavailable)?;
            let retained = self
                .retain_observation_snapshot(&selected.observation().admitted_basis())
                .map_err(Denial::Retention)?;
            Ok(RelationalCanonicalRecordHistoryCursor {
                source: Arc::new(RetainedCanonicalRecordHistory {
                    envelope,
                    retained,
                    selection: selection.clone(),
                    _memory: memory,
                }),
                next_record: 0,
            })
        })
    }
}

impl RelationalCanonicalRecordHistoryCursor {
    /// The new cursor source wrapper shares its canonical envelope and sealed selection.
    pub const fn owned_source_backing_bytes() -> usize {
        size_of::<RetainedCanonicalRecordHistory>() + 2 * size_of::<usize>()
    }

    /// Canonical record patches are the source-visit granule. Selected-aspect
    /// lookup is logarithmic in a record's operations; no patch payload is copied.
    /// All failures leave the accepted position unchanged.
    pub fn next_page(
        &mut self,
        runtime: &RelationalRuntime,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        limits: RelationalRecordHistoryPageLimits,
    ) -> Result<RelationalRecordHistoryPage, Denial> {
        let prepared = self.prepare_next_page(runtime, execution, limits)?;
        let (page, successor) = self.accept_page(prepared)?;
        *self = successor;
        Ok(page)
    }

    /// Stage finite native work before a composition owner completes fresh
    /// authorization/disclosure. No accepted cursor position changes here.
    pub fn prepare_next_page<'request, 'authority>(
        &self,
        runtime: &RelationalRuntime,
        execution: worth_execution::ExecutionRequest<'request, 'authority>,
        limits: RelationalRecordHistoryPageLimits,
    ) -> Result<RelationalPreparedRecordHistoryPage<'request, 'authority>, Denial> {
        if self
            .source
            .retained
            .observation()
            .identity()
            .runtime_instance_id()
            != runtime.runtime_instance_id()
        {
            return Err(Denial::ForeignOwner);
        }
        let page = super::request_stage::run(execution, |checkpoint| {
            self.prepare_page(execution, checkpoint, limits)
        })?;
        Ok(RelationalPreparedRecordHistoryPage {
            source: Arc::clone(&self.source),
            execution,
            expected_record: self.next_record,
            page,
        })
    }

    /// Return a sealed successor only for this cursor's still-current offset.
    /// The stored originating request controls the final handoff. The original
    /// cursor stays unchanged until its composition owner installs the successor.
    pub fn accept_page(
        &self,
        prepared: RelationalPreparedRecordHistoryPage<'_, '_>,
    ) -> Result<(RelationalRecordHistoryPage, Self), Denial> {
        if !Arc::ptr_eq(&self.source, &prepared.source)
            || self.next_record != prepared.expected_record
        {
            return Err(Denial::InvalidContinuation);
        }
        let (accepted, _) = prepared
            .execution
            .run_native(|_| {
                let successor = Self {
                    source: Arc::clone(&self.source),
                    next_record: prepared.page.next_record,
                };
                (prepared.page, successor)
            })
            .map_err(Denial::Request)?;
        Ok(accepted)
    }

    fn prepare_page(
        &self,
        execution: worth_execution::ExecutionRequest<'_, '_>,
        checkpoint: &mut worth_execution::MapKernelContext<'_, '_>,
        limits: RelationalRecordHistoryPageLimits,
    ) -> Result<RelationalRecordHistoryPage, Denial> {
        checkpoint.poll()?;
        let slot_bytes = std::mem::size_of::<Option<RelationalRecordHistoryChange>>();
        let capacity = limits
            .maximum_entries
            .get()
            .min(limits.maximum_source_visits.get())
            .min(limits.maximum_owned_bytes.get() / slot_bytes);
        if capacity == 0 {
            return Err(Denial::PageStorageTooSmall);
        }
        let mut slots =
            worth_execution::ExecutionRequestArrayBuilder::allocate(execution, capacity)?;
        let mut next = self.next_record;
        let mut visits = 0;
        let patches = &self.source.envelope.patch.authoritative_record_patches;
        while next < patches.len()
            && visits < limits.maximum_source_visits.get()
            && slots.len() < capacity
        {
            charge_visit(checkpoint)?;
            let patch = &patches[next];
            visits += 1;
            if self.source.includes(patch, checkpoint)? {
                slots.push(Some(RelationalRecordHistoryChange {
                    source: Arc::clone(&self.source),
                    record_index: next,
                }))?;
            }
            next += 1;
        }
        let entry_count = slots.len();
        while slots.len() < capacity {
            charge_visit(checkpoint)?;
            slots.push(None)?;
        }
        Ok(RelationalRecordHistoryPage {
            slots: slots.seal()?,
            entry_count,
            source_visits: visits,
            next_record: next,
            complete: next == patches.len(),
        })
    }
}

impl RetainedCanonicalRecordHistory {
    fn includes(
        &self,
        patch: &crate::publication::patch::data::PublishedAuthoritativeRecordPatch,
        checkpoint: &mut worth_execution::MapKernelContext<'_, '_>,
    ) -> Result<bool, Denial> {
        if self
            .selection
            .subjects()
            .binary_search_by(|subject| subject.record().cmp(&patch.target))
            .is_err()
        {
            return Ok(false);
        }
        // Native may retain an authored update with no surviving aspect delta.
        // Preserve that occurrence without attributing it to any selected key.
        if patch.structural_change != RecordStructuralChange::Updated
            || patch.authoritative_patch.full_grammar_operation_count() == 0
        {
            return Ok(true);
        }
        for aspect in self.selection.aspects() {
            charge_visit(checkpoint)?;
            if patch.authoritative_patch.changes_aspect(aspect) {
                return Ok(true);
            }
        }
        Ok(false)
    }
}

fn charge_visit(checkpoint: &mut worth_execution::MapKernelContext<'_, '_>) -> Result<(), Denial> {
    if checkpoint.has_accounting() {
        checkpoint.checkpoint(1)?;
    } else {
        checkpoint.poll()?;
    }
    Ok(())
}
