//! A declared generated geometry and both memberships come from one genuine producer.
use super::super::super::schema::*;
use super::super::schema::*;
use super::source;
use worth_query_decl::facade::{application_operation::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;
mod handler;
pub use handler::Handler;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Generate {
    pub directory: String,
}
#[derive(Debug)]
pub struct Generated;
#[derive(Debug)]
pub enum Denial {}
pub struct Decision {
    pub a: WorthQueryInvariantMutationTarget<Catalog, Directory>,
    pub b: WorthQueryInvariantMutationTarget<Catalog, Directory>,
}
worth_query_structured_value_binding!(pub InputBinding for Generate { identity: "maintained-generated-geometry.operation-input.v1" });
worth_query_structured_value_binding!(pub ResultBinding for Generated { identity: "maintained-generated-geometry.operation-result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "maintained-generated-geometry.operation-denial.v1" });
worth_query_operation!(pub GenerateGeometry for Catalog, input InputBinding);
worth_query_operation_reads!(GenerateGeometry => [Directory, DirectoryIdentity, DirectoryNote]);
worth_query_operation_creates!(GenerateGeometry => [Edge, Vertex]);
worth_query_operation_links!(GenerateGeometry => [ContainsEdge, Starts, Ends]);
worth_query_operation_writes!(GenerateGeometry => [EdgeIdentity, VertexIdentity, VertexRegion]);

pub struct Outputs;
pub struct EdgeOutput;
impl WorthQueryApplicationOutputRole for EdgeOutput {
    type Schema = Catalog;
    type Contract = Outputs;
    type Entity = Edge;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "edge";
}
pub struct StartOutput;
pub struct EndOutput;
impl WorthQueryApplicationOutputRole for StartOutput {
    type Schema = Catalog;
    type Contract = Outputs;
    type Entity = Vertex;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "start";
}
impl WorthQueryApplicationOutputRole for EndOutput {
    type Schema = Catalog;
    type Contract = Outputs;
    type Entity = Vertex;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "end";
}
impl ApplicationMutationOutputContract<Catalog> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[
        <EdgeOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <StartOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
        <EndOutput as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR,
    ];
}

pub struct Binding;
impl ApplicationMutationAdmissionBinding<Catalog> for Binding {
    type Admission = ApplicationConventionalAdmission;
}
impl ApplicationMutationBinding<Catalog> for Binding {
    type Input = Generate;
    type InputBinding = InputBinding;
    type Result = Generated;
    type ResultBinding = ResultBinding;
    type IdempotencyKey = u64;
    type Operation = GenerateGeometry;
    type Decision = Decision;
    type Denial = Denial;
    type DenialBinding = DenialBinding;
    type Output = Outputs;
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
    type SourceExpectation = ApplicationQueryMutationSource<source::Query>;
    const IDENTITY: &'static str = "maintained-generated-geometry.operation-binding.v1";
    const HANDLER_IDENTITY: &'static str = "maintained-generated-geometry.handler.v1";
    const IDEMPOTENCY_IDENTITY: &'static str = "maintained-generated-geometry.idempotency.v1";
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
}
impl ApplicationMutationIntent<Catalog> for Generate {
    type Binding = Binding;
    fn input(&self) -> &Self {
        self
    }
    fn scope_binding(&self) -> <Binding as ApplicationMutationBinding<Catalog>>::ScopeBinding {
        ApplicationMutationFieldScope::new(DirectoryIdentity::reference(), self.directory.clone())
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = GenerateGeometry::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 64)
        .operation_read_entity(op, Directory::reference())
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_read_field(op, DirectoryNote::reference())
        .operation_create(op, Edge::reference())
        .operation_create(op, Vertex::reference())
        .operation_link(op, ContainsEdge::reference())
        .operation_link(op, Starts::reference())
        .operation_link(op, Ends::reference())
        .operation_write(op, EdgeIdentity::reference())
        .operation_write(op, VertexIdentity::reference())
        .operation_write(op, VertexRegion::reference())
        .application_mutation_binding::<Binding>()
}
