use crate::data::error::SignalError;
use crate::data::graph::SignalGraph;
use crate::data::proof::{
    SignalInvalidationExecutionObservation, SignalInvalidationExecutionReceipt,
};
use crate::logic::transaction::{
    SignalObservationAdmissionDenial, SignalObservationCompletion, SignalObservationRequest,
    SignalObservationSession,
};

#[cfg(test)]
mod admission_tests;

impl SignalGraph {
    pub(crate) fn interrupt_observation_at_boundary(&mut self) -> bool {
        if self.observation_session_active_generation() == 0 {
            return false;
        }
        let request = self.observation_sessions.active_request();
        if !self.observation_sessions.interrupt() {
            return false;
        }
        if request.includes(crate::logic::transaction::SignalObservationSurface::PerformedCounters)
        {
            self.invalidation_performed_counter_state().reset();
        }
        if request.includes(crate::logic::transaction::SignalObservationSurface::PerformedWork) {
            self.invalidation_performed_work.reset();
        }
        self.observation_sessions
            .record_completion(SignalObservationCompletion::InterruptedByBoundary);
        true
    }

    pub(crate) fn record_boundary_interruption(&self) {
        self.observation_sessions
            .record_completion(SignalObservationCompletion::InterruptedByBoundary);
    }

    pub fn begin_observation_session(
        &mut self,
        request: SignalObservationRequest,
    ) -> Result<SignalObservationSession, SignalObservationAdmissionDenial> {
        let request = crate::logic::transaction::admit_signal_observation_request(
            request,
            self.observation_session_active_generation(),
        )?;
        if self.observation_capture_cleanup.is_none() {
            self.rebind_observation_capture_state();
        }
        self.set_default_observation_surface_mask(
            self.installed_runtime_policy()
                .observation_capture_plan()
                .default_surface_mask(),
        );
        // Prepare capture before publishing a live generation. A poisoned work
        // store must not leave an active session for which no token exists.
        if request.includes(crate::logic::transaction::SignalObservationSurface::PerformedWork) {
            self.invalidation_performed_work.reset();
        }
        let liveness = self.observation_session_liveness();
        let drop_cleanup = self
            .observation_capture_cleanup
            .as_ref()
            .expect("observation cleanup initialized")
            .clone();
        if request.includes(crate::logic::transaction::SignalObservationSurface::PerformedCounters)
        {
            self.invalidation_performed_counters.begin_capture();
        }
        let generation = self.observation_sessions.begin(request);
        self.diagnostics_state_mut()
            .record_observation_activation(request.mask());
        Ok(SignalObservationSession {
            graph_instance: self.runtime_instance_id(),
            generation,
            request,
            liveness,
            drop_cleanup,
        })
    }

    pub fn finish_observation_session(
        &self,
        observation: &SignalObservationSession,
    ) -> Result<SignalInvalidationExecutionReceipt, SignalError> {
        self.finish_optional_observation_session(observation)?
            .ok_or_else(|| {
                SignalError::invalid_input(
                    "invalidation execution observation contains no executed invalidation batch",
                )
            })
    }

    fn finish_optional_observation_session(
        &self,
        observation: &SignalObservationSession,
    ) -> Result<Option<SignalInvalidationExecutionReceipt>, SignalError> {
        self.finish_optional_observation_session_with_work(
            observation,
            &mut crate::logic::evaluation::EvaluationWork::Ordinary,
        )
    }

    pub(crate) fn finish_optional_observation_session_with_work(
        &self,
        observation: &SignalObservationSession,
        work: &mut crate::logic::evaluation::EvaluationWork<'_>,
    ) -> Result<Option<SignalInvalidationExecutionReceipt>, SignalError> {
        if observation.graph_instance() != self.runtime_instance_id() {
            return Err(SignalError::invalid_input(
                "observation session belongs to another runtime",
            ));
        }
        if observation.generation() != self.observation_session_active_generation() {
            return Err(SignalError::invalid_input(
                "observation session was superseded",
            ));
        }
        let captures_counters = observation
            .request()
            .includes(crate::logic::transaction::SignalObservationSurface::PerformedCounters);
        let captures_work = observation
            .request()
            .includes(crate::logic::transaction::SignalObservationSurface::PerformedWork);
        let counters = if captures_counters {
            self.invalidation_performed_counters()
        } else {
            crate::data::telemetry::SignalInvalidationRealizedCounters::default()
        };
        let completed_execution_boundaries = self.completed_observation_execution_boundaries();
        let captured_targets = if completed_execution_boundaries != 0 {
            Some(self.snapshot_invalidation_performed_targets(captures_work, work)?)
        } else {
            // No receipt is constructed. Cleanup must remain possible after
            // execution exhausts its work allowance, including capture faults.
            if captures_work {
                self.invalidation_performed_work.ensure_available();
            }
            None
        };
        if !self.finish_observation_generation(observation.generation()) {
            return Err(SignalError::invalid_input(
                "observation session is no longer active",
            ));
        }
        if completed_execution_boundaries == 0 {
            self.observation_sessions
                .record_completion(SignalObservationCompletion::NoExecution);
            if captures_counters {
                self.invalidation_performed_counter_state().reset();
            }
            if captures_work {
                self.invalidation_performed_work.reset();
            }
            return Ok(None);
        }
        self.observation_sessions
            .record_completion(SignalObservationCompletion::Completed);
        let (executed_targets, storage_custody) = captured_targets
            .map(|snapshot| (snapshot.targets, snapshot.custody))
            .unwrap_or_default();
        let receipt = SignalInvalidationExecutionReceipt::after_execution(
            self.runtime_instance_id(),
            counters,
            executed_targets,
            observation.request(),
            storage_custody,
        );
        if captures_counters {
            self.invalidation_performed_counter_state().reset();
        }
        if captures_work {
            self.invalidation_performed_work.reset();
        }
        Ok(Some(receipt))
    }

