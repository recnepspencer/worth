//! Retained target authority crosses decision/candidate without a second lookup.
use super::super::{
    application_contribution::QueryMemoryReservation, WorthQueryInvariantMutationTarget,
};

pub struct PreparedBodyPathWrites<S> {
    pub targets: Vec<WorthQueryInvariantMutationTarget<S, ()>>,
    pub _memory: Option<QueryMemoryReservation>,
}
