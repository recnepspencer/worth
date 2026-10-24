//! Reapplication retains the original source action's required-output delivery.
use super::*;
use crate::application_entry::mutation::{
    authorization::WorthQueryBindingAdmission, performed_source::require_program_output_root,
};
use worth_query_declaration::facade::application_program::*;
use worth_query_execution::facade::{
    application_installation::WorthQueryProgramApplicationRuntime,
    primary_graph::{
        WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource,
    },
};

type Connection<S, R> = <<R as ApplicationOutputGraphShape<S>>::RootConnection as ApplicationConnectionShape<S>>::Binding;

impl<'a, S: ApplicationSchema> WorthQueryApplicationReapplicationRequest<'a, '_, '_, '_, S> {
    /// Prepares original retained input with the program's typed required output root.
    pub fn prepare_performed<C, P, R>(
        self,
        application: &'a WorthQueryProgramApplicationRuntime<S, P>,
    ) -> Result<
        WorthQueryRequiredCorrectionPreparation<'a, S, C::Source, P, R>,
        WorthQueryApplicationCorrectionDenial,
    >
    where
        C: ApplicationCorrectionBinding<S>,
        P: ApplicationProgramDefinition<S>,
        R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
        Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
        C::Source: ApplicationMutationAdmissionBinding<S>
            + WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>,
        <C::Source as ApplicationMutationAdmissionBinding<S>>::Admission:
            WorthQueryBindingAdmission<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::Input:
            Clone + Send + Sync + ApplicationMutationInputScope<S, C::Source>,
        <C::Source as ApplicationMutationBinding<S>>::ScopeBinding:
            ApplicationMutationScopeResolution<
                S,
                <C::Source as ApplicationMutationBinding<S>>::PrincipalIdentity,
            >,
    {
        use WorthQueryApplicationCorrectionDenial as Denial;
        require_program_output_root::<S, P, R>(self.correction.request.application, application)
            .map_err(Denial::RequiredOutputs)?;
        let principal = self.correction.request.principal;
        let scope = self.correction.request.scope;
        let alternative = self.correction.alternative;
        let limits = self.limits;
        let runtime = self.correction.request.application;
        runtime.with_application_advancement(scope, |phase| {
            let mut demand = None;
            let prepared = PrepareReapplication { request: self, phase: &phase }
                .candidate::<C, <C::Source as ApplicationMutationAdmissionBinding<S>>::Admission>(|target, access, key| {
                    let (input, source) = target.derive_required_output_reapplication::<C, R>(
                        access, &phase, principal, scope, alternative, key,
                        limits.maximum_ancestors, limits.maximum_disclosure_results,
                    ).map_err(Denial::Derivation)?;
                    demand = Some(<C::Source as WorthQueryApplicationRequiredOutputSource<S, Connection<S, R>>>::demand_from_source(input.input())
                        .map_err(|cause| Denial::RequiredOutputs(crate::application_entry::WorthQueryPerformedMutationExecutionDenial::Connection(cause)))?);
                    Ok((input, source))
                })?;
            Ok(super::super::required_output::retain_required_output(prepared, application, demand))
        }).map_err(Denial::ExecutionRequest)?
    }
}
