//! Concrete pure refusals keep their type and original-request wrapper custody.
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use {
    crate::domain_computation::vocabulary::primary_graph::control::WorthQueryManagedIndexControl,
    crate::domain_computation::vocabulary::primary_graph::stop::WorthQueryManagedIndexStop,
};

#[cfg(test)]
mod tests;
