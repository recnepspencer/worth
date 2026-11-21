//! Actual graph discoveries belong to the registered edit, even without a new writer.
use super::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::{
    provider::{
        CandidateReadSupport, ParentRead, RetainedCandidateRead, RetainedReadRequalification,
    },
    WorthQueryApplicationAttemptDenial as Denial, WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_relational::facade::identity::EntityId;
#[derive(Default)]
pub(super) struct CandidateReadSupports {
    values: Vec<RetainedCandidateRead>,
    storage: Option<worth_execution::ExecutionMemoryReservation>,
    /// The slot of each retained verified parent, by its read.
    by_read: worth_execution::ExecutionOrderedMap<usize, usize>,
    /// The slots each wave task's discovery owns.
    by_owner: worth_execution::ExecutionOrderedMap<(usize, usize), ()>,
}
impl CandidateReadSupports {
    fn retain(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: EntityId,
        position: usize,
        owner: Option<usize>,
        support: CandidateReadSupport,
    ) -> Result<(), Denial> {
        let subject = support.subject();
        execution
            .run_native_with_checkpoint(|context| {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, 1)
            })
            .map_err(|cause| Denial::relational_read_stopped(cause.into(), subject))?
            .0
            .map_err(|cause| {
                Denial::relational_read_stopped(
                    worth_execution::WorkCeilingDenial::Stopped(cause).into(),
                    subject,
                )
            })?;
        if self.values.len() == self.values.capacity() {
            let capacity = || Denial::new(Kind::RetainedSourceStorageDenied, subject);
            let next = self
                .values
                .capacity()
                .checked_mul(2)
                .ok_or_else(capacity)?
                .max(1);
            let bytes = next
                .checked_mul(std::mem::size_of::<RetainedCandidateRead>())
                .and_then(|bytes| u64::try_from(bytes).ok())
                .ok_or_else(capacity)?;
            let memory = execution
                .allocation_policy()
                .reserve_memory(bytes)
                .map_err(|cause| Denial::allocation_denied(subject, cause))?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(next)
                .map_err(|_| Denial::new(Kind::CandidateCapacityExceeded, subject))?;
            if replacement.capacity() != next {
                return Err(Denial::new(Kind::CandidateCapacityExceeded, subject));
            }
            execution
                .run_native_with_checkpoint(|context| {
                    crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                        context, bytes,
                    )
                })
                .map_err(|cause| Denial::relational_read_stopped(cause.into(), subject))?
                .0
                .map_err(|cause| {
                    Denial::relational_read_stopped(
                        worth_execution::WorkCeilingDenial::Stopped(cause).into(),
                        subject,
                    )
                })?;
            replacement.append(&mut self.values);
            self.values = replacement;
            self.storage = memory;
        }
        let slot = self.values.len();
        self.index(execution, slot, owner, &support, subject)?;
        self.values.push(RetainedCandidateRead {
            source,
            position,
            owner,
            support,
            window_end: None,
        });
        Ok(())
    }

    fn index(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        slot: usize,
        owner: Option<usize>,
        support: &CandidateReadSupport,
        subject: &str,
    ) -> Result<(), Denial> {
        let stopped = |cause| match cause {
            worth_execution::ExecutionOrderedMapDenial::Allocation(cause) => {
                Denial::allocation_denied(subject, cause)
            }
            worth_execution::ExecutionOrderedMapDenial::Request(cause) => {
                Denial::relational_read_stopped(cause.into(), subject)
            }
            worth_execution::ExecutionOrderedMapDenial::CapacityOverflow => {
                Denial::new(Kind::CandidateCapacityExceeded, subject)
            }
        };
        if let CandidateReadSupport::Parent(parent) = support
            && let Some(read) = parent.read()
        {
            charge_scan(
                execution,
                "retained read index",
                log_work(self.by_read.len()),
            )?;
            self.by_read
                .insert_cloned_native_for_request(&read.0, &slot, *execution)
                .map_err(stopped)?;
        }
        if let Some(owner) = owner {
            charge_scan(
                execution,
                "retained read index",
                log_work(self.by_owner.len()),
            )?;
            self.by_owner
                .insert_cloned_native_for_request(&(owner, slot), &(), *execution)
                .map_err(stopped)?;
        }
        Ok(())
    }
}

