//! A real authored rename consumes the managed Parent's complete observation.
use super::{query, schema::*};
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;
mod handler;
pub use handler::Handler;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Rename {
    pub directory: String,
    pub prefix: String,
    pub item: String,
    pub replacement: String,
}
#[derive(Debug)]
pub struct Renamed;
#[derive(Debug)]
pub enum Denial {
    MissingName,
}
worth_query_structured_value_binding!(pub InputBinding for Rename { identity: "managed-catalog.rename.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Renamed { identity: "managed-catalog.rename.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "managed-catalog.rename.denial.v1" });
worth_query_operation!(pub RenameItem for Catalog, input InputBinding);
worth_query_operation_reads!(RenameItem => [DirectoryIdentity, ItemIdentity, ItemName]);
worth_query_operation_writes!(RenameItem => [ItemName]);
pub struct Binding;
impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for Binding {
    type Input = Rename;
    type InputBinding = InputBinding;
    type Result = Renamed;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = RenameItem;
    type Decision = WorthQueryInvariantMutationTarget<Catalog, Item>;
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
    type SourceExpectation = ApplicationQueryMutationSource<query::PrefixQuery>;
    const IDENTITY: &'static str = "managed-catalog.rename.binding.v1";
    const HANDLER_IDENTITY: &'static str = "managed-catalog.rename.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "managed-catalog.rename.key.v1";
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
        input: &Rename,
    ) -> Result<
        Option<ApplicationQueryParameterSet<query::PrefixQuery>>,
        ApplicationValueEncodeDenial,
    > {
        Ok(Some(<query::FindPrefix as ApplicationQueryIntent<
            Catalog,
        >>::parameters(&query::FindPrefix {
            directory: input.directory.clone(),
            prefix: input.prefix.clone(),
        })))
    }
}
impl ApplicationMutationIntent<Catalog> for Rename {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(DirectoryIdentity::reference(), self.directory.clone())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = RenameItem::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 32)
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_read_field(op, ItemIdentity::reference())
        .operation_read_field(op, ItemName::reference())
        .operation_write(op, ItemName::reference())
        .application_mutation_binding::<Binding>()
}
