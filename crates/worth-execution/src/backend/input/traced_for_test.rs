//! Records acquisition inside the owned queue lock, before kernel scheduling.
use super::*;
use std::sync::Mutex;
pub(in crate::backend) struct TracedOwnedInputsForTest<'a, T> {
    pub values: OwnedInputs<T>,
    pub order: &'a Mutex<Vec<usize>>,
}
impl<T: Send> InputMode for TracedOwnedInputsForTest<'_, T> {
    type Item = T;
    type Payload = T;
    fn memory_bytes<R, E>(
        &self,
        declaration: &BatchDeclaration,
        measure: Option<fn(&T) -> u64>,
    ) -> Option<u64> {
        self.values.memory_bytes::<R, E>(declaration, measure)
    }
    fn dispatch(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        parallel: bool,
        seed: Option<u64>,
        execute: &(impl Fn((usize, T)) + Sync),
    ) -> usize {
        let count = self.values.0.len();
        let order = self.order;
        let tasks = self
            .values
            .tasks(seed)
            .inspect(move |(rank, _)| order.lock().unwrap().push(*rank));
        if parallel {
            native::run_tasks(lease.expect("native requires lease"), count, tasks, execute)
        } else {
            for task in tasks {
                execute(task);
            }
            1
        }
    }
    fn discard(&mut self) -> Option<usize> {
        self.values.discard()
    }
}
