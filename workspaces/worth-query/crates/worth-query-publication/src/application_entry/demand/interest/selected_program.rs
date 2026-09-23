use worth_query_declaration::facade::application_program::{
    ApplicationConnectionShape, ApplicationOutputGraphShape, ApplicationProgramDefinition,
    ApplicationRequiredOutputRoot,
};
use worth_query_declaration::facade::application_query::{
    ApplicationQueryBinding, ApplicationQueryIntent, ApplicationQueryScopeResolution,
};
use worth_query_declaration::facade::application_schema::ApplicationSchema;
use worth_query_execution::facade::application_contribution::WorthQueryApplicationOutputDemand;
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationDependentOutputConnection, WorthQueryApplicationProjection,
    WorthQueryApplicationRequiredOutputConnection,
};

use super::types::{ConnectionBinding, RootConnection, SourceBinding, SourceQuery, SourceValue};
use super::{
    WorthQueryApplicationOutputDemandDenial, WorthQueryApplicationOutputDemandHandle,
    WorthQueryApplicationOutputDemandRequest,
};

impl<'application, 'principal, 'scope, Schema, Demand>
    WorthQueryApplicationOutputDemandRequest<'application, 'principal, 'scope, Schema, Demand>
where
    Schema: ApplicationSchema + 'static,
    Demand: WorthQueryApplicationOutputDemand<Schema>,
    SourceValue<Schema, Demand>:
        WorthQueryApplicationProjection<Schema, SourceQuery<Schema, Demand>> + Clone,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::Input:
        ApplicationQueryIntent<Schema, Binding = SourceBinding<Schema, Demand>>,
    <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::ScopeBinding:
        ApplicationQueryScopeResolution<
            Schema,
            <SourceBinding<Schema, Demand> as ApplicationQueryBinding<Schema>>::PrincipalIdentity,
        >,
{
    /// Binds a direct root demand to the exact output declared by this program.
    pub fn start_in_program<Program, Root>(
        self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema> + ApplicationRequiredOutputRoot,
        RootConnection<Schema, Root>:
            WorthQueryApplicationRequiredOutputConnection<Schema, Demand = Demand>,
    {
        if !std::ptr::eq(self.application, program.runtime()) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        if !program.contains_output_root::<Root>() {
            return Err(WorthQueryApplicationOutputDemandDenial::ProgramOutputUndeclared);
        }
        self.start_with_program_selection(program)
    }

    /// Binds a direct dependent demand to the exact connection declared by this program.
    pub fn start_dependent_in_program<Program, Connection>(
        self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Connection: ApplicationConnectionShape<Schema>,
        ConnectionBinding<Schema, Connection>:
            WorthQueryApplicationDependentOutputConnection<Schema, Demand = Demand>,
    {
        if !std::ptr::eq(self.application, program.runtime()) {
            return Err(WorthQueryApplicationOutputDemandDenial::FreshRequestMismatch);
        }
        if !program.contains_output_connection::<Connection>(
            &worth_query_execution::publication_boundary::program_publication_access(),
        ) {
            return Err(WorthQueryApplicationOutputDemandDenial::ProgramOutputUndeclared);
        }
        self.start_with_program_selection(program)
    }

    fn start_with_program_selection<Program>(
        self,
        program: &WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        WorthQueryApplicationOutputDemandHandle<'application, Schema, Demand>,
        WorthQueryApplicationOutputDemandDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
    {
        let selection = (
            program.installed_program().identity().clone(),
            program.installed_program().revision().clone(),
        );
        let mut handle = self.start()?;
        handle.selected_program = Some(selection);
        Ok(handle)
    }
}
