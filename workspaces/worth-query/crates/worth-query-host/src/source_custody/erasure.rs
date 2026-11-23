//! Heterogeneous command custody retains the original typed owner inside Host.
use super::*;
use std::{any::Any, fmt};
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitReceipt, WorthQueryRecoveredApplicationCommitReceipt,
};

/// One command from an application's installed program, with its action type hidden.
#[must_use = "resume the original command or explicitly drop its custody"]
pub struct WorthQueryAnySourceCustody<S, P>(Box<dyn ErasedCustody<S, P>>);

/// The same native outcome the typed custody runner produced.
#[must_use = "a pending step owns the original command"]
pub enum WorthQueryAnySourceCustodyStep<S, P> {
    Committed {
        closed: WorthQueryClosedOperation,
        evidence: WorthQueryAnySourceCompletion,
    },
    Declined {
        closed: WorthQueryClosedOperation,
        evidence: WorthQueryAnyTerminalSourceEvidence,
    },
    Pending {
        custody: WorthQueryAnySourceCustody<S, P>,
        cause: WorthQuerySourceCustodyPending,
    },
}

impl<S, P> WorthQueryAnySourceCustody<S, P> {
    pub fn resume(
        self,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryAnySourceCustodyStep<S, P> {
        self.0.resume(runtime, request)
    }
}

impl<S, I, P, A> WorthQuerySourceCustody<S, I, P, A>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    I: WorthQuerySourceMutation<S> + 'static,
    A: WorthQuerySourceCompletionAction<S, I::Binding> + 'static,
    WorthQueryTerminalSourceEvidence<S, I>: fmt::Debug,
{
    /// Stores different installed actions behind one move-only command type.
    pub fn erase(self) -> WorthQueryAnySourceCustody<S, P> {
        WorthQueryAnySourceCustody(Box::new(Some(self)))
    }
}

trait ErasedCustody<S, P> {
    fn resume(
        self: Box<Self>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryAnySourceCustodyStep<S, P>;
}

impl<S, I, P, A> ErasedCustody<S, P> for Option<WorthQuerySourceCustody<S, I, P, A>>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    I: WorthQuerySourceMutation<S> + 'static,
    A: WorthQuerySourceCompletionAction<S, I::Binding> + 'static,
    WorthQueryTerminalSourceEvidence<S, I>: fmt::Debug,
{
    fn resume(
        mut self: Box<Self>,
        runtime: &WorthQueryProgramApplicationRuntime<S, P>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryAnySourceCustodyStep<S, P> {
        use WorthQueryAnySourceCustodyStep as Step;
        match self
            .take()
            .expect("erased custody owns one command")
            .resume(runtime, request)
        {
            WorthQuerySourceCustodyStep::Committed { closed, evidence } => Step::Committed {
                closed,
                evidence: WorthQueryAnySourceCompletion(Box::new(evidence)),
            },
            WorthQuerySourceCustodyStep::Declined { closed, evidence } => Step::Declined {
                closed,
                evidence: WorthQueryAnyTerminalSourceEvidence(Box::new(evidence)),
            },
            WorthQuerySourceCustodyStep::Pending { custody, cause } => {
                *self = Some(custody);
                Step::Pending {
                    custody: WorthQueryAnySourceCustody(self),
                    cause,
                }
            }
        }
    }
}

/// Owned completion with the original result and receipt provenance intact.
pub struct WorthQueryAnySourceCompletion(Box<dyn ErasedCompletion>);

impl WorthQueryAnySourceCompletion {
    pub fn evidence<T: 'static>(&self) -> Option<&WorthQuerySourceCompletion<T>> {
        (self.0.as_ref() as &dyn Any).downcast_ref()
    }

    pub fn live_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        self.0.live_receipt()
    }

    pub fn recorded_receipt(&self) -> Option<&WorthQueryRecoveredApplicationCommitReceipt> {
        self.0.recorded_receipt()
    }
}

trait ErasedCompletion: Any {
    fn live_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt>;
    fn recorded_receipt(&self) -> Option<&WorthQueryRecoveredApplicationCommitReceipt>;
}

impl<T: 'static> ErasedCompletion for WorthQuerySourceCompletion<T> {
    fn live_receipt(&self) -> Option<&WorthQueryApplicationCommitReceipt> {
        WorthQuerySourceCompletion::live_receipt(self)
    }

    fn recorded_receipt(&self) -> Option<&WorthQueryRecoveredApplicationCommitReceipt> {
        WorthQuerySourceCompletion::recorded_receipt(self)
    }
}

/// Owned terminal evidence with the original typed refusal available by downcast.
#[derive(Debug)]
pub struct WorthQueryAnyTerminalSourceEvidence(Box<dyn ErasedTerminal>);

impl WorthQueryAnyTerminalSourceEvidence {
    pub fn evidence<S: ApplicationSchema, I: ApplicationMutationIntent<S> + 'static>(
        &self,
    ) -> Option<&WorthQueryTerminalSourceEvidence<S, I>> {
        (self.0.as_ref() as &dyn Any).downcast_ref()
    }
}

trait ErasedTerminal: Any + fmt::Debug {}
impl<T: Any + fmt::Debug> ErasedTerminal for T {}
