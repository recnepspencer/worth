#[path = "schema/records.rs"]
mod storage_model;
pub use storage_model::*;
pub use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
pub use worth_query_host::facade::{declaration, primary_graph};
pub use worth_query_host::facade::{
    worth_query_application_query, worth_query_effect, worth_query_operation,
    worth_query_operation_emits, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_portable_type, worth_query_structured_value_binding,
};

pub const SCALED_AMENDMENT_PROJECTION_WORK_BUDGET: usize = 256;

#[path = "schema/live_intent_query.rs"]
pub mod live_intent_query;
#[allow(unused_imports)]
pub use live_intent_query::{
    temporal_intent_live_query_definition, IntentLiveQueryParameters, IntentLiveQueryResult,
    TemporalIntentLiveCause, TemporalIntentLiveQuery,
};

#[path = "schema/declaration.rs"]
mod composition;
pub use composition::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalInput(pub String);
worth_query_portable_type!(TemporalInput => "worth.query.test.host.temporal.input.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AmendTemporalInput {
    pub revision: u64,
    pub due: u64,
    pub lifecycle: String,
    pub input: String,
    pub gate: String,
}
worth_query_portable_type!(AmendTemporalInput => "worth.query.test.host.temporal.amend_input.v1");

worth_query_structured_value_binding!(pub TemporalInputBinding for TemporalInput { identity: "worth.query.test.host.temporal.input.v1" });
worth_query_operation!(pub ExecuteTemporal for TemporalHostSchema, input TemporalInputBinding);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalExecutionNotice {
    pub identity: String,
}
worth_query_portable_type!(TemporalExecutionNotice => "worth.query.test.host.temporal.execution_notice.v1");

worth_query_structured_value_binding!(pub TemporalExecutionNoticeBinding for TemporalExecutionNotice { identity: "worth.query.test.host.temporal.execution_notice.v1" });