fn log_work(entries: usize) -> usize {
    entries.max(1).ilog2() as usize + 1
}
impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn retain_candidate_read_support(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: EntityId,
        position: usize,
        owner: Option<usize>,
        support: CandidateReadSupport,
    ) -> Result<(), Denial> {
        self.candidate_read_support
            .retain(execution, source, position, owner, support)
    }
    /// Discovery evidence a reopened task retained ends at its invalidator.
    pub(in crate::domain_computation::primary_graph) fn supersede_owned_candidate_support(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        owner: usize,
        invalidator: usize,
    ) -> Result<(), Denial> {
        let subject = "reopened discovery support";
        let supports = &mut self.candidate_read_support;
        charge_scan(execution, subject, log_work(supports.by_owner.len()))?;
        let Some(mut rank) = supports
            .by_owner
            .first_at_or_after(&(owner, 0))
            .and_then(|(first, ())| supports.by_owner.rank_of(first))
        else {
            return Ok(());
        };
        while let Some((&(found, slot), ())) = supports.by_owner.entry_at(rank) {
            if found != owner {
                break;
            }
            charge_scan(execution, subject, 1)?;
            crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::visited_retained_reads(1);
            let entry = &mut supports.values[slot];
            // Only evidence retained before the invalidating append.
            if entry.position < invalidator && entry.window_end.is_none() {
                entry.window_end = Some(invalidator);
            }
            rank += 1;
        }
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn reset_candidate_read_support(&mut self) {
        self.candidate_read_support = CandidateReadSupports::default();
    }
    /// The wave must replace these by a complete pure outgoing pass before returning.
    pub(in crate::domain_computation::primary_graph) fn refresh_outgoing_read_support(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let subject = "outgoing discovery support refresh";
        let work = u64::try_from(self.candidate_read_support.values.len())
            .map_err(|_| Denial::new(Kind::CandidateCapacityExceeded, subject))?;
        execution
            .run_native_with_checkpoint(|context| {
                crate::domain_computation::primary_graph::admitted_capture::checkpoint(
                    context, work,
                )
            })
            .map_err(|cause| Denial::relational_read_stopped(cause.into(), subject))?
            .0
            .map_err(|cause| {
                Denial::relational_read_stopped(
                    worth_execution::WorkCeilingDenial::Stopped(cause).into(),
                    subject,
                )
            })?;
        let supports = &mut self.candidate_read_support;
        supports
            .values
            .retain(|entry| !matches!(entry.support, CandidateReadSupport::Outgoing(_)));
        // Slots moved: index the kept entries again.
        supports.by_read = worth_execution::ExecutionOrderedMap::default();
        supports.by_owner = worth_execution::ExecutionOrderedMap::default();
        let kept = std::mem::take(&mut supports.values);
        for (slot, entry) in kept.iter().enumerate() {
            supports.index(execution, slot, entry.owner, &entry.support, subject)?;
        }
        supports.values = kept;
        Ok(())
    }
    pub(in crate::domain_computation::primary_graph) fn candidate_read_support(
        &self,
    ) -> &[RetainedCandidateRead] {
        &self.candidate_read_support.values
    }
    /// Compare the verified parent retained as `read` from its own image to
    /// `current`. A change base does not settle supersedes it at `invalidator`.
    #[allow(clippy::too_many_arguments)]
    pub(in crate::domain_computation::primary_graph) fn requalify_retained_parent(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &worth_relational::facade::runtime::RelationalRuntime,
        before: &worth_relational::facade::snapshots::SnapshotHandle,
        read: ParentRead,
        current: &worth_relational::facade::runtime::RelationalPinnedRead<
            worth_relational::facade::runtime::RelationalAppliedCandidateReadIdentity,
        >,
        invalidator: usize,
        settles: impl Fn(&crate::domain_computation::primary_graph::application_contribution::producer::VerifiedAppliedParent) -> bool,
    ) -> Result<RetainedReadRequalification, Denial> {
        let supports = &mut self.candidate_read_support;
        charge_scan(
            execution,
            "retained parent qualification",
            log_work(supports.by_read.len()),
        )?;
        let Some(&slot) = supports.by_read.get(&read.0) else {
            // An assembly round reset this support; nothing retained it.
            return Ok(RetainedReadRequalification::Equal);
        };
        crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::visited_retained_reads(1);
        let entry = &mut supports.values[slot];
        let CandidateReadSupport::Parent(parent) = &entry.support else {
            unreachable!("matched a verified parent")
        };
        if parent
            .changed_fact(execution, runtime, before, parent.pin(), current)?
            .is_none()
        {
            return Ok(RetainedReadRequalification::Equal);
        }
        if settles(parent) {
            return Ok(RetainedReadRequalification::Settled);
        }
        entry.window_end = Some(invalidator);
        Ok(RetainedReadRequalification::Superseded)
    }
}

/// Charge one pass over the retained reads.
fn charge_scan(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    subject: &'static str,
    entries: usize,
) -> Result<(), Denial> {
    let work = u64::try_from(entries)
        .map_err(|_| Denial::new(Kind::CandidateCapacityExceeded, subject))?;
    crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::observe_scheduling_work(work);
    execution
        .run_native_with_checkpoint(|context| {
            crate::domain_computation::primary_graph::admitted_capture::checkpoint(context, work)
        })
        .map_err(|cause| Denial::relational_read_stopped(cause.into(), subject))?
        .0
        .map_err(|cause| {
            Denial::relational_read_stopped(
                worth_execution::WorkCeilingDenial::Stopped(cause).into(),
                subject,
            )
        })
}
