//! Original installed action, full settlement and the one fresh native delivery gate.
use super::descriptor::WorthQueryOperationDescription;
use super::*;
mod batch;
mod query;
mod unprojected;
pub(super) struct FixedDeliveryCore<Schema, Intent, Program, Root, Action>
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
    pub(super) installed_action:
        worth_query_declaration::facade::application_program::ApplicationActionDeclaration,
    pub(super) description: WorthQueryOperationDescription<Schema, Intent>,
    pub(super) settlement: WorthQueryApplicationProgramOutputSettlement<RootQuery<Schema, Root>>,
    pub(super) _action: std::marker::PhantomData<Action>,
    // Always last, also after the enclosing owner's native custody/evidence.
    pub(super) runtime: WorthQueryProgramApplicationRuntime<Schema, Program>,
}
impl<Schema, Intent, Program, Root, Action> FixedDeliveryCore<Schema, Intent, Program, Root, Action>
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
            .resolve_declared_action::<Binding<Schema, Intent>>(&self.installed_action)
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
