use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
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
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationProjection, WorthQueryApplicationRequiredOutputConnection,
};

use super::WorthQueryApplicationRequest;

type RootConnectionRef<Schema, Root> =
    <Root as ApplicationOutputGraphShape<Schema>>::RootConnection;
type RootConnection<Schema, Root> =
    <RootConnectionRef<Schema, Root> as ApplicationConnectionShape<Schema>>::Binding;
type RootDemand<Schema, Root> =
    <RootConnection<Schema, Root> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type RootFamily<Schema, Root> =
    <RootDemand<Schema, Root> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type RootSource<Schema, Root> =
    <RootFamily<Schema, Root> as WorthQueryProducerOutputFamily<Schema>>::Source;
type RootQuery<Schema, Root> = <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Query;
type RootValue<Schema, Root> = <<RootSource<Schema, Root> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Re-enters one installed program's owner-retained required-output graph
    /// with fresh request authority after caller disposal or interruption.
    pub fn recover_required_outputs<Program, Root>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
        source_receipt: &worth_query_execution::facade::primary_graph::WorthQueryApplicationCommitReceipt,
        root_demand: RootDemand<Schema, Root>,
        controls: crate::application_entry::WorthQueryOutputDemandControls,
    ) -> Result<
        crate::application_entry::WorthQueryApplicationProgramOutputHandle<
            'application,
            Schema,
            Program,
            Root,
        >,
        crate::application_entry::WorthQueryRequiredOutputPreparationDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + worth_query_declaration::facade::application_program::ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        Root::Dependents:
            crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<
                'application,
                Schema,
                Program,
                RootDemand<Schema, Root>,
            >,
        RootDemand<Schema, Root>: Clone,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::Input:
            ApplicationQueryIntent<Schema, Binding = RootSource<Schema, Root>>,
        <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::ScopeBinding:
            ApplicationQueryScopeResolution<
                Schema,
                <RootSource<Schema, Root> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
            >,
        RootValue<Schema, Root>:
            WorthQueryApplicationProjection<Schema, RootQuery<Schema, Root>> + Clone,
    {
        if !std::ptr::eq(application.runtime(), self.application) {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ForeignProgram,
            );
        }
        if !application.contains_output_root::<Root>() {
            return Err(
                crate::application_entry::WorthQueryRequiredOutputPreparationDenial::UndeclaredOutputRoot,
            );
        }
        let (root, observation) = self
            .demand(root_demand.clone())
            .controls(controls)
            .start_recovery::<Program, Root>(application, source_receipt)
            .map_err(crate::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand)?;
        Ok(
            crate::application_entry::WorthQueryApplicationProgramOutputHandle::new(
                application,
                source_receipt.clone(),
                crate::application_entry::WorthQueryApplicationReadObservation::new(observation),
                root,
                root_demand,
                controls,
            ),
        )
    }
}
