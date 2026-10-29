//! Lawful directory retirement/recreation and an ordinary identity observation.
use super::{query::ParametersBinding, schema::*};
use worth_query_decl::facade::{
    application_operation::*, application_query::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub enum Edit {
    Retire,
    Create,
}
#[derive(Clone, Debug, serde::Serialize)]
pub struct Replace {
    pub edit: Edit,
}
#[derive(Debug)]
pub struct Changed;
#[derive(Debug)]
pub enum Denial {
    Missing,
}
worth_query_structured_value_binding!(pub InputBinding for Replace { identity: "managed-catalog.directory-edit.input.v1" });
worth_query_structured_value_binding!(pub ChangedBinding for Changed { identity: "managed-catalog.directory-edit.result.v1" });
worth_query_structured_value_binding!(pub DenialBinding for Denial { identity: "managed-catalog.directory-edit.denial.v1" });
worth_query_operation!(pub EditDirectory for Catalog, input InputBinding);
worth_query_operation_reads!(EditDirectory => [Directory, DirectoryIdentity]);
worth_query_operation_writes!(EditDirectory => [DirectoryIdentity]);
worth_query_operation_creates!(EditDirectory => [Directory]);
worth_query_operation_deletes!(EditDirectory => [Directory]);
fn scope(_: &Replace) -> String {
    "catalog".into()
}
worth_query_mutation_binding!(
    pub Binding for Replace, schema Catalog, identity "managed-catalog.directory-edit.binding.v1",
    input InputBinding, operation EditDirectory, result ChangedBinding,
    idempotency u64, identity "managed-catalog.directory-edit.key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Directory>, denial DenialBinding,
    handler identity "managed-catalog.directory-edit.handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(),
    outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value scope,
);
pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        input: &Replace,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Directory>, Denial> {
        let mut observe = || -> Result<_, HandlerExecutionDenial> {
            let identity = match input.edit {
                Edit::Retire => "empty",
                Edit::Create => "catalog",
            };
            let directory =
                reader.resolve_entity(DirectoryIdentity::reference(), identity.to_owned())?;
            reader
                .reader()
                .require_decision_entity(&directory, Directory::reference())
                .map_err(HandlerExecutionDenial::new)?;
            reader.field(&directory, DirectoryIdentity::reference())?;
            reader.mutation_target(&directory)
        };
        match observe() {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Replace,
        target: WorthQueryInvariantMutationTarget<Catalog, Directory>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<Changed, Denial> {
        let mut apply = || -> Result<(), HandlerExecutionDenial> {
            let directory = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            match input.edit {
                Edit::Retire => writer
                    .delete_entity(Directory::reference(), &directory)
                    .map_err(HandlerExecutionDenial::new),
                Edit::Create => {
                    let key = WorthQueryApplicationEntityKey::new("replacement-empty-directory")
                        .map_err(HandlerExecutionDenial::new)?;
                    let replacement = writer
                        .create_entity_in_context(&directory, Directory::reference(), key)
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .initialize_field(
                            &replacement,
                            DirectoryIdentity::reference(),
                            "empty".to_owned(),
                        )
                        .map_err(HandlerExecutionDenial::new)
                }
            }
        };
        match apply() {
            Ok(()) => HandlerResult::Completed(Changed),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Row {
    pub coordinate: (u32, u64, u32),
    pub identity: String,
}
worth_query_structured_value_binding!(pub ResultBinding for Row { identity: "managed-catalog.directory-row.v1" });
worth_query_application_query!(pub DirectoryQuery for Catalog,
    identity "managed-catalog.directory-observation.v1", parameters ParametersBinding,
    result ResultBinding, scope Directory => "Directory", name "directory_observation");
pub struct IdentitySlot;
worth_query_portable_type!(IdentitySlot => "managed-catalog.directory-identity-slot.v1");
fn identity() -> ApplicationQueryResultFieldRef<
    DirectoryQuery,
    IdentitySlot,
    Catalog,
    Directory,
    DirectoryFacts,
    DirectoryIdentity,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("identity", DirectoryIdentity::reference())
}
impl WorthQueryApplicationProjection<Catalog, DirectoryQuery> for Row {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Catalog, DirectoryQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let id = row.entity_id();
        Ok(Self {
            coordinate: (
                id.partition_value(),
                id.local_slot_value(),
                id.generation_value(),
            ),
            identity: row.field(identity())?,
        })
    }
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Read {
    pub directory: String,
}
worth_query_structured_value_binding!(pub ReadBinding for Read { identity: "managed-catalog.directory-read.v1" });
fn read_scope(input: Read) -> String {
    input.directory
}
worth_query_query_binding!(pub QueryBinding for Read, schema Catalog,
    identity "managed-catalog.directory-binding.v1", input ReadBinding,
    query DirectoryQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result ResultBinding, principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Directory, DirectoryFacts, DirectoryIdentity, String, ReadOnly, NoApplicationUnit,
    field DirectoryIdentity::reference(), value read_scope, limits results 1
);
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let query = ApplicationQueryDefinitionBuilder::declare(DirectoryQuery::reference())
        .root(Directory::reference())
        .scope(Directory::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Catalog,
                DirectoryQuery,
                Directory,
                Row,
                ResultBinding,
            >::new(Directory::reference())
            .field(identity())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap();
    let op = EditDirectory::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_entity(op, Directory::reference())
        .operation_read_field(op, DirectoryIdentity::reference())
        .operation_create(op, Directory::reference())
        .operation_delete(op, Directory::reference())
        .operation_write(op, DirectoryIdentity::reference())
        .application_mutation_binding::<Binding>()
        .application_query(query)
        .application_query_binding::<QueryBinding>()
}
