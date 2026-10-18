//! Noncounterfeit settled fixed output ownership and declared delivery.
use super::*;

/// Only the native output settlement handoff constructs this owning view.
/// ```compile_fail,E0451
/// use worth_query_publication::facade::application_entry::WorthQuerySettledFixedOperation;
/// use worth_query_declaration::facade::{application_operation::*, application_program::*, application_schema::ApplicationSchema};
/// use worth_query_execution::facade::{application_installation::WorthQueryProgramApplicationRuntime, primary_graph::{WorthQueryApplicationRequiredOutputConnection, WorthQueryApplicationRequiredOutputSource}};
/// fn counterfeit<S, I, P, R, A>(runtime: WorthQueryProgramApplicationRuntime<S, P>) -> WorthQuerySettledFixedOperation<S, I, P, R, A>
/// where
///     S: ApplicationSchema,
///     I: ApplicationMutationIntent<S>,
///     P: ApplicationProgramDefinition<S>,
///     R: ApplicationOutputGraphShape<S> + ApplicationRequiredOutputRoot,
///     <R::RootConnection as ApplicationConnectionShape<S>>::Binding: WorthQueryApplicationRequiredOutputConnection<S>,
///     I::Binding: WorthQueryApplicationRequiredOutputSource<S, <R::RootConnection as ApplicationConnectionShape<S>>::Binding>,
///     A: ApplicationAction<S, I::Binding>,
///     A::Delivery: ApplicationFixedActionDelivery<S, I::Binding, Root = R>,
/// {
///     WorthQuerySettledFixedOperation {
///         delivery: todo!(), initial_cause: None, recovered: None, result: None,
///         cleanup: todo!(),
///     }
/// }
/// ```
#[must_use = "delivery does not discharge cleanup; retain this settled owner until lawful close"]
pub struct WorthQuerySettledFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    pub(super) initial_cause:
        Option<worth_query_execution::facade::runtime::ProductUnpublishedCause>,
    pub(super) recovered: Option<
        worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryPerformed,
    >,
    pub(super) result:
        Option<<Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result>,
    pub(super) cleanup: WorthQueryOperationCleanup,
    // All native values above are destroyed before the SAME authentic runtime.
    pub(super) delivery: FixedDeliveryCore<Schema, Intent, Program, Root, Action>,
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

impl<Schema, Intent, Program, Root, Action>
    WorthQuerySettledFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>:
        WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>:
        ApplicationFixedActionDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
{
    /// The original owner-issued branch selector for fresh delivery and close context.
    pub fn branch(&self) -> worth_query_execution::facade::product::WorthQueryProductBranch {
        self.delivery.description.branch
    }

    pub fn declared_action(
        &self,
    ) -> &worth_query_declaration::facade::application_program::ApplicationActionDeclaration {
        &self.delivery.installed_action
    }
    pub fn initial_cause(
        &self,
    ) -> Option<worth_query_execution::facade::runtime::ProductUnpublishedCause> {
        self.initial_cause
    }
    pub fn settlement(
        &self,
    ) -> &WorthQueryApplicationProgramOutputSettlement<RootQuery<Schema, Root>> {
        &self.delivery.settlement
    }
    pub fn handler_result(
        &self,
    ) -> Option<&<Binding<Schema, Intent> as ApplicationMutationBinding<Schema>>::Result> {
        self.result.as_ref()
    }
    pub fn recovered_progress(&self) -> Option<&worth_query_execution::facade::primary_graph::WorthQueryManagedApplicationRecoveryPerformed>{
        self.recovered.as_ref()
    }
}

impl<Schema, Intent, Program, Root, Action> WorthQuerySettledFixedOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
    Connection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
    Binding<Schema, Intent>: WorthQueryApplicationRequiredOutputSource<Schema, Connection<Schema, Root>>,
    Action: ApplicationAction<Schema, Binding<Schema, Intent>>,
    Delivery<Schema, Intent, Action>: ApplicationFixedQueryDelivery<Schema, Binding<Schema, Intent>, Root = Root>,
    <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Input: ApplicationQueryIntent<Schema, Binding = Read<Schema, Intent, Action>>,
    <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ScopeBinding: ApplicationQueryScopeResolution<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::PrincipalIdentity>,
    <<Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<Schema, <Read<Schema, Intent, Action> as ApplicationQueryBinding<Schema>>::Query> + Clone,
{
    pub fn read_current(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        self.delivery.read_current(request)
    }

    /// Attempts delivery under fresh authority and narrower finite native ceilings.
    /// Refusal leaves this settled owner and its cleanup custody unchanged.
    pub fn read_current_with_policy(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>, policy: WorthQueryOperationDeliveryPolicy) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        self.delivery.read_current_with_policy(request, policy)
    }
    /// Reads the original declared projection at its retained settlement basis with fresh authority.
    /// This does not advance outputs or claim that their consumed facts are current.
    pub fn read_historical(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> Result<DeliveredProjection<Schema, Read<Schema, Intent, Action>>, WorthQueryOperationDeliveryDenial> {
        self.delivery.read_historical(request)
    }

}
