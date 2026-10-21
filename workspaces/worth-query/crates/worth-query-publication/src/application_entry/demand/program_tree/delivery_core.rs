//! Original complete settlement and the one fresh native delivery path.
use super::*;
pub(super) struct ProgramOutputDeliveryCore<S, P, R>
where
    S: ApplicationSchema,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    pub(super) settlement: WorthQueryApplicationProgramOutputSettlement<Query<S, R>>,
    pub(super) branch: WorthQueryProductBranch,
    // Original settlement and all enclosing native custody die before this clone.
    pub(super) runtime: WorthQueryProgramApplicationRuntime<S, P>,
}

impl<S, P, R> ProgramOutputDeliveryCore<S, P, R>
where
    S: ApplicationSchema + 'static,
    P: ApplicationProgramDefinition<S>,
    R: ApplicationOutputGraphShape<S> + ApplicationFixedOutputRoot,
    Connection<S, R>: WorthQueryApplicationRequiredOutputConnection<S>,
{
    fn matches(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, S>) -> bool {
        std::ptr::eq(self.runtime.runtime(), request.application) && self.branch == request.branch
    }
    fn current_observation(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        let observation = request
            .retain_read(phase)
            .map_err(WorthQueryOperationDeliveryDenial::Observation)?;
        self.require_current_at(&request.at(&observation), phase)?;
        Ok(observation)
    }
    /// The selected typed root and complete original lineage at the pinned observation.
    fn require_current_at(
        &self,
        retained: &WorthQueryApplicationRetainedRequest<'_, '_, '_, S>,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        self.runtime
            .validate_selected_output_root::<R>(
                phase,
                &worth_query_execution::publication_boundary::program_publication_access(),
                &retained.retained_observation().retained,
            )
            .map_err(Denial::Program)?;
        retained
            .require_current_program_output_in_advancement(phase, &self.settlement)
            .map_err(Denial::Currentness)
    }
    pub fn deliver_current(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        if !self.matches(request) {
            return Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                self.current_observation(request, &phase)
            })
            .map_err(WorthQueryOperationDeliveryDenial::Execution)?
    }
    pub fn deliver_current_at(
        &self,
        retained: &WorthQueryApplicationRetainedRequest<'_, '_, '_, S>,
    ) -> Result<(), WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        let request = retained.base_request();
        if !self.matches(&request) {
            return Err(Denial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                // A caller-pinned foreign branch is refused before its root eligibility is read.
                let selected = retained
                    .retained_observation()
                    .select_on(&phase, request.application)
                    .map_err(Denial::Observation)?;
                if selected.product().product_branch() != self.branch {
                    return Err(Denial::Currentness(
                        WorthQueryOutputCurrentnessDenial::ForeignBranch,
                    ));
                }
                self.require_current_at(retained, &phase)
            })
            .map_err(Denial::Execution)?
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
        let policy = WorthQueryOperationDeliveryPolicy::default();
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
        if !self.matches(request) {
            return Err(Denial::FreshRequestMismatch);
        }
        request
            .application
            .with_application_advancement(request.scope, |phase| {
                let observation = self.current_observation(request, &phase)?;
                let retained = request.at(&observation);
                let query = retained.query(intent);
                let query = match policy.query_limits {
                    Some(results) => query.limit_results(results),
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
