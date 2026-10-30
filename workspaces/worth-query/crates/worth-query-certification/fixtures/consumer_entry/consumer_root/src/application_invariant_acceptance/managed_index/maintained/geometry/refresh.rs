//! Authored scope metadata edit consumes genuine retained box Source expectations.
use super::super::{bounds, schema::*};
use super::query;
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;
#[derive(Clone, Debug, serde::Serialize)]
pub struct Refresh {
    pub directory: String,
    pub note: String,
}
#[derive(Debug)]
pub struct Refreshed;
#[derive(Debug)]
pub enum Denial {}
worth_query_structured_value_binding!(pub InputBinding for Refresh { identity: "managed-geometry.refresh.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Refreshed { identity: "managed-geometry.refresh.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "managed-geometry.refresh.denial.v1" });
worth_query_operation!(pub RefreshDirectory for Catalog, input InputBinding);
worth_query_operation_reads!(RefreshDirectory => [Directory, DirectoryIdentity, DirectoryNote]);
worth_query_operation_writes!(RefreshDirectory => [DirectoryNote]);
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for Binding {
    type Input = Refresh;
    type InputBinding = InputBinding;
    type Result = Refreshed;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = RefreshDirectory;
    type Decision = WorthQueryInvariantMutationTarget<Catalog, Directory>;
    type Denial = Denial;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        Catalog,
        Directory,
        DirectoryFacts,
        DirectoryIdentity,
        String,
        ReadOnly,
        NoApplicationUnit,
    >;
    type PrincipalBinding = CatalogPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::BoxQuery>;
    const IDENTITY: &'static str = "managed-geometry.refresh-directory.binding.v1";
    const HANDLER_IDENTITY: &'static str = "managed-geometry.refresh-directory.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "managed-geometry.refresh-directory.key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        Catalog,
        Directory,
        DirectoryFacts,
        DirectoryIdentity,
        String,
        ReadOnly,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        DirectoryIdentity::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        Catalog,
        CatalogPrincipal,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        CatalogPrincipal::reference()
    }
    fn expected_source_parameters(
        input: &Refresh,
    ) -> Result<Option<ApplicationQueryParameterSet<query::BoxQuery>>, ApplicationValueEncodeDenial>
    {
        Ok(Some(<query::FindBoxes as ApplicationQueryIntent<
            Catalog,
        >>::parameters(&query::FindBoxes {
            directory: input.directory.clone(),
            probe: bounds::Coordinates::new(-1, -1, 100, 1),
        })))
    }
}
impl ApplicationMutationIntent<Catalog> for Refresh {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(DirectoryIdentity::reference(), self.directory.clone())
    }
}

pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Refresh,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Directory>, Denial> {
        let result = (|| {
            let directory =
                reader.resolve_entity(DirectoryIdentity::reference(), input.directory.clone())?;
            reader
                .reader()
                .require_decision_entity(&directory, Directory::reference())
                .map_err(HandlerExecutionDenial::new)?;
            reader.field(&directory, DirectoryNote::reference())?;
            reader.mutation_target(&directory)
        })();
        match result {
            Ok(value) => HandlerResult::Completed(value),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Refresh,
        target: WorthQueryInvariantMutationTarget<Catalog, Directory>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Refreshed, Denial> {
        let result = (|| {
            let directory = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .write_field(&directory, DirectoryNote::reference(), input.note.clone())
                .map_err(HandlerExecutionDenial::new)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Refreshed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = RefreshDirectory::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 32)
        .operation_read_entity(op, Directory::reference())
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_read_field(op, DirectoryNote::reference())
        .operation_write(op, DirectoryNote::reference())
        .application_mutation_binding::<Binding>()
}
