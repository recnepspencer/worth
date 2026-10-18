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
        if owner == Some(NestedStopOwner::Request) {
            self.owner = owner;
        }
        if self.stop.is_some() {
            self.discarded = self.discarded.saturating_add(outcome.work);
            return None;
        }
        let next = self.work.checked_add(outcome.work);
        if next.is_none_or(|sum| sum > limits.ceiling()) {
            if self.owner != Some(NestedStopOwner::Request) {
                self.owner = Some(if next.is_none() {
                    NestedStopOwner::Request
                } else {
                    limits.work_stop_owner()
                });
            }
            self.stop = Some(BatchStop::WorkExhausted { identity: id });
            self.boundary = Some(id);
            self.discarded = self.discarded.saturating_add(outcome.work);
            return None;
        }
        self.work += outcome.work;
        self.span = self.span.max(outcome.span);
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
