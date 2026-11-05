//! Receiver-owned consumption at the actual program-source delivery handoff.
use std::sync::{Arc, Mutex, OnceLock};

use crate::domain_computation::primary_graph::application_installation::WorthQueryRecoveredProgramOutputSource;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_installation::facade::ApplicationSchema;

#[derive(Default)]
pub(in crate::domain_computation::primary_graph) struct RecoveredProgramSourceDeliveryRegistration {
    active: Mutex<Option<Arc<OnceLock<WorthQueryRecoveredProgramOutputSource>>>>,
}

/// This runtime receiver already has an owned program-source consumption arm.
#[derive(Debug, PartialEq, Eq)]
pub enum WorthQueryRecoveredProgramSourceDeliveryDenialForTest {
    AlreadyArmed,
}

/// Owns the actual first native carrier and clears its receiver registration on drop/unwind.
/// This consumption court does not represent ordinary public API reachability.
pub struct WorthQueryRecoveredProgramSourceDeliveryForTest<'receiver> {
    state: Arc<OnceLock<WorthQueryRecoveredProgramOutputSource>>,
    registration: &'receiver RecoveredProgramSourceDeliveryRegistration,
}

impl WorthQueryRecoveredProgramSourceDeliveryForTest<'_> {
    /// Borrows the genuinely consumed first delivery, never reconstructed parts.
    pub fn captured_delivery_for_test(&self) -> Option<&WorthQueryRecoveredProgramOutputSource> {
        self.state.get()
    }
}

impl Drop for WorthQueryRecoveredProgramSourceDeliveryForTest<'_> {
    fn drop(&mut self) {
        let mut active = self
            .registration
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active
            .as_ref()
            .is_some_and(|state| Arc::ptr_eq(state, &self.state))
        {
            *active = None;
        }
        // Registration and guard disposal release only the captured carrier.
        // The managed performed owner still owns its independent pending cleanup.
    }
}

pub trait RecoveredProgramSourceDeliveryOps<Schema: ApplicationSchema>:
    crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema>
{
    fn consume_next_recovered_program_source_delivery_for_test(
        &self,
    ) -> Result<
        WorthQueryRecoveredProgramSourceDeliveryForTest<'_>,
        WorthQueryRecoveredProgramSourceDeliveryDenialForTest,
    > {
        let registration = self.recovered_program_source_delivery();
        let mut active = registration
            .active
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if active.is_some() {
            return Err(WorthQueryRecoveredProgramSourceDeliveryDenialForTest::AlreadyArmed);
        }
        let state = Arc::new(OnceLock::new());
        *active = Some(Arc::clone(&state));
        Ok(WorthQueryRecoveredProgramSourceDeliveryForTest {
            state,
            registration,
        })
    }
}

impl<
        Schema: ApplicationSchema,
        HostRuntime: crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema> + ?Sized,
    > RecoveredProgramSourceDeliveryOps<Schema> for HostRuntime
{
}

pub(super) fn consume_first_if_armed<Schema: ApplicationSchema>(
    runtime: &dyn crate::domain_computation::primary_graph::provider::host::ProviderHost<Schema>,
    take: impl FnOnce() -> WorthQueryRecoveredProgramOutputSource,
) {
    let active = runtime
        .recovered_program_source_delivery()
        .active
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    if let Some(state) = active.as_ref()
        && state.get().is_none()
    {
        // The receiver registration lock serializes first consumption. The
        // immutable capture lets observation coexist with later recovery.
        let _ = state.set(take());
    }
}
