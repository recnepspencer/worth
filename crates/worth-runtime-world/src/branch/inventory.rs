//! A request-owned inventory over the installed World reference cells.
use super::{ProductBranchObservation, RuntimeWorldBranchAdmissionDenial};

#[derive(Debug)]
pub enum ProductBranchInventoryDenial {
    Branch(RuntimeWorldBranchAdmissionDenial),
    Request(worth_execution::WorkCeilingDenial),
    AllocationRefused,
}

/// Membership is captured once; heads are actual observations issued afterward.
/// This read does not claim an atomic cut across concurrently moving branches.
#[derive(Debug)]
pub struct ObservedProductBranches {
    pub(crate) branches: Vec<ProductBranchObservation>,
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl ObservedProductBranches {
    pub fn branches(&self) -> &[ProductBranchObservation] {
        &self.branches
    }
}
