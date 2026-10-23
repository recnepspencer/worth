//! Corrections transfer their exact inverse into the installed action's native source custody.
use super::binding::{Inverse, NativePreparation};
use super::*;
use crate::application_entry::mutation::authorization::WorthQueryBindingAdmission;
use crate::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRetainedMutationOutcome,
    WorthQueryCompletedSourceOperation, WorthQuerySourceDeliveryMode, WorthQuerySourceOnlyAdvance,
    WorthQuerySourceOnlyOperation, WorthQueryTerminalSourceOperation,
};
use worth_query_declaration::facade::application_program::*;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
mod publication;
pub use publication::WorthQueryPreparedOperationCorrection;

type Input<S, B> = <B as ApplicationMutationBinding<S>>::Input;
type Mode<S, B, A> =
    <<A as ApplicationAction<S, B>>::Delivery as ApplicationActionDelivery<S, B>>::Mode;
type Operation<S, B, P, A> = WorthQuerySourceOnlyOperation<S, Input<S, B>, P, A, Mode<S, B, A>>;
type Advance<S, B, P, A> = WorthQuerySourceOnlyAdvance<
    Operation<S, B, P, A>,
    WorthQueryCompletedSourceOperation<S, Input<S, B>, P, A, Mode<S, B, A>>,
    WorthQueryTerminalSourceOperation<S, Input<S, B>, P, A, Mode<S, B, A>>,
>;

/// First publication and exact replay keep their original native delivery and recovery owners.
pub enum WorthQueryOperationCorrectionPreparation<'a, S, B, P, A>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: ApplicationMutationIntent<S, Binding = B>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, B>,
    Mode<S, B, A>: WorthQuerySourceDeliveryMode,
{
    Prepared(WorthQueryPreparedOperationCorrection<'a, S, B, P, A>),
    Continued(Advance<S, B, P, A>),
    IdempotencyIntentDrift,
    PublicationPending,
    ProductUnpublished(worth_runtime_world::facade::ProductUnpublishedRecoveryHandle),
}

impl<'a, S: ApplicationSchema + 'static> WorthQueryApplicationCorrectionRequest<'a, '_, '_, '_, S> {
    /// Prepares the installed inverse action and its declared source-completion owner.
    /// Publish the candidate, then advance or close the returned native custody.
    pub fn prepare_operation<C, P, A>(
        self,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQueryOperationCorrectionPreparation<'a, S, Inverse<S, C>, P, A>,
        WorthQueryApplicationCorrectionDenial,
    >
    where
        C: ApplicationCorrectionBinding<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, Inverse<S, C>>,
        Mode<S, Inverse<S, C>, A>: WorthQuerySourceDeliveryMode,
        Inverse<S, C>: ApplicationMutationAdmissionBinding<S>,
        <Inverse<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission:
            WorthQueryBindingAdmission<S, Inverse<S, C>>,
        Input<S, Inverse<S, C>>: Clone
            + Send
            + Sync
            + ApplicationMutationIntent<S, Binding = Inverse<S, C>>
            + ApplicationMutationInputScope<S, Inverse<S, C>>,
        <Inverse<S, C> as ApplicationMutationBinding<S>>::IdempotencyKey: Clone,
        <Inverse<S, C> as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <Inverse<S, C> as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        use WorthQueryApplicationCorrectionDenial as Denial;
        let runtime = self.correction.request.application;
        if !std::ptr::eq(runtime, application.runtime()) {
            return Err(Denial::SourceOperation(
                crate::application_entry::WorthQueryOperationConstructionDenial::ForeignRuntime,
            ));
        }
        let principal = self.correction.request.principal;
        let scope = self.correction.request.scope;
        let branch = self.correction.request.branch;
        let source = self.correction.source;
        let limits = self.limits;
        runtime.with_application_advancement(scope, |phase| {
            let prepared = Prepare { request: self, phase: &phase }
                .candidate::<C, <Inverse<S, C> as ApplicationMutationAdmissionBinding<S>>::Admission>(|target, access, key| {
                    target.derive_correction_inverse::<C>(access, &phase, principal, scope, source, key,
                        limits.maximum_ancestors, limits.maximum_disclosure_results).map_err(Denial::Derivation)
                })?;
            use WorthQueryOperationCorrectionPreparation as Public;
            Ok(match prepared {
                NativePreparation::Prepared(inverse) => {
                    let operation = Operation::<S, Inverse<S, C>, P, A>::retain_recorded_correction(
                        application, branch, &inverse.identities, inverse.source.intent()).map_err(Denial::SourceOperation)?;
                    Public::Prepared(WorthQueryPreparedOperationCorrection { inverse, operation })
                }
                NativePreparation::Replay { outcome, identities, intent } => {
                    let operation = Operation::<S, Inverse<S, C>, P, A>::retain_recorded_correction(
                        application, branch, &identities, intent).map_err(Denial::SourceOperation)?;
                    Public::Continued(operation.accept_recorded_publication(WorthQueryApplicationRetainedMutationOutcome::Other(outcome)))
                }
                NativePreparation::IntentDrift => Public::IdempotencyIntentDrift,
                NativePreparation::PublicationPending => Public::PublicationPending,
                NativePreparation::ProductUnpublished(handle) => Public::ProductUnpublished(handle),
            })
        }).map_err(Denial::ExecutionRequest)?
    }
}