worth_query_effect!(pub TemporalExecutionEffect for TemporalHostSchema, payload TemporalExecutionNoticeBinding);
worth_query_operation_emits!(ExecuteTemporal => [TemporalExecutionEffect]);
worth_query_structured_value_binding!(pub AmendTemporalInputBinding for AmendTemporalInput { identity: "worth.query.test.host.temporal.amend_input.v1" });
worth_query_operation!(pub AmendTemporal for TemporalHostSchema, input AmendTemporalInputBinding);
worth_query_operation!(pub AmendTemporalAndPublishDefinition for TemporalHostSchema, input AmendTemporalInputBinding);
worth_query_operation_emits!(AmendTemporalAndPublishDefinition => [TemporalAmendmentEffect]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalAmendmentNotice(pub String);
worth_query_portable_type!(TemporalAmendmentNotice => "worth.query.test.host.temporal.amendment_notice.v1");

worth_query_structured_value_binding!(pub TemporalAmendmentNoticeBinding for TemporalAmendmentNotice { identity: "worth.query.test.host.temporal.amendment_notice.v1" });

impl declaration::application_schema::ApplicationExternalEffectBinding
    for TemporalAmendmentNoticeBinding
{
    const PROTOCOL: declaration::application_schema::ApplicationExternalEffectProtocol =
        declaration::application_schema::ApplicationExternalEffectProtocol::new(
            worth_foundational::facade::BoundaryProtocolIdentity::new(
                "worth.query.test.temporal-amendment",
            ),
            worth_foundational::facade::BoundaryProtocolVersion::new(1),
        );
    const MAX_EXTERNAL_BYTES: u64 = 128;

    fn external_effect_bytes(value: &Self::Value) -> Vec<u8> {
        value.0.as_bytes().to_vec()
    }
}

worth_query_effect!(pub TemporalAmendmentEffect for TemporalHostSchema, payload TemporalAmendmentNoticeBinding);
worth_query_operation!(pub RevokeTemporalPrincipal for TemporalHostSchema, input TemporalInputBinding);
pub struct IntentQueryParameters;
pub struct IntentIdentitySlot;
pub struct IntentRevisionSlot;
pub struct IntentDueSlot;
pub struct IntentLifecycleSlot;
pub struct IntentInputSlot;
worth_query_portable_type!(IntentIdentitySlot => "worth.query.test.host.temporal.intent_identity_slot.v1");
worth_query_portable_type!(IntentRevisionSlot => "worth.query.test.host.temporal.intent_revision_slot.v1");
worth_query_portable_type!(IntentDueSlot => "worth.query.test.host.temporal.intent_due_slot.v1");
worth_query_portable_type!(IntentLifecycleSlot => "worth.query.test.host.temporal.intent_lifecycle_slot.v1");
worth_query_portable_type!(IntentInputSlot => "worth.query.test.host.temporal.intent_input_slot.v1");

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentQueryResult {
    pub identity: String,
    pub revision: u64,
    pub due: u64,
    pub lifecycle: String,
    pub input: String,
}
worth_query_portable_type!(IntentQueryResult => "worth.query.test.host.temporal.intent_result.v1");

worth_query_structured_value_binding!(pub IntentQueryParametersBinding for IntentQueryParameters { identity: "IntentQueryParameters" });
worth_query_structured_value_binding!(pub IntentQueryResultBinding for IntentQueryResult { identity: "worth.query.test.host.temporal.intent_result.v1" });
worth_query_application_query!(
    pub TemporalIntentQuery for TemporalHostSchema,
    identity "TemporalIntentQuery",
    parameters IntentQueryParametersBinding,
    result IntentQueryResultBinding,
    scope TemporalIntent => "TemporalIntent",
    name "temporal_intent_query"
);

#[path = "schema/current_read.rs"]
pub mod current_read;
pub use current_read::TemporalIntentCurrentReadBinding;

pub fn temporal_intent_query_definition() -> ApplicationQueryDefinitionBuilder<
    TemporalHostSchema,
    TemporalIntentQuery,
    IntentQueryParameters,
    IntentQueryResult,
    TemporalIntent,
> {
    let shape = ApplicationQueryResultShapeBuilder::<
        TemporalHostSchema,
        TemporalIntentQuery,
        TemporalIntent,
        IntentQueryResult,
        IntentQueryResultBinding,
    >::new(TemporalIntent::reference())
    .field(identity_result())
    .field(revision_result())
    .field(due_result())
    .field(lifecycle_result())
    .field(input_result())
    .build();
    ApplicationQueryDefinitionBuilder::declare(TemporalIntentQuery::reference())
        .root(TemporalIntent::reference())
        .scope(TemporalIntent::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::ExactlyOne)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
}

impl primary_graph::WorthQueryApplicationProjection<TemporalHostSchema, TemporalIntentQuery>
    for IntentQueryResult
{
    fn project(
        row: &primary_graph::WorthQueryApplicationProjectionRow<
            '_,
            TemporalHostSchema,
            TemporalIntentQuery,
        >,
    ) -> Result<Self, primary_graph::WorthQueryApplicationProjectionDenial> {
        Ok(Self {
            identity: row.field(identity_result())?,
            revision: row.field(revision_result())?,
            due: row.field(due_result())?,
            lifecycle: row.field(lifecycle_result())?,
            input: row.field(input_result())?,
        })
    }
}

pub type ResultField<Slot, Field> =
    ApplicationQueryResultFieldRef<TemporalIntentQuery, Slot, TemporalHostSchema, Field>;

pub fn identity_result() -> ResultField<IntentIdentitySlot, IntentIdentityField> {
    ApplicationQueryResultFieldRef::new("identity", IntentIdentityField::at("IntentIdentityField"))
}
pub fn revision_result() -> ResultField<IntentRevisionSlot, IntentRevisionField> {
    ApplicationQueryResultFieldRef::new("revision", IntentRevisionField::at("IntentRevisionField"))
}
pub fn due_result() -> ResultField<IntentDueSlot, IntentDueField> {
    ApplicationQueryResultFieldRef::new("due", IntentDueField::at("IntentDueField"))
}
pub fn lifecycle_result() -> ResultField<IntentLifecycleSlot, IntentLifecycleField> {
    ApplicationQueryResultFieldRef::new(
        "lifecycle",
        IntentLifecycleField::at("IntentLifecycleField"),
    )
}
pub fn input_result() -> ResultField<IntentInputSlot, IntentInputField> {
    ApplicationQueryResultFieldRef::new("input", IntentInputField::at("IntentInputField"))
}
