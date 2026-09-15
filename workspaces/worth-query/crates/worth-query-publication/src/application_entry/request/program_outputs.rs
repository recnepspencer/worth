use worth_query_declaration::facade::application_program::{
    ApplicationProgramDefinition, ApplicationProgramInventoryIdentity,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::{
    ApplicationSchema, ApplicationStructuredValueBinding,
};
use worth_query_execution::facade::application_contribution::{
    WorthQueryApplicationOutputDemand, WorthQueryProducerOutputFamily,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::WorthQueryApplicationProjection;

use super::WorthQueryApplicationRequest;
use crate::application_entry::mutation::{
    WorthQueryProgramConnectionPlan, WorthQueryProgramRootConnection,
};

type RootDemand<Schema, Root> = <Root as WorthQueryProgramRootConnection<Schema>>::Demand;
type RootFamily<Schema, Root> =
    <RootDemand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type RootSource<Schema, Root> =
    <RootFamily<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;
type RootQuery<Schema, Root> = <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type RootValue<Schema, Root> =
    <<RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Re-enters the exact root demand retained by the installed program.
    /// Callers provide no occurrence key or reconstructed target.
    pub fn recover_required_outputs<Program, Inventory, Root>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<
            'application,
            Schema,
            Program,
            Inventory,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Connections: WorthQueryProgramConnectionPlan<Schema, Program, Inventory>,
        Inventory: ApplicationProgramInventoryIdentity,
        Root: WorthQueryProgramRootConnection<Schema>,
        RootDemand<Schema, Root>: Clone + 'static,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        RootQuery<Schema, Root>: 'static,
        RootValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, RootQuery<Schema, Root>> + Clone + 'static,
    {
        if !std::ptr::eq(application.runtime(), self.application)
            || application.installed_program().schema_binding()
                != &self.application.installed_schema().binding_identity()
        {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ForeignProgram,
            );
        }
        if !application
            .installed_program()
            .contains_connection_type::<Root>()
        {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingConnection,
            );
        }
        let demand = worth_query_execution::facade::publication_integration::program_execution_port(application)
            .recover_program_demand::<Inventory, Root, RootDemand<Schema, Root>>(source_receipt)
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::DemandExecution)?;
        let root = self
            .demand(demand)
            .controls(controls)
            .start_recovery::<Program, Inventory>(application, source_receipt)
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?;
        crate::application_entry::WorthQueryApplicationProgramOutputHandle::new(
            application,
            source_receipt
                .committed_product_publication()
                .composite_commit()
                .clone(),
            Root::TARGET_FEATURE,
            Root::target_feature_type(),
            root,
            controls,
        )
    }
}
