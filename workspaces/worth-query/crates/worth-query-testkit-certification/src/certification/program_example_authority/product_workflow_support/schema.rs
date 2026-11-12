#[path = "schema/records.rs"]
mod storage_model;
pub use storage_model::*;
use worth_query_host::facade::declaration::application_query::{
    ApplicationQueryBasisSupport, ApplicationQueryCardinality, ApplicationQueryDefinitionBuilder,
    ApplicationQueryDisclosureContract, ApplicationQueryLaneEligibility,
    ApplicationQueryResultFieldRef, ApplicationQueryResultShapeBuilder,
};
use worth_query_host::facade::{declaration, primary_graph};
use worth_query_host::facade::{
    worth_query_application_query, worth_query_effect, worth_query_operation,
    worth_query_operation_emits, worth_query_portable_type, worth_query_structured_value_binding,
};

mod decision_reads;
pub mod integrity;
pub use integrity::TemporalIntegrity;

use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
#[derive(Clone, Copy)]
pub struct TemporalHostSchema;
impl Fragment for TemporalHostSchema {
    const NAME: &'static str = "TemporalHostSchema";
    type Bindings = NoBindings;
    type Members = (TemporalHostContribution,);
}
impl ApplicationSchema for TemporalHostSchema {
    const OWNER: &'static str = "temporal_host_courtroom";
    const NAME: &'static str = <Self as Fragment>::NAME;
    const MAJOR: u32 = 1;
    const MINOR: u32 = 0;
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        fragment::declare(Self)
    }
    fn visit_correction<
        V: worth_query_decl::facade::application_aftermath::ApplicationCorrectionVisitor<Self>,
    >(
        source: std::any::TypeId,
        visitor: V,
    ) -> V::Output {
        fragment::visit_correction(Self, source, visitor)
    }
}
impl TemporalHostSchema {
    pub fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        <Self as ApplicationSchema>::declaration()
    }
}
impl ApplicationSchemaComposition for TemporalHostSchema {
    type Contributions = <Self as Fragment>::Members;
}

pub struct TemporalHostContribution;
impl ApplicationSchemaContribution<TemporalHostSchema> for TemporalHostContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("temporal_host_courtroom.example.v1");
    type Members = (TemporalModel, Contracts);
}
impl TemporalHostContribution {
    pub const fn reference() -> ApplicationSchemaContributionRef<TemporalHostSchema, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct Contracts;
impl Registration<TemporalHostSchema> for Contracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<TemporalHostSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok({
            let schema = decision_reads::declare(schema)?;
            let schema = schema
                .principal_binding(TemporalPrincipalBinding::reference())
                .effect(TemporalExecutionEffect::reference())
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
                    RevokeTemporalPrincipal::reference()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_projection_work_budget(RevokeTemporalPrincipal::reference(), 3)
                .operation_read_field(
                    RevokeTemporalPrincipal::reference(),
                    MappingStatusField::at("MappingStatusField"),
                )
                .operation_write(
                    RevokeTemporalPrincipal::reference(),
                    MappingStatusField::at("MappingStatusField"),
                )
                .operation_projection_work_budget(ExecuteTemporal::reference(), 16)
                .operation_read_field(
                    ExecuteTemporal::reference(),
                    IntentEffectField::at("IntentEffectField"),
                )
                .operation_write(
                    ExecuteTemporal::reference(),
                    IntentRevisionField::at("IntentRevisionField"),
                )
                .operation_write(
                    ExecuteTemporal::reference(),
                    IntentLifecycleField::at("IntentLifecycleField"),
                )
                .operation_write(
                    ExecuteTemporal::reference(),
                    IntentEffectField::at("IntentEffectField"),
                )
                .operation_emit(
                    ExecuteTemporal::reference(),
                    TemporalExecutionEffect::reference(),
                )
                .operation_projection_work_budget(AmendTemporal::reference(), 12)
                .operation_read_field(
                    AmendTemporal::reference(),
                    IntentGateField::at("IntentGateField"),
                )
                .operation_read_field(
                    AmendTemporal::reference(),
                    IntentDueField::at("IntentDueField"),
                )
                .operation_read_field(
                    AmendTemporal::reference(),
                    IntentInputField::at("IntentInputField"),
                )
                .operation_write(
                    AmendTemporal::reference(),
                    IntentRevisionField::at("IntentRevisionField"),
                )
                .operation_write(
                    AmendTemporal::reference(),
                    IntentLifecycleField::at("IntentLifecycleField"),
                )
                .operation_write(
                    AmendTemporal::reference(),
                    IntentGateField::at("IntentGateField"),
                )
                .operation_write(
                    AmendTemporal::reference(),
                    IntentDueField::at("IntentDueField"),
                )
                .operation_write(
                    AmendTemporal::reference(),
                    IntentInputField::at("IntentInputField"),
                )
                .invariant(integrity::definition())
                .application_query(temporal_intent_query_definition());
            super::application_entry::declare(schema)
        })
    }
}

pub struct TemporalPrincipalBinding;
impl TemporalPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        TemporalHostSchema,
        Self,
        ExternalMapping,
        Principal,
        <PrincipalIdentityField as RecordSelection>::Value,
        <PrincipalIdentityField as RecordSelection>::Binding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "TemporalPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdentityField::at("PrincipalIdentityField"),
                ),
            },
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemporalInput(pub String);
worth_query_portable_type!(TemporalInput => "worth.query.test.host.temporal.input.v1");

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
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
