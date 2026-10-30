//! Ordinary committed projections expose issued model and target coordinates.
use super::schema::{data::Coordinates, *};
use worth_query_decl::facade::{application_query::*, application_schema::*, *};
use worth_query_host::facade::primary_graph::*;

pub struct Parameters;
worth_query_structured_value_binding!(pub ParametersBinding for Parameters { identity:"proposal-application.query.parameters.v1" });
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ModelRow {
    pub name: String,
    pub occurrence: Coordinates,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TargetRow {
    pub name: String,
    pub days: u64,
    pub occurrence: Coordinates,
}
worth_query_structured_value_binding!(pub ModelRowBinding for ModelRow { identity:"proposal-application.model-row.v1" });
worth_query_structured_value_binding!(pub TargetRowBinding for TargetRow { identity:"proposal-application.target-row.v1" });
worth_query_application_query!(pub ModelQuery for Schema, identity "proposal-application.model-query.v1",
    parameters ParametersBinding, result ModelRowBinding, scope Model => "Model", name "proposal_model");
worth_query_application_query!(pub TargetQuery for Schema, identity "proposal-application.target-query.v1",
    parameters ParametersBinding, result TargetRowBinding, scope Model => "Model", name "proposal_target");
pub struct ModelNameSlot;
pub struct TargetNameSlot;
pub struct DaysSlot;
worth_query_portable_type!(ModelNameSlot => "proposal-application.model-name-slot.v1");
worth_query_portable_type!(TargetNameSlot => "proposal-application.target-name-slot.v1");
worth_query_portable_type!(DaysSlot => "proposal-application.days-slot.v1");
fn model_name() -> ApplicationQueryResultFieldRef<
    ModelQuery,
    ModelNameSlot,
    Schema,
    Model,
    ModelFacts,
    ModelName,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("name", ModelName::reference())
}
fn target_name() -> ApplicationQueryResultFieldRef<
    TargetQuery,
    TargetNameSlot,
    Schema,
    Target,
    TargetFacts,
    TargetName,
    String,
    ReadOnly,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("name", TargetName::reference())
}
fn days() -> ApplicationQueryResultFieldRef<
    TargetQuery,
    DaysSlot,
    Schema,
    Target,
    TargetFacts,
    Retention,
    u64,
    ReadWrite,
    EqualityPredicate,
    NoApplicationUnit,
> {
    ApplicationQueryResultFieldRef::new("days", Retention::reference())
}
impl WorthQueryApplicationProjection<Schema, ModelQuery> for ModelRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, ModelQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let id = row.entity_id();
        Ok(Self {
            name: row.field(model_name())?,
            occurrence: Coordinates {
                partition: id.partition_value(),
                slot: id.local_slot_value(),
                generation: id.generation_value(),
            },
        })
    }
}
impl WorthQueryApplicationProjection<Schema, TargetQuery> for TargetRow {
    fn project(
        row: &WorthQueryApplicationProjectionRow<'_, Schema, TargetQuery>,
    ) -> Result<Self, WorthQueryApplicationProjectionDenial> {
        let id = row.entity_id();
        Ok(Self {
            name: row.field(target_name())?,
            days: row.field(days())?,
            occurrence: Coordinates {
                partition: id.partition_value(),
                slot: id.local_slot_value(),
                generation: id.generation_value(),
            },
        })
    }
}
#[derive(Clone, Debug, WorthValue)]
pub struct ReadModel {
    pub model: String,
}
#[derive(Clone, Debug, WorthValue)]
pub struct ReadTarget {
    pub model: String,
}
worth_query_structured_value_binding!(pub ReadModelInput for ReadModel { identity:"proposal-application.read-model.v1" });
worth_query_structured_value_binding!(pub ReadTargetInput for ReadTarget { identity:"proposal-application.read-target.v1" });
fn model_scope(input: ReadModel) -> String {
    input.model
}
fn target_scope(input: ReadTarget) -> String {
    input.model
}
worth_query_query_binding!(pub ModelBinding for ReadModel, schema Schema,
    identity "proposal-application.model-binding.v1", input ReadModelInput,
    query ModelQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result ModelRowBinding,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,
    field ModelName::reference(), value model_scope, limits results 1);
worth_query_query_binding!(pub TargetBinding for ReadTarget, schema Schema,
    identity "proposal-application.target-binding.v1", input ReadTargetInput,
    query TargetQuery, parameters ParametersBinding => |_| ApplicationQueryParameterSet::new(), result TargetRowBinding,
    principal PrincipalBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Model,ModelFacts,ModelName,String,ReadOnly,NoApplicationUnit,
    field ModelName::reference(), value target_scope, limits results 1);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let model = ApplicationQueryDefinitionBuilder::declare(ModelQuery::reference())
        .root(Model::reference())
        .scope(Model::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                ModelQuery,
                Model,
                ModelRow,
                ModelRowBinding,
            >::new(Model::reference())
            .field(model_name())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .unwrap();
    let target = ApplicationQueryDefinitionBuilder::declare(TargetQuery::reference())
        .root(Target::reference())
        .scope(Model::reference())
        .result_shape(
            ApplicationQueryResultShapeBuilder::<
                Schema,
                TargetQuery,
                Target,
                TargetRow,
                TargetRowBinding,
            >::new(Target::reference())
            .field(target_name())
            .field(days())
            .build(),
        )
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Model::reference()).forward(ModelTarget::reference()),
        )
        .build()
        .unwrap();
    schema
        .application_query(model)
        .application_query(target)
        .application_query_binding::<ModelBinding>()
        .application_query_binding::<TargetBinding>()
}
