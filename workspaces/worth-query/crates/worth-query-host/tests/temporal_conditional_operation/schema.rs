use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinition,
    ApplicationQueryDefinitionBuilder, ApplicationQueryDependencyCeiling,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
use worth_query_host::facade::{declaration, primary_graph};
use worth_query_host::facade::{
    worth_query_application_query, worth_query_application_schema, worth_query_aspect,
    worth_query_effect, worth_query_entity, worth_query_field, worth_query_operation,
    worth_query_operation_emits, worth_query_operation_reads, worth_query_operation_writes,
    worth_query_portable_type, worth_query_principal_binding, worth_query_relation,
};

worth_query_application_schema! {
    pub schema TemporalHostSchema {
        owner: temporal_host_courtroom,
        version: (1, 0),
        members: |schema| {
            schema
                .entity(ExternalMapping::reference())
                .entity(Principal::reference())
                .entity(TemporalIntent::reference())
                .entity(UnrelatedRecord::reference())
                .aspect(ExternalMapping::reference(), ExternalIdentity::reference())
                .aspect(Principal::reference(), PrincipalFacts::reference())
                .aspect(TemporalIntent::reference(), IntentFacts::reference())
                .aspect(UnrelatedRecord::reference(), UnrelatedFacts::reference())
                .field(ExternalMapping::reference(), ExternalIdentityField::reference())
                .field(ExternalMapping::reference(), MappingStatusField::reference())
                .field(Principal::reference(), PrincipalIdentityField::reference())
                .field(TemporalIntent::reference(), IntentIdentityField::reference())
                .field(TemporalIntent::reference(), IntentRevisionField::reference())
                .field(TemporalIntent::reference(), IntentDueField::reference())
                .field(TemporalIntent::reference(), IntentLifecycleField::reference())
                .field(TemporalIntent::reference(), IntentInputField::reference())
                .field(TemporalIntent::reference(), IntentGateField::reference())
                .field(TemporalIntent::reference(), IntentEffectField::reference())
                .field(UnrelatedRecord::reference(), UnrelatedValueField::reference())
                .relation(MappingTarget::reference(), ExternalMapping::reference(), Principal::reference())
                .principal_binding(TemporalPrincipalBinding::reference())
                .effect(TemporalAmendmentEffect::reference())
                .operation(
                    ExecuteTemporal::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation(
                    AmendTemporal::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation(
                    AmendTemporalAndPublishDefinition::reference()
                        .definition()
                        .external_effect(
                            TemporalAmendmentEffect::reference(),
                            worth_query_host::facade::domain::WorthQueryExternalEffectCorrelationFamily::new(
                                "temporal-amendment",
                            )
                            .unwrap(),
                        )
                        .no_aftermath()
                        .finish(),
                )
                .operation(RevokeTemporalPrincipal::reference().definition().no_external_effect().no_aftermath().finish())
                .operation_decision_fact_budget(RevokeTemporalPrincipal::reference(), 1)
                .operation_projection_work_budget(RevokeTemporalPrincipal::reference(), 3)
                .operation_read_field(RevokeTemporalPrincipal::reference(), MappingStatusField::reference())
                .operation_write(RevokeTemporalPrincipal::reference(), MappingStatusField::reference())
                .operation_decision_fact_budget(ExecuteTemporal::reference(), 4)
                .operation_projection_work_budget(ExecuteTemporal::reference(), 16)
                .operation_read_field(ExecuteTemporal::reference(), IntentIdentityField::reference())
                .operation_read_field(ExecuteTemporal::reference(), IntentRevisionField::reference())
                .operation_read_field(ExecuteTemporal::reference(), IntentLifecycleField::reference())
                .operation_read_field(ExecuteTemporal::reference(), IntentEffectField::reference())
                .operation_write(ExecuteTemporal::reference(), IntentRevisionField::reference())
                .operation_write(ExecuteTemporal::reference(), IntentLifecycleField::reference())
                .operation_write(ExecuteTemporal::reference(), IntentEffectField::reference())
                .operation_decision_fact_budget(AmendTemporal::reference(), 5)
                .operation_projection_work_budget(AmendTemporal::reference(), 12)
                .operation_read_field(AmendTemporal::reference(), IntentRevisionField::reference())
                .operation_read_field(AmendTemporal::reference(), IntentLifecycleField::reference())
                .operation_read_field(AmendTemporal::reference(), IntentGateField::reference())
                .operation_read_field(AmendTemporal::reference(), IntentDueField::reference())
                .operation_read_field(AmendTemporal::reference(), IntentInputField::reference())
                .operation_write(AmendTemporal::reference(), IntentRevisionField::reference())
                .operation_write(AmendTemporal::reference(), IntentLifecycleField::reference())
                .operation_write(AmendTemporal::reference(), IntentGateField::reference())
                .operation_write(AmendTemporal::reference(), IntentDueField::reference())
                .operation_write(AmendTemporal::reference(), IntentInputField::reference())
                .operation_decision_fact_budget(AmendTemporalAndPublishDefinition::reference(), 5)
                .operation_projection_work_budget(AmendTemporalAndPublishDefinition::reference(), 12)
                .operation_read_field(AmendTemporalAndPublishDefinition::reference(), IntentRevisionField::reference())
                .operation_read_field(AmendTemporalAndPublishDefinition::reference(), IntentLifecycleField::reference())
                .operation_read_field(AmendTemporalAndPublishDefinition::reference(), IntentGateField::reference())
                .operation_read_field(AmendTemporalAndPublishDefinition::reference(), IntentDueField::reference())
                .operation_read_field(AmendTemporalAndPublishDefinition::reference(), IntentInputField::reference())
                .operation_write(AmendTemporalAndPublishDefinition::reference(), IntentRevisionField::reference())
                .operation_write(AmendTemporalAndPublishDefinition::reference(), IntentLifecycleField::reference())
                .operation_write(AmendTemporalAndPublishDefinition::reference(), IntentGateField::reference())
                .operation_write(AmendTemporalAndPublishDefinition::reference(), IntentDueField::reference())
                .operation_write(AmendTemporalAndPublishDefinition::reference(), IntentInputField::reference())
                .operation_emit(AmendTemporalAndPublishDefinition::reference(), TemporalAmendmentEffect::reference())
                .application_query(temporal_intent_query_definition())
        }
    }
}

worth_query_entity!(pub ExternalMapping in TemporalHostSchema);
worth_query_entity!(pub Principal in TemporalHostSchema);
worth_query_entity!(pub TemporalIntent in TemporalHostSchema);
worth_query_entity!(pub UnrelatedRecord in TemporalHostSchema);
worth_query_aspect!(pub ExternalIdentity in TemporalHostSchema, ExternalMapping; identity = AspectIdentity(0x91611042), revision = AspectContractRevision(1),);
worth_query_aspect!(pub PrincipalFacts in TemporalHostSchema, Principal; identity = AspectIdentity(0x91611043), revision = AspectContractRevision(1),);
worth_query_aspect!(pub IntentFacts in TemporalHostSchema, TemporalIntent; identity = AspectIdentity(0x91611044), revision = AspectContractRevision(1),);
worth_query_aspect!(pub UnrelatedFacts in TemporalHostSchema, UnrelatedRecord; identity = AspectIdentity(0x91611045), revision = AspectContractRevision(1),);
worth_query_field!(pub ExternalIdentityField in TemporalHostSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryExternalPrincipalIdentity, read_only, equality);
worth_query_field!(pub MappingStatusField in TemporalHostSchema, ExternalMapping, ExternalIdentity: declaration::authentication::WorthQueryPrincipalMappingStatus, read_write, equality);
worth_query_field!(pub PrincipalIdentityField in TemporalHostSchema, Principal, PrincipalFacts: u64, read_only, equality);
worth_query_field!(pub IntentIdentityField in TemporalHostSchema, TemporalIntent, IntentFacts: String, read_only, equality);
worth_query_field!(pub IntentRevisionField in TemporalHostSchema, TemporalIntent, IntentFacts: u64, read_write, equality);
worth_query_field!(pub IntentDueField in TemporalHostSchema, TemporalIntent, IntentFacts: u64, read_write, equality);
worth_query_field!(pub IntentLifecycleField in TemporalHostSchema, TemporalIntent, IntentFacts: String, read_write, equality);
worth_query_field!(pub IntentInputField in TemporalHostSchema, TemporalIntent, IntentFacts: String, read_write, equality);
worth_query_field!(pub IntentGateField in TemporalHostSchema, TemporalIntent, IntentFacts: String, read_write, equality);
worth_query_field!(pub IntentEffectField in TemporalHostSchema, TemporalIntent, IntentFacts: String, read_write, equality);
worth_query_field!(pub UnrelatedValueField in TemporalHostSchema, UnrelatedRecord, UnrelatedFacts: u64, read_only, equality);
worth_query_relation!(pub MappingTarget in TemporalHostSchema, ExternalMapping => Principal);
worth_query_principal_binding!(
    pub TemporalPrincipalBinding in TemporalHostSchema,
    mapping ExternalMapping {
        identity: ExternalIdentityField,
        status: MappingStatusField,
        target: MappingTarget => Principal,
        principal_identity: PrincipalIdentityField
    }
);

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

worth_query_operation!(pub ExecuteTemporal(TemporalInput) in TemporalHostSchema);
worth_query_operation_reads!(ExecuteTemporal => [IntentIdentityField, IntentRevisionField, IntentLifecycleField, IntentEffectField]);
worth_query_operation_writes!(ExecuteTemporal => [IntentRevisionField, IntentLifecycleField, IntentEffectField]);
worth_query_operation!(pub AmendTemporal(AmendTemporalInput) in TemporalHostSchema);
worth_query_operation_reads!(AmendTemporal => [IntentRevisionField, IntentDueField, IntentLifecycleField, IntentInputField, IntentGateField]);
worth_query_operation_writes!(AmendTemporal => [IntentRevisionField, IntentDueField, IntentLifecycleField, IntentInputField, IntentGateField]);
worth_query_operation!(pub AmendTemporalAndPublishDefinition(AmendTemporalInput) in TemporalHostSchema);
worth_query_operation_reads!(AmendTemporalAndPublishDefinition => [IntentRevisionField, IntentDueField, IntentLifecycleField, IntentInputField, IntentGateField]);
worth_query_operation_writes!(AmendTemporalAndPublishDefinition => [IntentRevisionField, IntentDueField, IntentLifecycleField, IntentInputField, IntentGateField]);
worth_query_operation_emits!(AmendTemporalAndPublishDefinition => [TemporalAmendmentEffect]);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalAmendmentNotice(pub String);
worth_query_portable_type!(TemporalAmendmentNotice => "worth.query.test.host.temporal.amendment_notice.v1");

impl declaration::application_schema::ApplicationEffectPayload for TemporalAmendmentNotice {
    fn retained_bytes(&self) -> u64 {
        u64::try_from(self.0.len()).unwrap_or(u64::MAX)
    }
}

impl declaration::application_schema::ApplicationExternalEffectPayload for TemporalAmendmentNotice {
    const PROTOCOL: declaration::application_schema::ApplicationExternalEffectProtocol =
        declaration::application_schema::ApplicationExternalEffectProtocol::new(
            worth_foundational::facade::BoundaryProtocolIdentity::new(
                "worth.query.test.temporal-amendment",
            ),
            worth_foundational::facade::BoundaryProtocolVersion::new(1),
        );
    const MAX_EXTERNAL_BYTES: u64 = 128;

    fn external_effect_bytes(&self) -> Vec<u8> {
        self.0.as_bytes().to_vec()
    }
}

worth_query_effect!(pub TemporalAmendmentEffect(TemporalAmendmentNotice) in TemporalHostSchema);
worth_query_operation!(pub RevokeTemporalPrincipal(TemporalInput) in TemporalHostSchema);
worth_query_operation_reads!(RevokeTemporalPrincipal => [MappingStatusField]);
worth_query_operation_writes!(RevokeTemporalPrincipal => [MappingStatusField]);
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

worth_query_application_query!(
    pub TemporalIntentQuery in TemporalHostSchema,
    parameters IntentQueryParameters,
    result IntentQueryResult,
    scope TemporalIntent,
    name "temporal_intent_query"
);

pub fn temporal_intent_query_definition() -> ApplicationQueryDefinition<
    TemporalHostSchema,
    TemporalIntentQuery,
    IntentQueryParameters,
    IntentQueryResult,
    TemporalIntent,
> {
    let shape = ApplicationQueryResultShapeBuilder::new(TemporalIntent::reference())
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
        .dependency_ceiling(ApplicationQueryDependencyCeiling::bounded(0, 0, 5))
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .build()
        .expect("temporal intent query is canonical")
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

type ResultField<Slot, Field, Value, Write> = ApplicationQueryResultFieldRef<
    TemporalIntentQuery,
    Slot,
    TemporalHostSchema,
    TemporalIntent,
    IntentFacts,
    Field,
    Value,
    Write,
    declaration::application_schema::EqualityPredicate,
    declaration::application_schema::NoApplicationUnit,
>;

fn identity_result() -> ResultField<
    IntentIdentitySlot,
    IntentIdentityField,
    String,
    declaration::application_schema::ReadOnly,
> {
    ApplicationQueryResultFieldRef::new("identity", IntentIdentityField::reference())
}
fn revision_result() -> ResultField<
    IntentRevisionSlot,
    IntentRevisionField,
    u64,
    declaration::application_schema::ReadWrite,
> {
    ApplicationQueryResultFieldRef::new("revision", IntentRevisionField::reference())
}
fn due_result(
) -> ResultField<IntentDueSlot, IntentDueField, u64, declaration::application_schema::ReadWrite> {
    ApplicationQueryResultFieldRef::new("due", IntentDueField::reference())
}
fn lifecycle_result() -> ResultField<
    IntentLifecycleSlot,
    IntentLifecycleField,
    String,
    declaration::application_schema::ReadWrite,
> {
    ApplicationQueryResultFieldRef::new("lifecycle", IntentLifecycleField::reference())
}
fn input_result() -> ResultField<
    IntentInputSlot,
    IntentInputField,
    String,
    declaration::application_schema::ReadWrite,
> {
    ApplicationQueryResultFieldRef::new("input", IntentInputField::reference())
}
