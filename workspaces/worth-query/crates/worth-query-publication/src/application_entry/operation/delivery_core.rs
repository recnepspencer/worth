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
        if !self.matches(request) {
            return Err(WorthQueryOperationDeliveryDenial::FreshRequestMismatch);
        }
        let observation = request
            .retain_read(phase)
            .map_err(WorthQueryOperationDeliveryDenial::Observation)?;
        self.require_current_at(&request.at(&observation), phase, policy)?;
        Ok(observation)
    }
    fn matches(&self, request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>) -> bool {
        std::ptr::eq(self.runtime.runtime(), request.application)
            && self.description.branch == request.branch
    }
    /// The original action, ceiling and complete lineage at the request's pinned observation.
    pub(super) fn require_current_at(
        &self,
        retained: &WorthQueryApplicationRetainedRequest<'_, '_, '_, Schema>,
        phase: &WorthQueryAdvancementPhase<'_>,
        policy: WorthQueryOperationDeliveryPolicy,
    ) -> Result<(), WorthQueryOperationDeliveryDenial> {
        use WorthQueryOperationDeliveryDenial as Denial;
        let selected = retained
            .retained_observation()
            .select_on(phase, self.runtime.runtime())
            .map_err(Denial::Observation)?;
        // A foreign branch is refused before its installed meaning is read.
        if selected.product().product_branch() != self.description.branch {
            return Err(Denial::Currentness(
                WorthQueryOutputCurrentnessDenial::ForeignBranch,
            ));
        }
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
        retained.require_current_program_output_in_advancement(phase, &self.settlement, work).map_err(|denial| match denial {
            WorthQueryOutputCurrentnessDenial::Output(output) if matches!(output.kind(), worth_query_execution::facade::primary_graph::WorthQueryOutputDemandDenialKind::Superseded) => Denial::Stale(output),
            other => Denial::Currentness(other),
        })
    }
}
