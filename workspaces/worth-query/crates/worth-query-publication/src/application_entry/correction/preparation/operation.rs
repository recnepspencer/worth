//! Corrections transfer their exact inverse into the installed action's native source custody.
use super::*;
use super::{binding::Inverse, candidate::NativePreparation};
use crate::application_entry::mutation::authorization::WorthQueryBindingAdmission;
use crate::application_entry::{
    WorthQueryApplicationMutationOutcome, WorthQueryApplicationRetainedMutationOutcome,
    WorthQueryCapturedMutationInput, WorthQueryCompletedSourceOperation,
    WorthQuerySourceDeliveryMode, WorthQuerySourceOnlyAdvance, WorthQuerySourceOnlyOperation,
    WorthQueryTerminalSourceOperation,
};
use worth_query_declaration::facade::application_program::*;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQueryProgramOwner,
};
mod publication;
pub use publication::WorthQueryPreparedOperationCorrection;

type Input<S, B> = <B as ApplicationMutationBinding<S>>::Input;
type Mode<S, B, A> =
    <<A as ApplicationAction<S, B>>::Delivery as ApplicationActionDelivery<S, B>>::Mode;
type Operation<S, B, P, A> =
    WorthQuerySourceOnlyOperation<S, WorthQueryCapturedMutationInput<S, B>, P, A, Mode<S, B, A>>;
type Advance<S, B, P, A> = WorthQuerySourceOnlyAdvance<
    Operation<S, B, P, A>,
    WorthQueryCompletedSourceOperation<
        S,
        WorthQueryCapturedMutationInput<S, B>,
        P,
        A,
        Mode<S, B, A>,
    >,
    WorthQueryTerminalSourceOperation<
        S,
        WorthQueryCapturedMutationInput<S, B>,
        P,
        A,
        Mode<S, B, A>,
    >,
>;

/// First publication and exact replay keep their original native delivery and recovery owners.
pub enum WorthQueryOperationCorrectionPreparation<'a, S, B, P, A>
where
    S: ApplicationSchema,
    B: ApplicationMutationBinding<S>,
    B::Input: ApplicationMutationInputScope<S, B>,
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
        Input<S, Inverse<S, C>>:
            Clone + Send + Sync + ApplicationMutationInputScope<S, Inverse<S, C>>,
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
        // Resolve the real source binding and inverse delivery before candidate
        // handler work. Runtime request admission still governs every effect.
        let action = application
            .installed_program()
            .resolve_action::<Inverse<S, C>, A>()
            .map_err(|cause| {
                Denial::SourceOperation(
                    crate::application_entry::WorthQueryOperationConstructionDenial::Action(cause),
                )
            })?;
        if action.required_output_source()
            || application.owns_output_source(std::any::TypeId::of::<Inverse<S, C>>())
        {
            return Err(Denial::SourceOperation(crate::application_entry::WorthQueryOperationConstructionDenial::RequiredOutputSource));
        }
        let product = runtime.product_runtime();
        if !<Mode<S, Inverse<S, C>, A> as WorthQuerySourceDeliveryMode>::RETAIN_COMMIT {
            product.admit_retained_original_operation_owner().map_err(|cause| Denial::SourceOperation(crate::application_entry::WorthQueryOperationConstructionDenial::DurableStateAdmission(cause)))?;
        }
        let source_binding = runtime.installed_schema().installed_mutation_binding::<C::Source>()
            .map_err(|_| Denial::Derivation(worth_query_execution::facade::primary_graph::WorthQueryCorrectionDerivationDenial::UnsupportedSourceMeaning))?;
        product.admit_native_correction_source_owner(source_binding.operation().contracts(), source_binding.correction_binding())
            .map_err(|cause| Denial::SourceOperation(crate::application_entry::WorthQueryOperationConstructionDenial::DurableStateAdmission(cause)))?;
        let inverse_binding = runtime.installed_schema().installed_mutation_binding::<Inverse<S, C>>()
            .map_err(|_| Denial::Derivation(worth_query_execution::facade::primary_graph::WorthQueryCorrectionDerivationDenial::UnsupportedTargetProgram))?;
        product.admit_native_correction_inverse(inverse_binding.operation().contracts())
            .map_err(|cause| Denial::SourceOperation(crate::application_entry::WorthQueryOperationConstructionDenial::DurableStateAdmission(cause)))?;
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
            retain_operation::<S, Inverse<S, C>, P, A>(prepared, application, branch)
        }).map_err(Denial::ExecutionRequest)?
    }
}

pub(super) fn retain_operation<'a, S, B, P, A>(
    prepared: NativePreparation<'a, S, B>,
    application: &WorthQueryProgramApplicationRuntime<S, P>,
    branch: WorthQueryProductBranch,
) -> Result<
    WorthQueryOperationCorrectionPreparation<'a, S, B, P, A>,
    WorthQueryApplicationCorrectionDenial,
>
where
    S: ApplicationSchema + 'static,
    B: ApplicationMutationAdmissionBinding<S>,
    B::Admission: WorthQueryBindingAdmission<S, B>,
    B::Input: Clone + Send + Sync + ApplicationMutationInputScope<S, B>,
    B::IdempotencyKey: Clone,
    B::ScopeBinding: ApplicationMutationScopeResolution<S, B::PrincipalIdentity>,
    P: ApplicationProgramDefinition<S>,
    A: ApplicationAction<S, B>,
    Mode<S, B, A>: WorthQuerySourceDeliveryMode,
{
    use WorthQueryApplicationCorrectionDenial as Denial;
    use WorthQueryOperationCorrectionPreparation as Public;
    Ok(match prepared {
        NativePreparation::Prepared(action) => {
            let operation = Operation::<S, B, P, A>::retain_prepared_correction(
                application,
                branch,
                &action.identities,
                &action.source,
            )
            .map_err(Denial::SourceOperation)?;
            Public::Prepared(WorthQueryPreparedOperationCorrection {
                action: Box::new(action),
                operation,
            })
        }
        NativePreparation::Replay {
            outcome,
            identities,
            intent,
        } => {
            let continued = Operation::<S, B, P, A>::retain_replayed_correction(
                application,
                branch,
                &identities,
                outcome,
                intent,
            )
            .map_err(Denial::SourceOperation)?;
            Public::Continued(continued)
        }
        NativePreparation::IntentDrift => Public::IdempotencyIntentDrift,
        NativePreparation::PublicationPending => Public::PublicationPending,
        NativePreparation::ProductUnpublished(handle) => Public::ProductUnpublished(handle),
    })
}
