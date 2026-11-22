//! A prepared source becomes callable only with its actual contact custody.
use std::sync::Arc;
use crate::domain_computation::runtime_time::WorthQueryRuntimeClock;
use crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity;
use crate::domain_computation::primary_graph::{WorthQueryCommittedDispatchOutboxObservation, WorthQueryExternalDispatchAttemptOrdinal};
use crate::domain_computation::primary_graph::application_attempt::provider_execution::external_dispatch::durable_attempt::DurableDispatchContact;

pub enum DispatchAttemptCoordinate<'contact> {
    Memory(WorthQueryExternalDispatchAttemptOrdinal),
    Recorded(&'contact DurableDispatchContact<'contact>),
}

/// Move-only physical admission. A Home attempt borrows the original guard
/// that owns its durable attempt and eventual observation obligation.
pub struct WorthQueryAdmittedExternalDispatchAttempt<'contact> {
    pub committed: WorthQueryCommittedDispatchOutboxObservation,
    pub query_runtime: WorthQueryRuntimeAuthorityIdentity,
    pub coordinate: DispatchAttemptCoordinate<'contact>,
    pub clock: Arc<WorthQueryRuntimeClock>,
}
impl<'contact> WorthQueryAdmittedExternalDispatchAttempt<'contact> {
    pub fn seal_memory(
        committed: WorthQueryCommittedDispatchOutboxObservation,
        query_runtime: WorthQueryRuntimeAuthorityIdentity,
        ordinal: WorthQueryExternalDispatchAttemptOrdinal,
        clock: Arc<WorthQueryRuntimeClock>,
    ) -> Self {
        Self {
            committed,
            query_runtime,
            coordinate: DispatchAttemptCoordinate::Memory(ordinal),
            clock,
        }
    }
    pub fn seal_recorded(
        committed: WorthQueryCommittedDispatchOutboxObservation,
        query_runtime: WorthQueryRuntimeAuthorityIdentity,
        contact: &'contact DurableDispatchContact<'contact>,
        clock: Arc<WorthQueryRuntimeClock>,
    ) -> Self {
        Self {
            committed,
            query_runtime,
            coordinate: DispatchAttemptCoordinate::Recorded(contact),
            clock,
        }
    }
    pub fn ordinal_for_test(&self) -> u64 {
        match &self.coordinate {
            DispatchAttemptCoordinate::Memory(ordinal) => ordinal.value(),
            DispatchAttemptCoordinate::Recorded(contact) => {
                contact.recorded_attempt().attempt_position().get()
            }
        }
    }
}
