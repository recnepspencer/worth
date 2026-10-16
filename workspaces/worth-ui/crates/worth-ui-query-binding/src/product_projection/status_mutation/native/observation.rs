use super::*;
impl<I: ApplicationMutationIntent<WorthUiApplicationSchema> + 'static, A> State<I, A> {
    pub(super) fn retained(&self) -> Option<&WorthQueryApplicationReadObservation> {
        let Self::Completed(completed) = self else {
            return None;
        };
        match completed.evidence() {
            WorthQuerySourceCompletion::Fresh { retained, .. }
            | WorthQuerySourceCompletion::Recovered { retained, .. } => retained.as_ref(),
        }
    }
}

impl NativeCustody {
    pub(in crate::product_projection::status_mutation) fn retained(
        &self,
    ) -> Option<&WorthQueryApplicationReadObservation> {
        match self {
            Self::Source(state) => state.retained(),
            Self::Action(state) => state.retained(),
        }
    }
    pub(in crate::product_projection::status_mutation) fn revision_mismatch(&self) -> bool {
        matches!(self, Self::Action(State::Terminal(terminal)) if matches!(terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::DomainDenied(crate::declaration::WorthUiStatusUpdateDenial::RevisionMismatch))))
    }
    pub(in crate::product_projection::status_mutation) fn terminal_outcome(
        &self,
    ) -> Option<WorthUiStatusTerminalOutcome<'_>> {
        match self {
            Self::Source(State::Terminal(terminal)) => terminal
                .native_outcome()
                .map(WorthUiStatusTerminalOutcome::Source),
            Self::Action(State::Terminal(terminal)) => terminal
                .native_outcome()
                .map(WorthUiStatusTerminalOutcome::Action),
            _ => None,
        }
    }
    pub(in crate::product_projection::status_mutation) fn capture_denial(
        &self,
    ) -> Option<&worth_query_host::facade::application_entry::WorthQueryOperationInputCaptureDenial>
    {
        match self {
            Self::Source(State::Capture(original)) => Some(original.denial()),
            Self::Action(State::Capture(original)) => Some(original.denial()),
            _ => None,
        }
    }
    pub(in crate::product_projection::status_mutation) fn admission_denial(
        &self,
    ) -> Option<
        &worth_query_host::facade::application_entry::WorthQueryApplicationRequestMutationDenial,
    > {
        match self {
            Self::Source(State::Terminal(terminal)) => terminal.native_admission_denial(),
            Self::Action(State::Terminal(terminal)) => terminal.native_admission_denial(),
            _ => None,
        }
    }
}
