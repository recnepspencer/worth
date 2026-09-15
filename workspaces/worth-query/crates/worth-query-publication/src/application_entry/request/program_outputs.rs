use worth_query_declaration::facade::application_program::ApplicationProgramDefinition;
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
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection,
};

use super::WorthQueryApplicationRequest;

type RootDemand<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::Connections as WorthQueryApplicationRequiredOutputConnection<
        Schema,
    >>::Demand;
type RootFamily<Schema, Program> =
    <RootDemand<Schema, Program> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type RootSource<Schema, Program> =
    <RootFamily<Schema, Program> as WorthQueryProducerOutputFamily<Schema>>::Source;
type RootQuery<Schema, Program> =
    <RootSource<Schema, Program> as ApplicationQueryBinding<Schema>>::Query;
type RootValue<Schema, Program> = <<RootSource<Schema, Program> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Re-enters one installed program's owner-retained required-output graph
    /// with fresh request authority after caller disposal or interruption.
    pub fn recover_required_outputs<Program>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        root_demand: RootDemand<Schema, Program>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<
            'application,
            Schema,
            Program,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Program::Connections: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Program::DependentConnection: WorthQueryApplicationDependentOutputConnection<
            Schema,
            RootDemand = RootDemand<Schema, Program>,
        >,
        RootDemand<Schema, Program>: Clone,
        <RootSource<Schema, Program> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Program>>,
        <RootSource<Schema, Program> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <RootSource<Schema, Program> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        RootValue<Schema, Program>:
            WorthQueryApplicationProjection<Schema, RootQuery<Schema, Program>> + Clone,
    {
        if !std::ptr::eq(application.runtime(), self.application) {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ForeignProgram,
            );
        }
        if !application
            .installed_program()
            .contains_connection(Program::Connections::IDENTITY)
            || !application
                .installed_program()
                .contains_connection(Program::DependentConnection::IDENTITY)
        {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::MissingConnection,
            );
        }
        let root = self
            .demand(root_demand.clone())
            .controls(controls)
            .start_recovery(application, source_receipt)
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?;
        Ok(
            crate::application_entry::WorthQueryApplicationProgramOutputHandle::new(
                application,
                root,
                root_demand,
                controls,
            ),
        )
    }
}
