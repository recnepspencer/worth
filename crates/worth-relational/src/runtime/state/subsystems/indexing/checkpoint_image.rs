//! Authenticated generation rows and the original funding retained with them.
pub(crate) mod admission;
pub(crate) mod captured_inputs;

use crate::indexes::data::{DerivedIndexGeneration, IndexBasisCheckpoint};
use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionMemoryReservation};

/// Fixed SDK storage and the reservation that admitted its exact layout.
pub(crate) struct IndexArray<T> {
    pub(crate) values: ExecutionArray<T>,
    pub(crate) _funding: Option<ExecutionMemoryReservation>,
}
impl<T> std::ops::Deref for IndexArray<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
pub(crate) struct CapturedCheckpointGenerations {
    pub(crate) generations: IndexArray<Arc<DerivedIndexGeneration>>,
    /// Exact-root selections the image carried beside its generations.
    pub(crate) bases: Vec<IndexBasisCheckpoint>,
    pub(crate) _funding: IndexArray<Option<ExecutionMemoryReservation>>,
}
