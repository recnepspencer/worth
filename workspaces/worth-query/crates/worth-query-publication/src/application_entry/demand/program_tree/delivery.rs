//! Whole original membership and typed read share the exact native observation/phase.
use super::*;
use std::num::NonZeroUsize;
type ReadBinding<S, I> = <I as ApplicationQueryIntent<S>>::Binding;
type ReadQuery<S, I> = <ReadBinding<S, I> as ApplicationQueryBinding<S>>::Query;
type ReadValue<S,I> = <<ReadBinding<S,I> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
impl<S, P, R> WorthQuerySettledProgramOutputOperation<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    fn current_observation(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        let observation = request.retain_read(phase).map_err(Denial::Observation)?;
        self.core
            .runtime
            .validate_selected_output_root::<R>(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                &observation.retained,
            )
            .map_err(Denial::Program)?;
        let installed = self
            .core
            .runtime
            .runtime()
            .output_demand_resource_profile()
            .limits()
            .source_currentness_work();
        let work = NonZeroUsize::new(policy.currentness_work.get().min(installed))
            .expect("native currentness ceiling is positive");
        request
            .at(&observation)
            .require_current_program_output_in_advancement(phase, &self.settlement, work)
            .map_err(Denial::Currentness)?;
        Ok(observation)
    }
    pub fn deliver_current(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        if !self.core.matches(request) {
            return Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.current_observation(request, &phase, policy)
            })
            .map_err(WorthQueryOperationDeliveryDenial::Execution)?
    }
    pub fn read_current<I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        intent: I,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<
            ReadQuery<S, I>,
            ReadValue<S, I>,
        >,
        WorthQueryOperationDeliveryDenial,
    >
    where
        I: ApplicationQueryIntent<S>,
        <ReadBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <ReadBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        ReadValue<S, I>: WorthQueryApplicationProjection<S, ReadQuery<S, I>>,
    {
        let policy = WorthQueryOperationDeliveryPolicy::currentness(
            NonZeroUsize::new(
                self.core
                    .runtime
                    .runtime()
                    .output_demand_resource_profile()
                    .limits()
                    .source_currentness_work(),
            )
            .expect("native policy is finite"),
        );
        self.read_current_with_policy(request, intent, policy)
    }
    pub fn read_current_with_policy<I>(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        intent: I,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<
        crate::domain_computation::WorthQueryPublishedApplicationResult<
            ReadQuery<S, I>,
            ReadValue<S, I>,
        >,
        WorthQueryOperationDeliveryDenial,
    >
    where
        I: ApplicationQueryIntent<S>,
        <ReadBinding<S, I> as ApplicationQueryBinding<S>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                S,
                <ReadBinding<S, I> as ApplicationQueryBinding<S>>::PrincipalIdentity,
            >,
        ReadValue<S, I>: WorthQueryApplicationProjection<S, ReadQuery<S, I>>,
    {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !self.core.matches(request) {
            return Err(Denial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                let observation = self.current_observation(request, &phase, policy)?;
                let retained = request.at(&observation);
                let query = retained.query(intent);
                let query = match policy.query_limits {
                    Some((results, work)) => query.limits(results, work),
                    None => query,
                };
                query.execute_in_advancement(&phase).map_err(Denial::Query)
            })
            .map_err(Denial::Execution)?
    }
    pub fn settlement(&self) -> &WorthQueryApplicationProgramOutputSettlement<Query<S, R>> {
        &self.settlement
    }
}
