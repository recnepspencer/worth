//! A direct Query Source expectation guards an ordinary field write.
use super::{query, schema::*};
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;
#[path = "plain/handler.rs"]
mod handler;
pub use handler::Handler;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Rename {
    pub directory: String,
    pub item: String,
    pub replacement: String,
}
#[derive(Debug)]
pub struct Renamed;
#[derive(Debug)]
pub enum Denial {
    MissingName,
}
worth_query_structured_value_binding!(pub InputBinding for Rename { identity: "event-catalog.plain-rename.input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Renamed { identity: "event-catalog.plain-rename.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "event-catalog.plain-rename.denial.v1" });
worth_query_operation!(pub PlainRenameItem for Catalog, input InputBinding);

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
    type Operation = PlainRenameItem;
    type Decision = WorthQueryInvariantMutationTarget<Catalog, Item>;
    type Denial = Denial;
    type DenialBinding = DenialBinding;
    type Output = NoApplicationMutationOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<DirectoryIdentity>;
    type PrincipalBinding = CatalogPrincipal;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    type SourceExpectation = ApplicationQueryMutationSource<query::SourceQuery>;
    const IDENTITY: &'static str = "event-catalog.plain-rename.binding.v1";
    const HANDLER_IDENTITY: &'static str = "event-catalog.plain-rename.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "event-catalog.plain-rename.key.v1";
    const REQUIRES_APPLICATION_PROGRAM: bool = true;
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> DirectoryIdentity {
        DirectoryIdentity::at("DirectoryIdentity")
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
        _: &Rename,
    ) -> Result<
        Option<ApplicationQueryParameterSet<query::SourceQuery>>,
        ApplicationValueEncodeDenial,
    > {
        Ok(Some(ApplicationQueryParameterSet::new()))
    }
}
impl ApplicationMutationIntent<Catalog> for Rename {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(
            DirectoryIdentity::at("DirectoryIdentity"),
            self.directory.clone(),
        )
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = PlainRenameItem::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 32)
        .operation_read_field(op, DirectoryIdentity::at("DirectoryIdentity"))
        .operation_read_field(op, ItemIdentity::at("ItemIdentity"))
        .operation_read_field(op, ItemName::at("ItemName"))
        .operation_write(op, ItemName::at("ItemName"))
        .application_mutation_binding::<Binding>()
}
