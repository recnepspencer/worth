//! Reapplication transfers source completion to the ordinary installed action owner.
use super::*;
use crate::application_entry::mutation::authorization::WorthQueryBindingAdmission;
use crate::application_entry::WorthQuerySourceDeliveryMode;
use worth_query_declaration::facade::application_program::*;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;

type Mode<S, B, A> =
    <<A as ApplicationAction<S, B>>::Delivery as ApplicationActionDelivery<S, B>>::Mode;

impl<'a, S: ApplicationSchema + 'static>
    WorthQueryApplicationReapplicationRequest<'a, '_, '_, '_, S>
{
    /// Prepares the original action with its installed source-completion owner.
    pub fn prepare_operation<C, P, A>(
        self,
        application: &WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQueryOperationCorrectionPreparation<'a, S, C::Source, P, A>,
        WorthQueryApplicationCorrectionDenial,
    >
    where
        C: ApplicationCorrectionBinding<S>,
        P: ApplicationProgramDefinition<S>,
        A: ApplicationAction<S, C::Source>,
        Mode<S, C::Source, A>: WorthQuerySourceDeliveryMode,
        C::Source: ApplicationMutationAdmissionBinding<S>,
        <C::Source as ApplicationMutationAdmissionBinding<S>>::Admission:
            WorthQueryBindingAdmission<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::IdempotencyKey: Clone,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
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
        let alternative = self.correction.alternative;
        let limits = self.limits;
        runtime
            .with_application_advancement(scope, |phase| {
                let prepared = PrepareReapplication {
                    request: self,
                    phase: &phase,
                }
                .candidate::<C, <C::Source as ApplicationMutationAdmissionBinding<S>>::Admission>(
                    |target, access, key| {
                        target
                            .derive_correction_reapplication::<C>(
                                access,
                                &phase,
                                principal,
                                scope,
                                alternative,
                                key,
                                limits.maximum_ancestors,
                                limits.maximum_disclosure_results,
                            )
                            .map_err(Denial::Derivation)
                    },
                )?;
                super::super::operation::retain_operation::<S, C::Source, P, A>(
                    prepared,
                    application,
                    branch,
                )
            })
            .map_err(Denial::ExecutionRequest)?
    }
}
