//! One retained observation binds current installed meaning, complete native guard and delivery.
use super::*;
impl<Schema, Intent, Program, Root, Action>
    WorthQuerySettledDiscoveredOperation<Schema, Intent, Program, Root, Action>
where
    Schema: ApplicationSchema + 'static,
    Intent: ApplicationMutationIntent<Schema>,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema> + ApplicationDiscoveredOutputRoot,
    Connection<Schema, Root>:
        WorthQueryApplicationDiscoveredOutputConnection<Schema, Source = Intent::Binding>,
    Action: ApplicationAction<Schema, Intent::Binding>,
    ActionDelivery<Schema, Intent, Action>:
        ApplicationDiscoveredActionDelivery<Schema, Intent::Binding, Root = Root>,
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
        let work = policy
            .currentness_work
            .map(std::num::NonZeroUsize::get)
            .into_iter()
            .chain(installed)
            .min();
        request.at(&observation).require_current_discovered_program_outputs_in_advancement(phase, &self.settlement, work).map_err(|denial| match denial {
            WorthQueryOutputCurrentnessDenial::Output(output) if matches!(output.kind(), worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded) => Denial::Stale(output),
            other => Denial::Currentness(other),
        })?;
        Ok(observation)
    }
}
