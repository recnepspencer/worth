//! Compact task lifecycle and request-funded container helpers for the wave.
use super::*;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionOrderedMap, ExecutionOrderedMapDenial, ExecutionRequest,
};

pub(super) enum TaskState {
    Pending,
    Identified,
    Alias {
        canonical: usize,
    },
    Closed {
        inputs: Option<PreparedCoupledInputs>,
        waiting: usize,
    },
    Done {
        parent: Option<AppliedOutputParent>,
    },
}

pub(super) struct Scheduled<S: ApplicationSchema> {
    pub(super) root: bool,
    /// Reopened after its retained parent went stale (PR C R2): it seeds
    /// `needed` like a root and reads its source with no reuse until done.
    pub(super) reopened: bool,
    pub(super) needed: bool,
    pub(super) key: Option<super::super::occurrences::OccurrenceKey>,
    pub(super) next_bucket: Option<usize>,
    pub(super) component: Option<std::any::TypeId>,
    pub(super) state: TaskState,
    pub(super) close_epoch: u32,
    pub(super) prerequisite_queued: bool,
    pub(super) next_prerequisite: Option<usize>,
    pub(super) continuation_done: bool,
    pub(super) waiting_aliases_head: Option<usize>,
    pub(super) next_waiting_alias: Option<usize>,
    pub(super) prerequisites: Vec<usize>,
    pub(super) _prerequisite_memory: Option<ExecutionMemoryReservation>,
    pub(super) dependents: Vec<(usize, u32)>,
    pub(super) _dependents_memory: Option<ExecutionMemoryReservation>,
    /// Every close that read this task, kept after it completes: a reopen
    /// re-closes those that have not run yet (PR C R2).
    pub(super) readers: Vec<(usize, u32)>,
    pub(super) _readers_memory: Option<ExecutionMemoryReservation>,
    /// Later rows coalesced onto this canonical task.
    pub(super) alias_rows: Vec<usize>,
    pub(super) _alias_rows_memory: Option<ExecutionMemoryReservation>,
    /// The rows its discovery selected: its latest pass, plus every row a
    /// quiescent outgoing refresh selected since.
    pub(super) children: Vec<usize>,
    pub(super) _children_memory: Option<ExecutionMemoryReservation>,
    /// Set while reopened: what it selected before, checked on re-discovery.
    pub(super) reopen_check: Option<super::reopen::ReopenCheck>,
    pub(super) subject: &'static str,
    pub(super) rank: usize,
    /// The demand that owes this task, carried to everything it enqueues.
    pub(super) owed:
        crate::domain_computation::primary_graph::application_contribution::coupled_publication::OwedDemand,
    pub(super) tree: Box<dyn capture::RequiredTree<S>>,
    pub(super) _memory: Option<ExecutionMemoryReservation>,
}

impl<S: ApplicationSchema> Scheduled<S> {
    pub(super) fn is_done(&self) -> bool {
        matches!(self.state, TaskState::Done { .. } | TaskState::Alias { .. })
    }

    pub(super) fn canonical_parent(&self) -> Option<usize> {
        match self.state {
            TaskState::Alias { canonical } => Some(canonical),
            _ => None,
        }
    }

    pub(super) fn parent(&self) -> Option<&AppliedOutputParent> {
        match &self.state {
            TaskState::Done { parent } => parent.as_ref(),
            _ => None,
        }
    }

    pub(super) fn parent_mut(&mut self) -> Option<&mut AppliedOutputParent> {
        match &mut self.state {
            TaskState::Done { parent } => parent.as_mut(),
            _ => None,
        }
    }

    pub(super) fn inputs(&self) -> Option<&PreparedCoupledInputs> {
        match &self.state {
            TaskState::Closed { inputs, .. } => inputs.as_ref(),
            _ => None,
        }
    }

    pub(super) fn take_inputs(&mut self) -> Option<PreparedCoupledInputs> {
        match &mut self.state {
            TaskState::Closed { inputs, .. } => inputs.take(),
            _ => None,
        }
    }
}

pub(super) fn capacity_denied(subject: &'static str) -> SynchronousOutputFailure {
    SynchronousOutputFailure(
        super::super::super::WorthQueryOutputDemandDenial::new(
            super::super::super::WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
            subject,
        )
        .into(),
    )
}

fn map_denial(subject: &'static str, cause: ExecutionOrderedMapDenial) -> SynchronousOutputFailure {
    use crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial as Attempt;
    match cause {
        ExecutionOrderedMapDenial::Allocation(cause) => SynchronousOutputFailure(
            CoupledSourceReadStop::capture(Attempt::allocation_denied(subject, cause)),
        ),
        ExecutionOrderedMapDenial::Request(cause) => SynchronousOutputFailure(
            CoupledSourceReadStop::capture(Attempt::relational_read_stopped(cause.into(), subject)),
        ),
        ExecutionOrderedMapDenial::CapacityOverflow => capacity_denied(subject),
    }
}

pub(super) fn map_insert<K: Ord + Clone, V: Clone>(
    map: &mut ExecutionOrderedMap<K, V>,
    key: &K,
    value: &V,
    execution: &ExecutionRequest<'_, '_>,
    subject: &'static str,
) -> Result<(), SynchronousOutputFailure> {
    checkpoint(execution, subject, map.len().max(1).ilog2() as u64 + 1)?;
    map.insert_cloned_native_for_request(key, value, *execution)
        .map_err(|cause| map_denial(subject, cause))
}

pub(super) fn map_remove<K: Ord, V>(
    map: &mut ExecutionOrderedMap<K, V>,
    key: &K,
    execution: &ExecutionRequest<'_, '_>,
    subject: &'static str,
) -> Result<bool, SynchronousOutputFailure> {
    checkpoint(execution, subject, map.len().max(1).ilog2() as u64 + 1)?;
    map.remove_for_request(key, *execution)
        .map_err(|cause| map_denial(subject, cause))
}

pub(super) fn push_funded<T>(
    vec: &mut Vec<T>,
    memory: &mut Option<ExecutionMemoryReservation>,
    item: T,
    execution: &ExecutionRequest<'_, '_>,
    subject: &'static str,
) -> Result<(), SynchronousOutputFailure> {
    if vec.len() == vec.capacity() {
        let next = vec
            .capacity()
            .checked_mul(2)
            .ok_or_else(|| capacity_denied(subject))?
            .max(1);
        let bytes = next
            .checked_mul(size_of::<T>())
            .and_then(|bytes| u64::try_from(bytes).ok())
            .ok_or_else(|| capacity_denied(subject))?;
        let reserved = execution
            .allocation_policy()
            .reserve_memory(bytes)
            .map_err(|cause| {
                SynchronousOutputFailure(CoupledSourceReadStop::capture(
                    crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial::allocation_denied(
                        subject, cause,
                    ),
                ))
            })?;
        let mut replacement = Vec::new();
        replacement
            .try_reserve_exact(next)
            .map_err(|_| capacity_denied(subject))?;
        if replacement.capacity() != next {
            return Err(capacity_denied(subject));
        }
        checkpoint(execution, subject, next as u64)?;
        replacement.append(vec);
        *vec = replacement;
        *memory = reserved;
    }
    vec.push(item);
    Ok(())
}
