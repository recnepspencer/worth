use std::sync::Arc;

use worth_foundational::facade::AspectKey;

use crate::change_source::{RelationalRetainedObservation, RelationalSelectedCommit};
use crate::history::data::CanonicalCommitEnvelope;
use crate::publication::patch::data::RecordStructuralChange;
use crate::runtime::RelationalRuntime;

use super::{
    RelationalPreparedRecordHistoryPage, RelationalRecordHistoryChange,
    RelationalRecordHistoryDenial as Denial, RelationalRecordHistoryPage,
    RelationalRecordHistoryPageLimits, RelationalRecordHistorySubject,
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
    subjects: Vec<RelationalRecordHistorySubject>,
    pub(super) aspects: Vec<AspectKey>,
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
        execution.check_live()?;
        if selected.runtime_instance_id() != self.runtime_instance_id()
            || subjects
                .iter()
                .any(|subject| subject.runtime_instance_id() != self.runtime_instance_id())
        {
            return Err(Denial::ForeignOwner);
        }
        if subjects.is_empty() {
            return Err(Denial::EmptySubjectSelection);
        }
        if aspects.is_empty() {
            return Err(Denial::EmptyAspectSelection);
        }
        if selected.observation().commit_id() != Some(selected.commit_id()) {
            return Err(Denial::NotExactSelectedCommit);
        }
        let envelope = self
            .history
            .canonical_envelope(selected.commit_id())
            .or_else(|| {
                self.history
                    .commit_artifact(selected.commit_id())
                    .map(|artifact| Arc::clone(artifact.envelope()))
            })
            .ok_or(Denial::CommitUnavailable)?;
        let memory = execution
            .allocation_policy()
            .reserve_memory(selection_bytes(subjects, aspects)?)?;
        let retained = self
            .retain_observation_snapshot(&selected.observation().admitted_basis())
            .map_err(Denial::Retention)?;
        let mut subjects = subjects.to_vec();
        subjects.sort_unstable_by(|left, right| left.record().cmp(right.record()));
        subjects.dedup_by(|left, right| left.record() == right.record());
        let mut aspects = aspects.to_vec();
        aspects.sort_unstable();
        aspects.dedup();
        execution.check_live()?;
        Ok(RelationalCanonicalRecordHistoryCursor {
            source: Arc::new(RetainedCanonicalRecordHistory {
                envelope,
                retained,
                subjects,
                aspects,
                _memory: memory,
            }),
            next_record: 0,
        })
    }
}

impl RelationalCanonicalRecordHistoryCursor {
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
        let (page, _) = execution
            .run_native_with_checkpoint(|checkpoint| {
                self.prepare_page(execution, checkpoint, limits)
            })
            .map_err(Denial::Request)?;
        Ok(RelationalPreparedRecordHistoryPage {
            source: Arc::clone(&self.source),
            execution,
            expected_record: self.next_record,
            page: page?,
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
        let mut slots = worth_execution::ExecutionArrayBuilder::allocate(
            capacity,
            execution.allocation_policy(),
        )?;
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
            .subjects
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
        for aspect in &self.aspects {
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

fn selection_bytes(
    subjects: &[RelationalRecordHistorySubject],
    aspects: &[AspectKey],
) -> Result<u64, Denial> {
    let bytes = subjects
        .len()
        .checked_mul(std::mem::size_of::<RelationalRecordHistorySubject>())
        .and_then(|bytes| {
            bytes.checked_add(
                aspects
                    .len()
                    .checked_mul(std::mem::size_of::<AspectKey>())?,
            )
        })
        .and_then(|bytes| bytes.checked_add(std::mem::size_of::<RetainedCanonicalRecordHistory>()))
        .ok_or(Denial::SizeOverflow)?;
    let bytes = aspects.iter().try_fold(bytes, |bytes, aspect| {
        bytes
            .checked_add(aspect.owned_allocation_capacity_bytes())
            .ok_or(Denial::SizeOverflow)
    })?;
    u64::try_from(bytes).map_err(|_| Denial::SizeOverflow)
}
