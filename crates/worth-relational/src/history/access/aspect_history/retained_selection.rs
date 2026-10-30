//! One sealed selection shared by exact canonical-commit cursors; no continuation authority.
use super::{RelationalRecordHistoryDenial as Denial, RelationalRecordHistorySubject};
use crate::runtime::RelationalRuntime;
use std::{num::NonZeroUsize, sync::Arc};
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::AspectKey;

#[derive(Clone, Copy, Debug)]
pub struct RelationalRecordHistorySelectionLimits {
    pub maximum_subjects: NonZeroUsize,
    pub maximum_aspects: NonZeroUsize,
    pub maximum_owned_bytes: NonZeroUsize,
}

/// Genuine retained generations and actual selected keys, canonicalized once.
#[derive(Clone, Debug)]
pub struct RelationalRecordHistorySelection {
    owner: Arc<RetainedSelection>,
}

#[derive(Debug)]
struct RetainedSelection {
    subjects: Vec<RelationalRecordHistorySubject>,
    aspects: Vec<AspectKey>,
    runtime: u64,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

impl RelationalRecordHistorySelection {
    pub fn subjects(&self) -> &[RelationalRecordHistorySubject] {
        &self.owner.subjects
    }
    pub fn aspects(&self) -> &[AspectKey] {
        &self.owner.aspects
    }
    pub(super) fn runtime(&self) -> u64 {
        self.owner.runtime
    }
}

impl RelationalRuntime {
    /// Limits are checked before walking, copying, or canonicalizing the input.
    #[expect(
        clippy::disallowed_methods,
        reason = "Comparison uses declared semantic keys; type names label diagnostics only"
    )]
    pub fn retain_record_history_selection(
        &self,
        execution: ExecutionRequest<'_, '_>,
        subjects: &[RelationalRecordHistorySubject],
        aspects: &[AspectKey],
        limits: RelationalRecordHistorySelectionLimits,
    ) -> Result<RelationalRecordHistorySelection, Denial> {
        if subjects.is_empty() {
            return Err(Denial::EmptySubjectSelection);
        }
        if aspects.is_empty() {
            return Err(Denial::EmptyAspectSelection);
        }
        if subjects.len() > limits.maximum_subjects.get()
            || aspects.len() > limits.maximum_aspects.get()
        {
            return Err(Denial::SelectionLimitExceeded);
        }
        super::request_stage::run(execution, |checkpoint| {
            for subject in subjects {
                charge(checkpoint, 1)?;
                if subject.runtime_instance_id() != self.runtime_instance_id() {
                    return Err(Denial::ForeignOwner);
                }
            }
            let bytes = selection_bytes(subjects, aspects, checkpoint)?;
            if bytes > limits.maximum_owned_bytes.get() as u64 {
                return Err(Denial::SelectionLimitExceeded);
            }
            let memory = execution
                .reserve_retained_memory(bytes)
                .map_err(Denial::Request)?;
            for count in [subjects.len(), aspects.len()] {
                let count = u64::try_from(count).map_err(|_| Denial::SizeOverflow)?;
                let work = count
                    .checked_mul(u64::from(count.checked_ilog2().unwrap_or(0)) + 1)
                    .ok_or(Denial::SizeOverflow)?;
                charge(checkpoint, work)?;
            }
            let mut subjects = subjects.to_vec();
            subjects.sort_unstable_by(|left, right| left.record().cmp(right.record()));
            subjects.dedup_by(|left, right| left.record() == right.record());
            let mut aspects = aspects.to_vec();
            aspects.sort_unstable();
            aspects.dedup();
            Ok(RelationalRecordHistorySelection {
                owner: Arc::new(RetainedSelection {
                    subjects,
                    aspects,
                    runtime: self.runtime_instance_id(),
                    _memory: memory,
                }),
            })
        })
    }
}

fn charge(checkpoint: &mut MapKernelContext<'_, '_>, work: u64) -> Result<(), Denial> {
    if checkpoint.has_accounting() {
        checkpoint.checkpoint(work)?;
    } else {
        checkpoint.poll()?;
    }
    Ok(())
}

fn selection_bytes(
    subjects: &[RelationalRecordHistorySubject],
    aspects: &[AspectKey],
    checkpoint: &mut MapKernelContext<'_, '_>,
) -> Result<u64, Denial> {
    let bytes = subjects
        .len()
        .checked_mul(size_of::<RelationalRecordHistorySubject>())
        .and_then(|bytes| bytes.checked_add(aspects.len().checked_mul(size_of::<AspectKey>())?))
        .and_then(|bytes| {
            bytes.checked_add(size_of::<RetainedSelection>() + 2 * size_of::<usize>())
        })
        .ok_or(Denial::SizeOverflow)?;
    let bytes = aspects.iter().try_fold(bytes, |bytes, aspect| {
        charge(checkpoint, 1)?;
        bytes
            .checked_add(aspect.owned_allocation_capacity_bytes())
            .ok_or(Denial::SizeOverflow)
    })?;
    u64::try_from(bytes).map_err(|_| Denial::SizeOverflow)
}
