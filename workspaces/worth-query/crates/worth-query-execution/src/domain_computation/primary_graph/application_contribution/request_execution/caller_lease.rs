//! A request opened under its caller's own lease.
//!
//! A run inside an active worker charges only that worker's lineage, so a
//! caller who funds work with `Execution(lease)` inside an advancement opens
//! the advancement under that lease: its worker is then a descendant of the
//! caller's lease, and the caller's charge reaches the lease it names.

use super::{Form, QueryRequestExecution, WorthQueryManagedComputationInterruption};
use worth_execution::ExecutionResourceLease;

impl<'a> QueryRequestExecution<'a> {
    /// The request's execution on a controlled child of the caller's `lease`,
    /// under the lease's policy and cancellation lineage, its deadline and
    /// the caller's own `cancellation` and `deadline`.
    pub(super) fn under_lease(
        lease: &ExecutionResourceLease<'a>,
        cancellation: worth_execution::CancellationToken,
        deadline: Option<std::time::Instant>,
    ) -> Self {
        let policy = *lease.policy();
        Self {
            cancellation: cancellation.clone(),
            deadline,
            declared_work: policy.work_ceiling(),
            certified_seed: super::test_placement::certifying(),
            form: Form::Leased {
                lease: Ok(lease.controlled_child(cancellation, deadline)),
                policy,
            },
        }
    }

    /// The request's cancellation or elapsed deadline, if either happened,
    /// including a cancellation anywhere in its lease's lineage.
    pub(in crate::domain_computation::primary_graph) fn interruption(
        &self,
    ) -> Option<WorthQueryManagedComputationInterruption> {
        let lease = match &self.form {
            Form::Leased { lease, .. } | Form::Serial { lease, .. } => lease.as_ref().ok(),
        };
        if self.cancellation.is_cancelled()
            || lease.is_some_and(ExecutionResourceLease::is_cancelled)
        {
            Some(WorthQueryManagedComputationInterruption::Cancelled)
        } else if self
            .deadline
            .is_some_and(|deadline| std::time::Instant::now() >= deadline)
            || lease.is_some_and(ExecutionResourceLease::deadline_elapsed)
        {
            Some(WorthQueryManagedComputationInterruption::DeadlineExceeded)
        } else {
            None
        }
    }
}