    pub fn cancel_observation_session(
        &self,
        observation: &SignalObservationSession,
    ) -> Result<SignalObservationCompletion, SignalError> {
        if observation.graph_instance() != self.runtime_instance_id() {
            return Err(SignalError::invalid_input(
                "observation session belongs to another runtime",
            ));
        }
        if observation.generation() != self.observation_session_active_generation() {
            return Err(SignalError::invalid_input(
                "observation session was superseded",
            ));
        }
        if !self.finish_observation_generation(observation.generation()) {
            return Err(SignalError::invalid_input(
                "observation session is no longer active",
            ));
        }
        if observation
            .request()
            .includes(crate::logic::transaction::SignalObservationSurface::PerformedCounters)
        {
            self.invalidation_performed_counter_state().reset();
        }
        if observation
            .request()
            .includes(crate::logic::transaction::SignalObservationSurface::PerformedWork)
        {
            self.invalidation_performed_work.reset();
        }
        self.observation_sessions
            .record_completion(SignalObservationCompletion::Cancelled);
        Ok(SignalObservationCompletion::Cancelled)
    }

    #[deprecated(note = "use begin_observation_session(SignalObservationRequest::operation())")]
    pub fn begin_invalidation_execution_observation(
        &mut self,
    ) -> Result<SignalInvalidationExecutionObservation, SignalObservationAdmissionDenial> {
        self.begin_observation_session(SignalObservationRequest::operation())
    }

    pub fn finish_invalidation_execution_observation(
        &self,
        observation: &SignalInvalidationExecutionObservation,
    ) -> Result<SignalInvalidationExecutionReceipt, SignalError> {
        self.finish_observation_session(observation)
    }

    pub fn finish_optional_invalidation_execution_observation(
        &self,
        observation: &SignalInvalidationExecutionObservation,
    ) -> Result<Option<SignalInvalidationExecutionReceipt>, SignalError> {
        self.finish_optional_observation_session(observation)
    }

    pub fn observe_invalidation_execution<Outcome>(
        &mut self,
        execute: impl FnOnce(&mut Self) -> Result<Outcome, SignalError>,
    ) -> Result<(Outcome, SignalInvalidationExecutionReceipt), SignalError> {
        self.observe_execution(SignalObservationRequest::operation(), execute)
    }

    pub fn observe_execution<Outcome>(
        &mut self,
        request: SignalObservationRequest,
        execute: impl FnOnce(&mut Self) -> Result<Outcome, SignalError>,
    ) -> Result<(Outcome, SignalInvalidationExecutionReceipt), SignalError> {
        let observation = self
            .begin_observation_session(request)
            .map_err(|denial| SignalError::invalid_input(denial.to_string()))?;
        let outcome = match execute(self) {
            Ok(outcome) => outcome,
            Err(error) => {
                let _ = self.cancel_observation_session(&observation);
                return Err(error);
            }
        };
        let receipt = self.finish_observation_session(&observation)?;
        Ok((outcome, receipt))
    }
}

impl<D, I, E, Ctx, T> crate::logic::transaction::SignalRuntime<D, I, E, Ctx, T>
where
    D: Copy + Ord + std::fmt::Debug + 'static,
    I: Copy + Ord,
    T: Copy + Ord,
{
    pub fn begin_invalidation_execution_observation(
        &mut self,
    ) -> Result<SignalInvalidationExecutionObservation, SignalObservationAdmissionDenial> {
        self.graph_mut()
            .begin_observation_session(SignalObservationRequest::operation())
    }

    pub fn finish_observation_session(
        &self,
        observation: &SignalObservationSession,
    ) -> Result<SignalInvalidationExecutionReceipt, SignalError> {
        self.graph().finish_observation_session(observation)
    }

    pub fn cancel_observation_session(
        &self,
        observation: &SignalObservationSession,
    ) -> Result<SignalObservationCompletion, SignalError> {
        self.graph().cancel_observation_session(observation)
    }

    pub fn begin_observation_session(
        &mut self,
        request: SignalObservationRequest,
    ) -> Result<SignalObservationSession, SignalObservationAdmissionDenial> {
        self.graph_mut().begin_observation_session(request)
    }

    pub fn finish_invalidation_execution_observation(
        &self,
        observation: &SignalInvalidationExecutionObservation,
    ) -> Result<SignalInvalidationExecutionReceipt, SignalError> {
        self.graph().finish_observation_session(observation)
    }

    pub fn observe_invalidation_execution<Outcome>(
        &mut self,
        execute: impl FnOnce(&mut Self) -> Result<Outcome, SignalError>,
    ) -> Result<(Outcome, SignalInvalidationExecutionReceipt), SignalError> {
        self.observe_execution(SignalObservationRequest::operation(), execute)
    }

    pub fn observe_execution<Outcome>(
        &mut self,
        request: SignalObservationRequest,
        execute: impl FnOnce(&mut Self) -> Result<Outcome, SignalError>,
    ) -> Result<(Outcome, SignalInvalidationExecutionReceipt), SignalError> {
        let observation = self
            .begin_observation_session(request)
            .map_err(|denial| SignalError::invalid_input(denial.to_string()))?;
        let outcome = match execute(self) {
            Ok(outcome) => outcome,
            Err(error) => {
                let _ = self.cancel_observation_session(&observation);
                return Err(error);
            }
        };
        let receipt = self.graph().finish_observation_session(&observation)?;
        Ok((outcome, receipt))
    }
}
