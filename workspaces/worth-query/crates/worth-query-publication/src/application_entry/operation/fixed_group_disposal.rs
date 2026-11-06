//! Retiring-group custody can close original required work without fulfilling it.
use super::fixed::FixedState;
use super::*;
use worth_query_execution::publication_boundary::document_history::WorthQueryDocumentHistoryGroupAborting;

enum Original<S, I, P, R, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
{
    Operation(WorthQueryFixedOperation<S, I, P, R, A>),
    Terminal(WorthQueryTerminalFixedOperation<S, I, P, R, A>),
}

/// Actual original custody; this has no source or producer advancement entrance.
pub struct WorthQueryClosingFixedGroupOperation<S, I, P, R, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
{
    original: Original<S, I, P, R, A>,
    closed: Option<WorthQueryClosedOperation>,
}

impl<S, I, P, R, A> Original<S, I, P, R, A>
where
    S: ApplicationSchema,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
{
    fn operation(&self) -> &WorthQueryFixedOperation<S, I, P, R, A> {
        match self {
            Self::Operation(operation) => operation,
            Self::Terminal(terminal) => &terminal.operation,
        }
    }
    fn operation_mut(&mut self) -> &mut WorthQueryFixedOperation<S, I, P, R, A> {
        match self {
            Self::Operation(operation) => operation,
            Self::Terminal(terminal) => &mut terminal.operation,
        }
    }
}

impl<S, I, P, R, A> WorthQueryClosingFixedGroupOperation<S, I, P, R, A>
where
    S: ApplicationSchema + 'static,
    I: ApplicationMutationIntent<S>,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
    I::Binding: WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
    A: ApplicationAction<S, I::Binding>,
    A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
{
    pub(super) fn from_operation(
        mut operation: WorthQueryFixedOperation<S, I, P, R, A>,
    ) -> Result<
        Self,
        (
            WorthQueryFixedOperation<S, I, P, R, A>,
            WorthQueryOperationCloseDenial,
        ),
    > {
        match Self::prepare_cleanup(&mut operation) {
            Ok(()) => {
                // A live partial has not transferred its original ledger.
                // TerminalFixedOperation::new already performed this move.
                if let FixedState::Unpublished(partial) = &mut operation.state {
                    operation
                        .cleanup
                        .retain_original_failures(partial.take_prior_cleanup());
                }
                Ok(Self {
                    original: Original::Operation(operation),
                    closed: None,
                })
            }
            Err(cause) => Err((operation, cause)),
        }
    }
    pub(super) fn from_terminal(
        mut terminal: WorthQueryTerminalFixedOperation<S, I, P, R, A>,
    ) -> Result<
        Self,
        (
            WorthQueryTerminalFixedOperation<S, I, P, R, A>,
            WorthQueryOperationCloseDenial,
        ),
    > {
        match Self::prepare_cleanup(&mut terminal.operation) {
            Ok(()) => Ok(Self {
                original: Original::Terminal(terminal),
                closed: None,
            }),
            Err(cause) => Err((terminal, cause)),
        }
    }
    fn prepare_cleanup(
        operation: &mut WorthQueryFixedOperation<S, I, P, R, A>,
    ) -> Result<(), WorthQueryOperationCloseDenial> {
        match &mut operation.state {
            FixedState::Unpublished(partial) => {
                partial.retain_group_disposal_cleanup(&mut operation.cleanup)?
            }
            FixedState::Blocked(blocked)
                if !super::close::terminal_outcome_can_close(blocked.outcome()) =>
            {
                return Err(WorthQueryOperationCloseDenial::NativeObligationRetained)
            }
            FixedState::NotPerformed(outcome)
                if !super::close::terminal_outcome_can_close(outcome) =>
            {
                return Err(WorthQueryOperationCloseDenial::NativeObligationRetained)
            }
            _ => {}
        }
        if let Some(performed) = operation.recovered.as_mut() {
            operation.cleanup.absorb_recovered(performed)?;
        }
        Ok(())
    }
    pub(super) fn target(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.original.operation().description.branch
    }
    pub(super) fn close(
        mut self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        aborting: Option<&WorthQueryDocumentHistoryGroupAborting>,
    ) -> Result<WorthQueryClosedOperation, (Self, WorthQueryOperationCloseDenial)> {
        use WorthQueryOperationCloseDenial as Denial;
        let runtime = self.original.operation().runtime.clone();
        if !std::ptr::eq(runtime.runtime(), request.application) || self.target() != request.branch
        {
            return Err((self, Denial::FreshRequestMismatch));
        }
        let outcome=runtime.runtime().with_application_advancement(request.scope,|phase|{
            if self.closed.is_none(){
                let operation=self.original.operation_mut();
                if !matches!(operation.state,FixedState::Initial){
                    aborting.ok_or(Denial::NativeObligationRetained)?.validate_candidate_cleanup(&phase,&worth_query_execution::publication_boundary::program_publication_access(),runtime.runtime(),operation.description.branch).map_err(Denial::HistoryGroup)?;
                }
                if let FixedState::Outputs(tree)=&mut operation.state{
                    tree.close_group_original(&phase,&runtime,operation.description.branch,aborting.ok_or(Denial::NativeObligationRetained)?)?;
                }
                self.closed=Some(WorthQueryClosedOperation::from_cleanup(operation.cleanup.close_original_in_advancement(&phase,runtime.runtime())?));
            }
            Ok(())
        });
        match outcome {
            Ok(Ok(())) => Ok(self
                .closed
                .take()
                .expect("actual original cleanup completed")),
            Ok(Err(cause)) => Err((self, cause)),
            Err(cause) => Err((self, Denial::Execution(cause))),
        }
    }
}
