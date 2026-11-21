//! Existing anchors are observed once and carried into candidate assembly.
use super::super::{
    application_contribution::QueryMemoryReservation, WorthQueryInvariantMutationTarget,
};
pub struct PreparedBodyPathLinks<S> {
    pub targets:
        Vec<WorthQueryInvariantMutationTarget<S, ()>>,
    pub _memory: Option<QueryMemoryReservation>,
}
