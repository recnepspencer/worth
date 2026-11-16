//! A real disjoint Directory edit precedes required nested Source acquisition.
use super::super::{authoring, schema::*};
use super::{program, Demand};
use worth_query_decl::facade::{
    application_operation::*, application_program::*, application_schema::*, *,
};
use worth_query_host::facade::primary_graph::*;

#[derive(Clone, Debug, serde::Serialize)]
pub struct Create {
    pub directory: String,
}
worth_query_structured_value_binding!(pub InputBinding for Create { identity: "nested-required-support.directory-input.v1" });
worth_query_operation!(pub CreateDirectory for Catalog, input InputBinding);
worth_query_operation_reads!(CreateDirectory => [Directory]);
worth_query_operation_creates!(CreateDirectory => [Directory]);
fn scope(_: &Create) -> String {
    "catalog".into()
}
worth_query_mutation_binding!(pub Binding for Create, schema Catalog, identity "nested-required-support.directory-binding.v1",
    input InputBinding, operation CreateDirectory, result authoring::ResultBinding,
    idempotency u64, identity "nested-required-support.directory-key.v1",
    decision WorthQueryInvariantMutationTarget<Catalog, Directory>, denial authoring::DenialBinding,
    handler identity "nested-required-support.directory-handler.v1", program required,
    workflow_requirements ApplicationWorkflowExecutionRequirements::not_applicable(), outputs NoApplicationMutationOutputs,
    principal CatalogPrincipal, mapping ExternalMapping, principal_entity Principal, principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope DirectoryIdentity, field DirectoryIdentity::at("DirectoryIdentity"), value scope,);

pub struct Handler;
impl OperationHandler<Catalog, Binding> for Handler {
    fn decide(
        &self,
        _: &Create,
        reader: &mut DecisionReader<'_, '_, '_, Catalog, Binding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<Catalog, Directory>, authoring::Denial>
    {
        HandlerResult::from_result((|| {
            let entity = reader.resolve_entity(
                DirectoryIdentity::at("DirectoryIdentity"),
                "catalog".to_owned(),
            )?;
            reader
                .reader()
                .require_decision_entity(&entity, Directory::reference())
                .map_err(HandlerExecutionDenial::new)?;
            Ok(reader.mutation_target(&entity)?)
        })())
    }
    fn build_candidate(
        &self,
        input: &Create,
        target: WorthQueryInvariantMutationTarget<Catalog, Directory>,
        writer: &mut CandidateWriter<'_, Catalog, Binding>,
    ) -> HandlerResult<authoring::Changed, authoring::Denial> {
        HandlerResult::from_result((|| {
            let target = writer
                .projected_entity(&target)
                .map_err(HandlerExecutionDenial::new)?;
            let created = writer
                .create_entity_in_context(
                    &target,
                    Directory::reference(),
                    WorthQueryApplicationEntityKey::new(&input.directory)
                        .map_err(HandlerExecutionDenial::new)?,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(
                    &created,
                    DirectoryIdentity::at("DirectoryIdentity"),
                    input.directory.clone(),
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(authoring::Changed)
        })())
    }
}
pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Catalog>,
) -> ApplicationSchemaDeclarationBuilder<Catalog> {
    let op = CreateDirectory::reference();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 16)
        .operation_read_entity(op, Directory::reference())
        .operation_read_field(op, DirectoryIdentity::at("DirectoryIdentity"))
        .operation_create(op, Directory::reference())
        .operation_write(op, DirectoryIdentity::at("DirectoryIdentity"))
        .application_mutation_binding::<Binding>()
}
impl WorthQueryApplicationRequiredOutputSource<Catalog, program::Connection> for Binding {
    fn demand_from_source(_: &Create) -> Result<Demand, WorthQueryRequiredOutputConnectionDenial> {
        Ok(Demand {
            directory: "A".into(),
            prefix: "match".into(),
        })
    }
}
pub(in super::super) struct Delivery;
impl ApplicationActionDelivery<Catalog, Binding> for Delivery {
    type Mode = ApplicationFixedDeliveryMode<program::Root, ApplicationNoProjection>;
    const IDENTITY: &'static str = "nested-required-support.directory-delivery.v1";
}
impl ApplicationFixedActionDelivery<Catalog, Binding> for Delivery {
    type Root = program::Root;
    type Projection = ApplicationNoProjection;
}
