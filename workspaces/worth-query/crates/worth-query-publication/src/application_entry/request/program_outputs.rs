use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition, ApplicationProgramRootConnection,
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

type RootDemand<Schema, Program> =
    <ApplicationProgramRootConnection<Schema, Program> as WorthQueryApplicationRequiredOutputConnection<Schema>>::Demand;
type RootConnection<Schema, Program> = ApplicationProgramRootConnection<Schema, Program>;
type RootFamily<Schema, Program> =
    <RootDemand<Schema, Program> as WorthQueryApplicationOutputDemand<Schema>>::OutputFamily;
type RootSource<Schema, Program> =
    <RootFamily<Schema, Program> as WorthQueryProducerOutputFamily<Schema>>::Source;
type RootQuery<Schema, Program> =
    <RootSource<Schema, Program> as ApplicationQueryBinding<Schema>>::Query;
type RootValue<Schema, Program> = <<RootSource<Schema, Program> as ApplicationQueryBinding<
    Schema,
>>::ResultBinding as ApplicationStructuredValueBinding>::Value;
type RootEdges<Schema, Program> =
    <<Program as ApplicationProgramDefinition<Schema>>::OutputGraph as ApplicationOutputGraphShape<
        Schema,
    >>::Dependents;

impl<'application, 'principal, 'scope, Schema>
    WorthQueryApplicationRequest<'application, 'principal, 'scope, Schema>
where
    Schema: ApplicationSchema + 'static,
{
    /// Starts the complete declared output graph for one current root source.
    ///
    /// The returned handle owns transitive traversal. Callers cannot settle the
    /// root and silently omit its declared dependent outputs.
    pub fn start_program_outputs<Program>(
        &self,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
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
        Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Program>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        RootEdges<Schema, Program>:
            crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<
                'application,
                Schema,
                Program,
                RootDemand<Schema, Program>,
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
        let observation = self.retain_read().map_err(
            crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ReadObservation,
        )?;
        let root = self
            .at(&observation)
            .demand(root_demand.clone())
            .controls(controls)
            .start_for_program(application)
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
        Program::OutputGraph: ApplicationOutputGraphShape<Schema>,
        RootConnection<Schema, Program>: WorthQueryApplicationRequiredOutputConnection<Schema>,
        RootEdges<Schema, Program>:
            crate::application_entry::mutation::program_output_continuation::ProgramOutputContinuationFactory<
                'application,
                Schema,
                Program,
                RootDemand<Schema, Program>,
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
        let observation = self.retain_read().map_err(
            crate::application_entry::WorthQueryRequiredOutputPreparationDenial::ReadObservation,
        )?;
        let root = self
            .at(&observation)
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
