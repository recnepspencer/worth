use std::sync::{Arc, Mutex};

/// Explicit attempt-local operation posture. It keeps lifecycle state separate
/// from identity, component ports, and the product registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeWorldOperationState {
    Idle,
    Preparing,
    Executing,
    Publishing,
    Recovering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeWorldOperationTransitionDenial {
    expected: RuntimeWorldOperationState,
    actual: RuntimeWorldOperationState,
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeWorldOperationLedgerState {
    pub(crate) active: usize,
    pub(crate) recovery_active: usize,
}

/// Close-admission ledger for independent attempt-local operation phases.
/// It counts live attempts without serializing their owner work or product CAS.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeWorldOperationLedger {
    pub(crate) state: Arc<Mutex<RuntimeWorldOperationLedgerState>>,
}

impl RuntimeWorldOperationLedger {
    pub(in crate::lifecycle::owner) fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(RuntimeWorldOperationLedgerState::default())),
        }
    }

    pub(super) fn preparing_reservation(&self) -> RuntimeWorldOperationReservation {
        self.reservation(RuntimeWorldOperationState::Preparing)
    }

    pub(super) fn recovering_reservation(&self) -> RuntimeWorldOperationReservation {
        self.reservation(RuntimeWorldOperationState::Recovering)
    }

    fn reservation(
        &self,
        initial_state: RuntimeWorldOperationState,
    ) -> RuntimeWorldOperationReservation {
        RuntimeWorldOperationReservation {
            state: Arc::new(Mutex::new(initial_state)),
            ledger: self.clone(),
            armed: true,
        }
    }

    #[cfg(test)]
    pub(crate) fn active(&self) -> usize {
        self.state
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .active
    }
}

/// Attempt-local operation custody. It keeps close from racing a live attempt
/// and releases exactly one shared close-admission ledger entry on every exit.
#[must_use = "a live owner operation must remain held by its attempt"]
pub(crate) struct RuntimeWorldOperationReservation {
    state: Arc<Mutex<RuntimeWorldOperationState>>,
    ledger: RuntimeWorldOperationLedger,
    armed: bool,
}

impl std::fmt::Debug for RuntimeWorldOperationReservation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeWorldOperationReservation")
            .field("armed", &self.armed)
            .finish_non_exhaustive()
    }
}

impl Drop for RuntimeWorldOperationReservation {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        assert_ne!(
            *state,
            RuntimeWorldOperationState::Idle,
            "a live operation reservation cannot be dropped while Idle"
        );
        let recovering = *state == RuntimeWorldOperationState::Recovering;
        *state = RuntimeWorldOperationState::Idle;
        drop(state);
        let mut ledger = self
            .ledger
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        ledger.active = ledger
            .active
            .checked_sub(1)
            .expect("a live operation reservation owns one ledger entry");
        if recovering {
            ledger.recovery_active = ledger
                .recovery_active
                .checked_sub(1)
                .expect("a recovering operation reservation owns one recovery entry");
        }
        self.armed = false;
    }
}

impl RuntimeWorldOperationReservation {
    fn transition(
        &mut self,
        expected: RuntimeWorldOperationState,
        next: RuntimeWorldOperationState,
    ) -> Result<(), RuntimeWorldOperationTransitionDenial> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if *state != expected {
            return Err(RuntimeWorldOperationTransitionDenial {
                expected,
                actual: *state,
            });
        }
        *state = next;
        Ok(())
    }

    pub(crate) fn begin_owner_execution(
        &mut self,
    ) -> Result<(), RuntimeWorldOperationTransitionDenial> {
        self.transition(
            RuntimeWorldOperationState::Preparing,
            RuntimeWorldOperationState::Executing,
        )
    }

    pub(crate) fn begin_publication(
        &mut self,
    ) -> Result<(), RuntimeWorldOperationTransitionDenial> {
        self.transition(
            RuntimeWorldOperationState::Executing,
            RuntimeWorldOperationState::Publishing,
        )
    }

    pub(crate) fn begin_recovery(&mut self) -> Result<(), RuntimeWorldOperationTransitionDenial> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if *state != RuntimeWorldOperationState::Publishing {
            return Err(RuntimeWorldOperationTransitionDenial {
                expected: RuntimeWorldOperationState::Publishing,
                actual: *state,
            });
        }
        let mut ledger = self
            .ledger
            .state
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        ledger.recovery_active =
            ledger
                .recovery_active
                .checked_add(1)
                .ok_or(RuntimeWorldOperationTransitionDenial {
                    expected: RuntimeWorldOperationState::Publishing,
                    actual: *state,
                })?;
        *state = RuntimeWorldOperationState::Recovering;
        Ok(())
    }
}
