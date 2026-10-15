//! Cleanup composes the existing collection; unresolved native obligations never become Closed.
use super::*;
impl<S, I, P, R, A> WorthQueryDiscoveredOperation<S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    /// Closing Initial custody abandons unexecuted meaning; active native work stays owned.
    pub fn close(
        self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err((self, WorthQueryOperationCloseDenial::FreshRequestMismatch));
        }
        if !matches!(&self.state, State::Initial) {
            return Err((
                self,
                WorthQueryOperationCloseDenial::NativeObligationRetained,
            ));
        }
        let terminal = WorthQueryTerminalDiscoveredOperation { operation: self };
        terminal
            .close(request)
            .map_err(|(terminal, denial)| (terminal.operation, denial))
    }
    /// Retries only issuer-owned cleanup; it does not complete source or output work.
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err(WorthQueryOperationCloseDenial::FreshRequestMismatch);
        }
        request.application
            .with_application_advancement(request.scope, |phase| self.release_in_phase(&phase))
            .map_err(WorthQueryOperationCloseDenial::Execution)?
    }
    pub(super) fn release_in_phase(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        while !self.prior_cleanup.is_empty() {
            self.cleanup.reserve_release_failure()?;
            self.cleanup
                .absorb_release_failure(self.prior_cleanup.pop());
        }
        if let Some(performed) = self.recovered.as_mut() {
            self.cleanup.absorb_recovered(performed)?;
        }
        self.cleanup
            .close_in_advancement(phase, self.runtime.runtime())
    }
}
impl<S, I, P, R, A> WorthQuerySettledDiscoveredOperation<S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.description.branch
    }
    pub fn declared_action(&self) -> &ApplicationActionDeclaration {
        &self.installed_action
    }
    /// The native aggregate, including retired roots and every dependent settlement.
    /// This observation alone makes no currentness claim.
    pub fn settlement(
        &self,
    ) -> &WorthQueryDiscoveredProgramOutputSettlement<RootQuery<S, R>, RootDemand<S, R>> {
        &self.settlement
    }
    pub fn handler_result(&self) -> Option<&<I::Binding as ApplicationMutationBinding<S>>::Result> {
        self.result.as_ref()
    }
    pub fn recovered_progress(&self) -> Option<&WorthQueryManagedApplicationRecoveryPerformed> {
        self.recovered.as_ref()
    }
    pub fn initial_cause(
        &self,
    ) -> Option<worth_query_execution::facade::runtime::ProductUnpublishedCause> {
        self.initial_cause
    }
    /// Releases actual cleanup under one admitted phase; refusal returns the entire settlement.
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err((self, WorthQueryOperationCloseDenial::FreshRequestMismatch));
        }
        let runtime = request.application;
        let closed = runtime.with_application_advancement(request.scope, |phase| {
            while !self.prior_cleanup.is_empty() {
                self.cleanup.reserve_release_failure()?;
                self.cleanup
                    .absorb_release_failure(self.prior_cleanup.pop());
            }
            if let Some(performed) = self.recovered.as_mut() {
                self.cleanup.absorb_recovered(performed)?;
            }
            self.cleanup
                .close_in_advancement(&phase, runtime)
                .map(WorthQueryClosedOperation::from_cleanup)
        });
        match closed {
            Ok(Ok(closed)) => Ok(closed),
            Ok(Err(denial)) => Err((self, denial)),
            Err(denial) => Err((self, WorthQueryOperationCloseDenial::Execution(denial))),
        }
    }
}
impl<S, I, P, R, A> WorthQueryTerminalDiscoveredOperation<S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationDiscoveredOutputRoot,
    Connection<S, R>: WorthQueryApplicationDiscoveredOutputConnection<S, Source = I::Binding>,
    A: ApplicationAction<S, I::Binding>,
    ActionDelivery<S, I, A>: ApplicationDiscoveredActionDelivery<S, I::Binding, Root = R>,
{
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.operation.branch()
    }
    pub fn native_outcome(&self) -> Option<&NativeOutcome<S, I>> {
        match &self.operation.state {
            State::NotPerformed(outcome) => Some(outcome),
            State::Blocked(blocked) => Some(blocked.outcome()),
            _ => None,
        }
    }
    pub fn handler_result(&self) -> Option<&<I::Binding as ApplicationMutationBinding<S>>::Result> {
        self.operation.handler_result()
    }
    pub fn recovered_progress(&self) -> Option<&WorthQueryManagedApplicationRecoveryPerformed> {
        self.operation.recovered_progress()
    }
    /// Discharge only the shared collection; terminal source/output obligations stay owned.
    pub fn release_cleanup(
        &mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryOperationCleanupReceipt, WorthQueryOperationCloseDenial> {
        self.operation.release_cleanup(request)
    }
    /// No-effect or historical receipt custody can close; unresolved native owners cannot.
    pub fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        if !std::ptr::eq(self.operation.runtime.runtime(), request.application)
            || self.operation.description.branch != request.branch
        {
            return Err((self, WorthQueryOperationCloseDenial::FreshRequestMismatch));
        }
        let closable = match &self.operation.state {
            State::Initial => true,
            State::NotPerformed(outcome) => {
                super::super::close::terminal_outcome_can_close(outcome)
            }
            _ => false,
        };
        let runtime = request.application;
        let result = runtime.with_application_advancement(request.scope, |phase| {
            let terminal_carrier = match &mut self.operation.state {
                State::Unpublished(partial)
                    if partial.recovery_posture()
                        == WorthQueryRequiredOutputRecoveryPosture::Terminal =>
                {
                    let Some(performed) = partial.performed_mut() else {
                        return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
                    };
                    self.operation.cleanup.absorb_recovered(performed)?;
                    true
                }
                _ => false,
            };
            if !closable && !terminal_carrier {
                return Err(WorthQueryOperationCloseDenial::NativeObligationRetained);
            }
            self.operation
                .release_in_phase(&phase)
                .map(WorthQueryClosedOperation::from_cleanup)
        });
        match result {
            Ok(Ok(closed)) => Ok(closed),
            Ok(Err(denial)) => Err((self, denial)),
            Err(denial) => Err((self, WorthQueryOperationCloseDenial::Execution(denial))),
        }
    }
}
