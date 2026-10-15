//! Noncounterfeit settled fixed output ownership and declared delivery.
use super::descriptor::WorthQueryOperationDescription;
use super::*;

/// Only the native output settlement handoff constructs this owning view.
/// ```compile_fail,E0451
/// use worth_query_publication::facade::application_entry::WorthQuerySettledFixedOperation;
/// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime, primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource}};
/// fn counterfeit<'r, S, I, P, R, A>(runtime: &'r WorthQueryProgramApplicationRuntime<S, P>) -> WorthQuerySettledFixedOperation<'r, S, I, P, R, A>
/// where
///     S: ApplicationSchema,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationFixedAction<S, I::Binding>,
///     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
/// {
///     WorthQuerySettledFixedOperation {
///         runtime, installed_action: todo!(), description: todo!(), settlement: todo!(),
///         initial_cause: None, recovered: None, result: None,
///         cleanup: todo!(), action: std::marker::PhantomData,
///     }
/// }
/// ```
#[must_use = "delivery does not discharge cleanup; retain this settled owner until lawful close"]
pub struct WorthQuerySettledFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    pub(super) installed_action: &'runtime worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    pub(super) runtime: &'runtime WorthQueryProgramApplicationRuntime<Schema, Program>,
    pub(super) description: WorthQueryOperationDescription<Schema, Intent>,
    pub(super) settlement: WorthQueryApplicationProgramOutputSettlement<RootQuery<Schema, Root>>,
    pub(super) initial_cause: Option<worth_query_execution::facade::runtime::ProductUnpublishedCause>,
    pub(super) recovered: Option<worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryPerformed>,
    pub(super) result: Option<<Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result>,
    pub(super) cleanup: WorthQueryOperationCleanup,
    pub(super) action: std::marker::PhantomData<Action>,
}

#[derive(Debug)]
/// Explains current delivery refusal while the settled owner retains its output and cleanup custody.
pub enum WorthQueryOperationDeliveryDenial {
    FreshRequestMismatch,
    Execution(worth_query_execution::facade::application_contribution::WorthQueryAdvancementDenial),
    Observation(
        worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial,
    ),
    Program(worth_query_execution::facade::application_installation::WorthQuerySelectedProgramOwnerDenial),
    Action(worth_query_installation::facade::WorthQueryActionResolutionDenial),
    Currentness(WorthQueryOutputCurrentnessDenial),
    /// Native consumed-source movement makes current delivery stale; history remains owned.
    Stale(worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenial),
    Query(WorthQueryApplicationRequestQueryDenial),
    Batch(super::super::WorthQueryApplicationQueryBatchDenial),
}

impl<'runtime, Schema, Intent, Program, Root, Action>
    WorthQuerySettledFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    /// The original owner-issued branch selector for fresh delivery and close context.
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.description.branch
    }

    pub fn declared_action(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationActionDeclaration {
        self.installed_action
    }
    pub fn initial_cause(
        &self,
    ) -> Option<worth_query_execution::facade::runtime::ProductUnpublishedCause> {
        self.initial_cause
    }
    pub fn settlement(
        &self,
    ) -> &WorthQueryApplicationProgramOutputSettlement<RootQuery<Schema, Root>> {
        &self.settlement
    }
    pub fn handler_result(
        &self,
    ) -> Option<&<Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result> {
        self.result.as_ref()
    }
    pub fn recovered_progress(&self) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryPerformed>{
        self.recovered.as_ref()
    }

    pub(super) fn current_observation(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<WorthQueryApplicationReadObservation, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application)
            || self.description.branch != request.branch
        {
            return Err(Denial::FreshRequestMismatch);
        }
        let observation = request.retain_read(phase).map_err(Denial::Observation)?;
        let selected = observation
            .select_on(phase, request.application)
            .map_err(Denial::Observation)?;
        let owner = self
            .runtime
            .selected_program_owner(&selected)
            .map_err(Denial::Program)?;
        owner
            .resolve_declared_action::<Binding<Schema, Intent>>(self.installed_action)
            .map_err(Denial::Action)?;
        let installed = self
            .runtime
            .runtime()
            .output_demand_resource_profile()
            .limits()
            .source_currentness_work();
        let work = std::num::NonZeroUsize::new(policy.currentness_work.get().min(installed))
            .expect("installed finite positive currentness policy");
        request.at(&observation).require_current_program_output_in_advancement(phase, &self.settlement, work).map_err(|denial| match denial {
            WorthQueryOutputCurrentnessDenial::Output(output) if matches!(output.kind(), worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded) => Denial::Stale(output),
            other => Denial::Currentness(other),
        })?;
        Ok(observation)
    }
}

impl<'runtime, Schema, Intent, Program, Root, Action> WorthQuerySettledFixedOperation<'runtime, Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationFixedAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedQueryDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
    <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = Read<Schema, Intent, Action>>,
    <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Query> + Clone,
{
    pub fn read_current(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        self.read_current_with_policy(request, WorthQueryOperationDeliveryPolicy::currentness(
            std::num::NonZeroUsize::new(self.runtime.runtime().output_demand_resource_profile().limits().source_currentness_work()).expect("installed finite positive currentness policy")))
    }

    /// Attempts delivery under fresh authority and narrower finite native ceilings.
    /// Refusal leaves this settled owner and its cleanup custody unchanged.
    pub fn read_current_with_policy(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, policy: WorthQueryOperationDeliveryPolicy) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application) || self.description.branch != request.branch { return Err(Denial::FreshRequestMismatch); }
        self.runtime.runtime().with_application_advancement(request.scope, |phase| {
            let observation = self.current_observation(request, &phase, policy)?;
            let retained = request.at(&observation);
            let query = retained.query(Delivery::<Schema, Intent, Action>::read_input(self.description.intent.input()));
            let query = match policy.query_limits { Some((results, work)) => query.limits(results, work), None => query };
            query.execute_in_advancement(&phase).map_err(Denial::Query)
        }).map_err(Denial::Execution)?
    }
    /// Reads the original declared projection at its retained settlement basis with fresh authority.
    /// This does not advance outputs or claim that their consumed facts are current.
    pub fn read_historical(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        if !std::ptr::eq(self.runtime.runtime(), request.application) || self.description.branch != request.branch { return Err(Denial::FreshRequestMismatch); }
        self.runtime.runtime().with_application_advancement(request.scope, |phase| {
            request.at(self.settlement.observation()).query(Delivery::<Schema, Intent, Action>::read_input(self.description.intent.input())).execute_in_advancement(&phase).map_err(Denial::Query)
        }).map_err(Denial::Execution)?
    }

}
