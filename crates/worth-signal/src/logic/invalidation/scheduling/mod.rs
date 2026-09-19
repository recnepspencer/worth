//! Mechanical lifecycle for already-admitted invalidation work.

mod deduplication;
mod execution;
mod lowering;
mod queue;
mod readiness;

#[cfg(test)]
pub(crate) use deduplication::merge_repeated_current_admission;
pub(crate) use execution::{execute_ready, execute_ready_with_work};
pub(crate) use lowering::lower_current_work;
pub(crate) use queue::{ReadyInvalidationQueue, ReadyQueueEntry};
pub(crate) use readiness::admit_current_readiness;

#[cfg(test)]
mod tests;
