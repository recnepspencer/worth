//! Multi-intent temporal query traversing `IntentLiveTarget` from the scoped intent.
use super::*;
use worth_query_decl::facade::{
    application_query::{
        ApplicationQueryBasisSupport, ApplicationQueryCardinality,
        ApplicationQueryDefinitionBuilder, ApplicationQueryDisclosureContract,
        ApplicationQueryLaneEligibility, ApplicationQueryParameterSet,
        ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
        ApplicationQueryRootPath,
    },
    worth_query_application_query, worth_query_query_binding,
};

worth_query_application_query!(
    pub TemporalIntentTargetsQuery for TemporalHostSchema,
    identity "TemporalIntentTargetsQuery",
    parameters IntentQueryParametersBinding,
    result IntentQueryResultBinding,
    scope TemporalIntent => "TemporalIntent",
    name "temporal_intent_targets_query"
);

pub type TargetsResultField<Slot, Field> =
    ApplicationQueryResultFieldRef<TemporalIntentTargetsQuery, Slot, TemporalHostSchema, Field>;

fn target_identity() -> TargetsResultField<IntentIdentitySlot, IntentIdentityField> {
    ApplicationQueryResultFieldRef::new("identity", IntentIdentityField::at("IntentIdentityField"))
}
fn target_revision() -> TargetsResultField<IntentRevisionSlot, IntentRevisionField> {
    ApplicationQueryResultFieldRef::new("revision", IntentRevisionField::at("IntentRevisionField"))
}
fn target_due() -> TargetsResultField<IntentDueSlot, IntentDueField> {
    ApplicationQueryResultFieldRef::new("due", IntentDueField::at("IntentDueField"))
}
fn target_lifecycle() -> TargetsResultField<IntentLifecycleSlot, IntentLifecycleField> {
    ApplicationQueryResultFieldRef::new(
        "lifecycle",
        IntentLifecycleField::at("IntentLifecycleField"),
    )
}
fn target_input() -> TargetsResultField<IntentInputSlot, IntentInputField> {
    ApplicationQueryResultFieldRef::new("input", IntentInputField::at("IntentInputField"))
}

pub fn temporal_intent_targets_query_definition() -> ApplicationQueryDefinitionBuilder<
    TemporalHostSchema,
    TemporalIntentTargetsQuery,
    IntentQueryParameters,
    IntentQueryResult,
    TemporalIntent,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        TemporalHostSchema,
        TemporalIntentTargetsQuery,
        TemporalIntent,
        IntentQueryResult,
        IntentQueryResultBinding,
    >::new(TemporalIntent::reference())
    .field(target_identity())
    .field(target_revision())
    .field(target_due())
    .field(target_lifecycle())
    .field(target_input())
    .build();
    ApplicationQueryDefinitionBuilder::declare(TemporalIntentTargetsQuery::reference())
        .root(TemporalIntent::reference())
        .scope(TemporalIntent::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(TemporalIntent::reference())
                .forward(IntentLiveTarget::reference()),
        )
}

impl primary_graph::WorthQueryApplicationProjection<TemporalHostSchema, TemporalIntentTargetsQuery>
    for IntentQueryResult
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            TemporalHostSchema,
            TemporalIntentTargetsQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(target_identity())?,
            revision: row.field(target_revision())?,
            due: row.field(target_due())?,
            lifecycle: row.field(target_lifecycle())?,
            input: row.field(target_input())?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalIntentTargetsReadRequest {
    pub identity: String,
}

impl TemporalIntentTargetsReadRequest {
    pub fn into_identity(self) -> String {
        self.identity
    }
}

worth_query_structured_value_binding!(
    pub TemporalIntentTargetsReadRequestBinding for TemporalIntentTargetsReadRequest {
        identity: "worth.query.test.host.temporal.intent_targets_read_request.v1"
    }
);

worth_query_query_binding!(
    pub TemporalIntentTargetsReadBinding for TemporalIntentTargetsReadRequest,
    schema TemporalHostSchema,
    identity "worth.query.test.host.temporal.intent_targets_read.v1",
    input TemporalIntentTargetsReadRequestBinding,
    query TemporalIntentTargetsQuery,
    parameters IntentQueryParametersBinding => |_| ApplicationQueryParameterSet::new(),
    result IntentQueryResultBinding,
    principal TemporalPrincipalBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding worth_query_decl::facade::application_schema::U64ApplicationValueBinding,
    scope IntentIdentityField,
    field IntentIdentityField::at("IntentIdentityField"),
    value TemporalIntentTargetsReadRequest::into_identity,
    limits results 8
);
