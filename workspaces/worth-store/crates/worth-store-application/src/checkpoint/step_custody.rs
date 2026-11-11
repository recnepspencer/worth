//! Before-spawn registration, original input lifetime, and terminal delivery.
use super::owner::HomeCheckpointAttempt;
use std::sync::{Arc, Weak};
use worth_store::physical_runtime::*;

pub(crate) type StepObserver<'a> = dyn Fn(&AnchoredRecordStepRegistration) + Send + Sync + 'a;

pub(super) struct HomeCheckpointStepPort<'a> {
    attempt: Weak<HomeCheckpointAttempt>,
    observer: Option<&'a StepObserver<'a>>,
}

struct HomeCheckpointStepProtection {
    _selection: worth_store_relational::RelationalCheckpointSelectionProtection,
    _input_memory: Arc<super::input_memory::CheckpointInputMemory>,
}
impl AnchoredRecordSelectionProtection for HomeCheckpointStepProtection {}

struct HomeCheckpointTerminalDestination {
    attempt: Weak<HomeCheckpointAttempt>,
}

impl<'a> HomeCheckpointStepPort<'a> {
    pub(super) fn new(
        attempt: &Arc<HomeCheckpointAttempt>,
        observer: Option<&'a StepObserver<'a>>,
    ) -> Self {
        Self {
            attempt: Arc::downgrade(attempt),
            observer,
        }
    }
}

impl AnchoredRecordStepCustodyPort for HomeCheckpointStepPort<'_> {
    fn retain_step(
        &self,
        registration: AnchoredRecordStepRegistration,
    ) -> Result<AnchoredRecordStepRetention, AnchoredRecordStepCustodyRefusal> {
        let attempt = self
            .attempt
            .upgrade()
            .ok_or(AnchoredRecordStepCustodyRefusal::OwnerUnavailable)?;
        let native = attempt
            .native()
            .ok_or(AnchoredRecordStepCustodyRefusal::SelectionUnavailable)?;
        let memory = attempt
            .input_memory
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .clone()
            .ok_or(AnchoredRecordStepCustodyRefusal::OwnerUnavailable)?;
        let retention = AnchoredRecordStepRetention::new(
            Box::new(HomeCheckpointStepProtection {
                _selection: native.selection_protection(),
                _input_memory: memory,
            }),
            Arc::new(HomeCheckpointTerminalDestination {
                attempt: self.attempt.clone(),
            }),
        );
        let registration = Arc::new(registration);
        {
            let mut physical = attempt
                .physical
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            physical.registration = Some(registration.clone());
        }
        if let Some(observer) = self.observer {
            observer(&registration);
        }
        Ok(retention)
    }
}

impl AnchoredRecordStepTerminalDestination for HomeCheckpointTerminalDestination {
    fn record_terminal(
        &self,
        terminal: &AnchoredRecordStepTerminal,
    ) -> Result<(), AnchoredRecordStepDeliveryFailure> {
        let attempt = self
            .attempt
            .upgrade()
            .ok_or(AnchoredRecordStepDeliveryFailure::OwnerUnavailable)?;
        let physical = attempt
            .physical
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let registration = physical
            .registration
            .as_ref()
            .ok_or(AnchoredRecordStepDeliveryFailure::ReceiptRefused)?;
        if registration.mutation_identity() != terminal.mutation_identity() {
            return Err(AnchoredRecordStepDeliveryFailure::ReceiptRefused);
        }
        Ok(())
    }
}
