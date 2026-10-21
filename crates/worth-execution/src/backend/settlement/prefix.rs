//! Canonical prefix accumulation shared by ordinary and retained native runs.
use super::super::{
    meter::{KernelFailure, KernelStop, NestedStopOwner, RunLimits},
    port::{BatchStop, TaskOutcome},
};
use worth_foundational::PartitionIdentity;

pub(super) struct Prefix<R, E> {
    pub(super) values: Vec<R>,
    pub(super) work: u64,
    pub(super) span: u64,
    pub(super) discarded: u64,
    pub(super) stop: Option<BatchStop<E>>,
    pub(super) boundary: Option<PartitionIdentity>,
    pub(super) owner: Option<NestedStopOwner>,
}
impl<R, E> Prefix<R, E> {
    pub(super) fn new(capacity: usize) -> Self {
        Self {
            values: Vec::with_capacity(capacity),
            work: 0,
            span: 0,
            discarded: 0,
            stop: None,
            boundary: None,
            owner: None,
        }
    }
    pub(super) fn accept<V>(
        &mut self,
        id: PartitionIdentity,
        outcome: TaskOutcome<V, E>,
        limits: &RunLimits,
    ) -> Option<V> {
        let (result, owner) = outcome.result.into_parts(limits);
        if owner == Some(NestedStopOwner::Request) || outcome.allocation_denial.is_some() {
            self.owner = Some(NestedStopOwner::Request);
        }
        if self.stop.is_some() {
            self.discarded = self.discarded.saturating_add(outcome.work);
            return None;
        }
        let next = self.work.checked_add(outcome.work);
        if next.is_none_or(|sum| limits.ceiling().is_some_and(|ceiling| sum > ceiling)) {
            if self.owner != Some(NestedStopOwner::Request) {
                self.owner = Some(if next.is_none() {
                    NestedStopOwner::Request
                } else {
                    limits.work_stop_owner()
                });
            }
            self.stop = Some(if next.is_none() {
                BatchStop::Failure {
                    identity: id,
                    cause: KernelFailure::Stop(KernelStop::WorkCounterOverflow),
                }
            } else {
                BatchStop::WorkExhausted { identity: id }
            });
            self.boundary = Some(id);
            self.discarded = self.discarded.saturating_add(outcome.work);
            return None;
        }
        self.work += outcome.work;
        self.span = self.span.max(outcome.span);
        if let Some(denial) = outcome.allocation_denial {
            self.owner = Some(NestedStopOwner::Request);
            self.stop = Some(BatchStop::Admission(denial));
            self.boundary = Some(id);
            return None;
        }
        let cause = match result {
            Ok(value) => return Some(value),
            Err(cause) => cause,
        };
        if self.owner != Some(NestedStopOwner::Request) {
            self.owner = owner;
        }
        self.stop = Some(match cause {
            KernelFailure::Stop(KernelStop::WorkCeiling) => {
                BatchStop::WorkExhausted { identity: id }
            }
            cause => BatchStop::Failure {
                identity: id,
                cause,
            },
        });
        self.boundary = Some(id);
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{backend::task_result::TaskResult, LeaseDenial};
    #[test]
    fn later_allocation_refusal_keeps_the_first_prefix_and_request_attribution() {
        let limits = RunLimits::for_run(None, true);
        for first_owner in [
            NestedStopOwner::Request,
            NestedStopOwner::Computation(
                super::super::super::meter::NestedComputationStop::DomainFailure,
            ),
        ] {
            let mut prefix = Prefix::<(), ()>::new(2);
            let first = PartitionIdentity::new(1);
            let later = PartitionIdentity::new(2);
            assert!(prefix
                .accept(
                    first,
                    TaskOutcome::<(), ()> {
                        result: TaskResult::new(Err(KernelFailure::Domain(())), Some(first_owner)),
                        allocation_denial: None,
                        work: 1,
                        span: 1,
                    },
                    &limits
                )
                .is_none());
            assert!(prefix
                .accept(
                    later,
                    TaskOutcome::<(), ()> {
                        result: TaskResult::Capacity,
                        allocation_denial: Some(LeaseDenial::ChargedBytesOverflow),
                        work: 2,
                        span: 2,
                    },
                    &limits
                )
                .is_none());
            assert_eq!(prefix.owner, Some(NestedStopOwner::Request));
            assert_eq!(prefix.boundary, Some(first));
            assert!(
                matches!(prefix.stop, Some(BatchStop::Failure { identity, cause: KernelFailure::Domain(()) }) if identity == first)
            );
            assert_eq!(prefix.work, 1);
            assert_eq!(prefix.discarded, 2);
        }
    }
}
