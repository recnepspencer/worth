//! Prepared corrections enter the same Host source-custody spine as ordinary commands.
use super::*;
use std::fmt;
use worth_query_declaration::facade::application_operation::ApplicationMutationInputScope;
use worth_query_publication::facade::application_entry::{
    WorthQueryApplicationCorrectionDenial, WorthQueryCapturedMutationInput,
    WorthQueryOperationCorrectionPreparation, WorthQuerySourceMutationBinding,
};

/// Outcome of admitting a prepared reversal or reapplication into Host source custody.
///
/// Once published or continued into native source custody, the correction runs through
/// [`WorthQuerySourceCustodyStep`] and can be erased into [`WorthQueryAnySourceCustodyStep`].
#[must_use = "handle the custody step or explicit correction outcome"]
pub enum WorthQueryCorrectionCustodyOutcome<S, B, P, A>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: ApplicationMutationInputScope<S, B>,
    P: ApplicationProgramDefinition<S>,
{
    /// The correction entered source custody and produced its first step.
    Custody(WorthQuerySourceCustodyStep<S, WorthQueryCapturedMutationInput<S, B>, P, A>),
    /// Publication of the prepared candidate was refused before entering source custody.
    PublicationRefused(WorthQueryApplicationCorrectionDenial),
    /// The client idempotency key already belongs to a different intent.
    IdempotencyIntentDrift,
    /// Publication for this correction key is already in flight.
    PublicationPending,
    /// Prior publication left unpublished product recovery custody.
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}

/// Type-erased counterpart of [`WorthQueryCorrectionCustodyOutcome`].
#[must_use = "handle the erased custody step or explicit correction outcome"]
pub enum WorthQueryAnyCorrectionCustodyOutcome<S, P> {
    Custody(WorthQueryAnySourceCustodyStep<S, P>),
    PublicationRefused(WorthQueryApplicationCorrectionDenial),
    IdempotencyIntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}

impl<S, B, P, A> WorthQuerySourceCustody<S, WorthQueryCapturedMutationInput<S, B>, P, A>
where
    S: ApplicationSchema + 'static,
    B: WorthQuerySourceMutationBinding<S>,
    B::Input: ApplicationMutationInputScope<S, B>,
    P: ApplicationProgramDefinition<S>,
    A: WorthQuerySourceCompletionAction<S, B>,
{
    /// Publishes or continues a prepared operation correction and transfers its native source
    /// owner into [`WorthQuerySourceCustody`], closing completed or terminal custody under
    /// `request`.
    pub fn accept_correction(
        prepared: WorthQueryOperationCorrectionPreparation<'_, S, B, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryCorrectionCustodyOutcome<S, B, P, A> {
        use WorthQueryCorrectionCustodyOutcome as Outcome;
        let advance = match prepared {
            WorthQueryOperationCorrectionPreparation::Prepared(prepared) => {
                match prepared.publish() {
                    Ok(advance) => advance,
                    Err(denial) => return Outcome::PublicationRefused(denial),
                }
            }
            WorthQueryOperationCorrectionPreparation::Continued(advance) => advance,
            WorthQueryOperationCorrectionPreparation::IdempotencyIntentDrift => {
                return Outcome::IdempotencyIntentDrift
            }
            WorthQueryOperationCorrectionPreparation::PublicationPending => {
                return Outcome::PublicationPending
            }
            WorthQueryOperationCorrectionPreparation::ProductUnpublished(handle) => {
                return Outcome::ProductUnpublished(handle)
            }
        };
        Outcome::Custody(Self::accept_advance(advance, request))
    }
}

impl<S, B, P, A> WorthQueryCorrectionCustodyOutcome<S, B, P, A>
where
    S: ApplicationSchema + 'static,
    B: WorthQuerySourceMutationBinding<S>,
    B::Input: ApplicationMutationInputScope<S, B>,
    P: ApplicationProgramDefinition<S> + 'static,
    A: WorthQuerySourceCompletionAction<S, B> + 'static,
    WorthQueryTerminalSourceEvidence<S, WorthQueryCapturedMutationInput<S, B>>: fmt::Debug,
{
    /// Erases a typed correction custody outcome so its pending command fits in the same
    /// [`WorthQueryAnySourceCustody`] slot as ordinary commands.
    pub fn erase(self) -> WorthQueryAnyCorrectionCustodyOutcome<S, P> {
        use WorthQueryAnyCorrectionCustodyOutcome as AnyOutcome;
        match self {
            Self::Custody(step) => AnyOutcome::Custody(step.erase()),
            Self::PublicationRefused(denial) => AnyOutcome::PublicationRefused(denial),
            Self::IdempotencyIntentDrift => AnyOutcome::IdempotencyIntentDrift,
            Self::PublicationPending => AnyOutcome::PublicationPending,
            Self::ProductUnpublished(handle) => AnyOutcome::ProductUnpublished(handle),
        }
    }
}

impl<S, P> WorthQueryAnySourceCustody<S, P>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S> + 'static,
{
    /// Publishes or continues a prepared operation correction directly into type-erased Host
    /// source custody.
    pub fn accept_correction<B, A>(
        prepared: WorthQueryOperationCorrectionPreparation<'_, S, B, P, A>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> WorthQueryAnyCorrectionCustodyOutcome<S, P>
    where
        B: WorthQuerySourceMutationBinding<S>,
        B::Input: ApplicationMutationInputScope<S, B>,
        A: WorthQuerySourceCompletionAction<S, B> + 'static,
        WorthQueryTerminalSourceEvidence<S, WorthQueryCapturedMutationInput<S, B>>: fmt::Debug,
    {
        WorthQuerySourceCustody::accept_correction(prepared, request).erase()
    }
}
