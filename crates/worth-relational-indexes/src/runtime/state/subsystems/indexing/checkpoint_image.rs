//! Authenticated generation rows and the original funding retained with them.
pub mod admission;
pub mod captured_inputs;

use std::sync::Arc;
use worth_execution::{ExecutionArray, ExecutionMemoryReservation};
use worth_relational_branch::internal_port::indexes::data::DerivedIndexGeneration;
use worth_relational_contracts::internal_port::indexes::data::basis_checkpoint::IndexBasisCheckpoint;

/// Fixed SDK storage and the reservation that admitted its exact layout.
pub struct IndexArray<T> {
    pub values: ExecutionArray<T>,
    pub _funding: Option<ExecutionMemoryReservation>,
}
impl<T> std::ops::Deref for IndexArray<T> {
    type Target = [T];
    fn deref(&self) -> &[T] {
        &self.values
    }
}
pub struct CapturedCheckpointGenerations {
    pub generations: IndexArray<Arc<DerivedIndexGeneration>>,
    /// Exact-root selections the image carried beside its generations.
    pub bases: Vec<IndexBasisCheckpoint>,
    pub _funding: IndexArray<Option<ExecutionMemoryReservation>>,
}
