//! Outcome and work move together through an attempted tree transition.

use worth_execution::{ReductionMetrics, ReductionRunFailure};

use super::super::WorthQueryPartitionedComputationDenial as Denial;
use super::tree_report::WorthQueryPartitionedTreeMetrics as Metrics;
use super::tree_report::{
    ReportedTree, WorthQueryPartitionedTreeRebuildCause as Cause,
    WorthQueryPartitionedTreeRun as Run,
};

pub(super) enum EditDisposition<Tree> {
    Completed(Tree),
    Rebuild(Cause),
}

pub(super) struct TreeAttempt<T, E> {
    outcome: Result<T, E>,
    work: Metrics,
}

impl<T, E> TreeAttempt<EditDisposition<T>, Denial<E>> {
    pub(super) fn resolve(
        self,
        rebuild: impl FnOnce() -> TreeAttempt<T, Denial<E>>,
    ) -> ReportedTree<T, E> {
        let (outcome, report) = match self.outcome {
            Ok(EditDisposition::Completed(tree)) => (Ok(tree), Run::Edited(self.work)),
            Err(denial) => (Err(denial), Run::Edited(self.work)),
            Ok(EditDisposition::Rebuild(cause)) => {
                let rebuilt = rebuild();
                (
                    rebuilt.outcome,
                    Run::Rebuilt(cause, self.work.followed_by(rebuilt.work)),
                )
            }
        };
        ReportedTree { outcome, report }
    }
}

impl<T, E> TreeAttempt<T, E> {
    /// Preparation has not entered any tree operation yet.
    pub(super) fn unstarted(
        _beginning: super::tree_update::TreeBeginning,
        outcome: Result<T, E>,
    ) -> Self {
        Self {
            outcome,
            work: Metrics::from_native(ReductionMetrics::default()),
        }
    }

    pub(super) fn map_result<U, F>(
        self,
        convert: impl FnOnce(Result<T, E>) -> Result<U, F>,
    ) -> TreeAttempt<U, F> {
        TreeAttempt {
            outcome: convert(self.outcome),
            work: self.work,
        }
    }

    pub(super) fn and_then<U>(
        self,
        next: impl FnOnce(T) -> TreeAttempt<U, E>,
    ) -> TreeAttempt<U, E> {
        match self.outcome {
            Ok(value) => {
                let next = next(value);
                TreeAttempt {
                    outcome: next.outcome,
                    work: self.work.followed_by(next.work),
                }
            }
            Err(error) => TreeAttempt {
                outcome: Err(error),
                work: self.work,
            },
        }
    }

    pub(super) fn outcome(&self) -> &Result<T, E> {
        &self.outcome
    }
}

impl<T, E> TreeAttempt<T, ReductionRunFailure<E>> {
    pub(super) fn native(outcome: Result<(T, ReductionMetrics), ReductionRunFailure<E>>) -> Self {
        match outcome {
            Ok((value, work)) => Self {
                outcome: Ok(value),
                work: Metrics::from_native(work),
            },
            Err(failure) => Self {
                work: Metrics::from_native(failure.metrics),
                outcome: Err(failure),
            },
        }
    }
}

impl<T> TreeAttempt<T, ReductionRunFailure<worth_execution::MapKernelStop>> {
    pub(super) fn finish_edits<E>(self) -> TreeAttempt<EditDisposition<T>, Denial<E>> {
        self.map_result(|outcome| match outcome {
            Ok(tree) => Ok(EditDisposition::Completed(tree)),
            Err(failure) => match super::tree_report::rebuild_cause(failure.reason) {
                Err(stop) => Err(Denial::from_kernel_stop(stop)),
                Ok(cause) => Ok(EditDisposition::Rebuild(cause)),
            },
        })
    }
}
