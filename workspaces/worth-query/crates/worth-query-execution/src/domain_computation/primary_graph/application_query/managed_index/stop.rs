//! Actual request and storage causes remain distinct from pure provider refusals.
use worth_execution::{ExecutionRequestArrayDenial, WorkCeilingDenial};
use crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop;


/// A provider's original typed refusal, or an original controlled-work stop.
#[derive(Debug)]
pub enum WorthQueryManagedIndexProviderStop<Domain> {
    Controlled(WorthQueryManagedIndexStop),
    Domain(Domain),
}

impl<D> From<WorthQueryManagedIndexStop> for WorthQueryManagedIndexProviderStop<D> {
    fn from(cause: WorthQueryManagedIndexStop) -> Self {
        Self::Controlled(cause)
    }
}

