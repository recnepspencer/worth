use std::{panic::AssertUnwindSafe, vec::IntoIter};

use super::{admission::BatchDeclaration, native, panic_boundary::contain, perturbation};
use crate::{authority::ExecutionResourceLease, report::ChargedBytes};

/// Only input acquisition differs; identity, admission and settlement do not.
pub(super) trait InputMode {
    type Item: Send;
    fn memory_bytes<R, E>(&self, declaration: &BatchDeclaration) -> Option<u64>;
    fn dispatch(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        parallel: bool,
        seed: Option<u64>,
        execute: &(impl Fn((usize, Self::Item)) + Sync),
    ) -> usize;
    /// Destroy every remaining input independently; return the least panic index.
    fn discard(&mut self) -> Option<usize>;
}

pub(super) struct BorrowedInputs<'a, T>(pub(super) &'a [T]);
pub(super) struct OwnedInputs<T>(pub(super) Vec<T>);

impl<'a, T: Sync + ChargedBytes> InputMode for BorrowedInputs<'a, T> {
    type Item = &'a T;
    fn memory_bytes<R, E>(&self, declaration: &BatchDeclaration) -> Option<u64> {
        declaration.execution_memory_bytes::<T, R, E>(self.0)
    }
    fn dispatch(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        parallel: bool,
        seed: Option<u64>,
        execute: &(impl Fn((usize, Self::Item)) + Sync),
    ) -> usize {
        if !parallel {
            for index in 0..self.0.len() {
                execute((index, &self.0[index]));
            }
            return 1;
        }
        let mut order: Vec<_> = (0..self.0.len()).collect();
        if let Some(seed) = seed {
            perturbation::permute(order.len(), seed, |a, b| order.swap(a, b));
        }
        native::run_order(lease.expect("native requires lease"), &order, &|index| {
            execute((index, &self.0[index]))
        })
    }
    fn discard(&mut self) -> Option<usize> {
        None
    }
}

impl<T: Send + ChargedBytes> InputMode for OwnedInputs<T> {
    type Item = T;
    fn memory_bytes<R, E>(&self, declaration: &BatchDeclaration) -> Option<u64> {
        declaration.execution_memory_bytes::<T, R, E>(&self.0)
    }
    fn dispatch(
        &mut self,
        lease: Option<&ExecutionResourceLease<'_>>,
        parallel: bool,
        seed: Option<u64>,
        execute: &(impl Fn((usize, Self::Item)) + Sync),
    ) -> usize {
        let order = seed.map(|seed| {
            let mut order: Vec<_> = (0..self.0.len()).collect();
            perturbation::permute(order.len(), seed, |a, b| {
                order.swap(a, b);
                self.0.swap(a, b);
            });
            order
        });
        let count = self.0.len();
        let tasks = OwnedTasks {
            values: std::mem::take(&mut self.0).into_iter().enumerate(),
            order,
        };
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
        let mut first = None;
        for (index, input) in self.0.drain(..).enumerate() {
            if contain(AssertUnwindSafe(|| drop(input))).is_err() && first.is_none() {
                first = Some(index);
            }
        }
        first
    }
}

/// An exceptional dispatcher unwind must not let one destructor skip the rest.
struct OwnedTasks<T> {
    values: std::iter::Enumerate<IntoIter<T>>,
    order: Option<Vec<usize>>,
}
impl<T> Iterator for OwnedTasks<T> {
    type Item = (usize, T);
    fn next(&mut self) -> Option<Self::Item> {
        self.values.next().map(|(index, input)| {
            (
                self.order.as_ref().map_or(index, |order| order[index]),
                input,
            )
        })
    }
}
impl<T> Drop for OwnedTasks<T> {
    fn drop(&mut self) {
        for (_, input) in self.values.by_ref() {
            let _ = contain(AssertUnwindSafe(|| drop(input)));
        }
    }
}
