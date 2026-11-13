//! Actual graph discoveries belong to the registered edit, even without a new writer.
use super::WorthQueryPrimaryGraphApplicationAttempt;
use crate::domain_computation::primary_graph::{
    provider::CandidateReadSupport, WorthQueryApplicationAttemptDenial as Denial,
    WorthQueryApplicationAttemptDenialKind as Kind,
};
use worth_relational::facade::identity::EntityId;
#[derive(Default)]
pub(super) struct CandidateReadSupports {
    values: Vec<(EntityId, usize, CandidateReadSupport)>,
    storage: Option<worth_execution::ExecutionMemoryReservation>,
}
impl CandidateReadSupports {
    fn retain(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: EntityId,
        position: usize,
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
                .checked_mul(std::mem::size_of::<(EntityId, usize, CandidateReadSupport)>())
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
        self.values.push((source, position, support));
        Ok(())
    }
}
impl WorthQueryPrimaryGraphApplicationAttempt {
    pub(in crate::domain_computation::primary_graph) fn retain_candidate_read_support(
        &mut self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: EntityId,
        position: usize,
        support: CandidateReadSupport,
    ) -> Result<(), Denial> {
        self.candidate_read_support
            .retain(execution, source, position, support)
    }
    pub(in crate::domain_computation::primary_graph) fn reset_candidate_read_support(&mut self) {
        self.candidate_read_support = CandidateReadSupports::default();
    }
    pub(in crate::domain_computation::primary_graph) fn candidate_read_support(
        &self,
    ) -> &[(EntityId, usize, CandidateReadSupport)] {
        &self.candidate_read_support.values
    }
}
