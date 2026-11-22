//! One optional-input body per schedule operation; both write the same fields,
//! so the forged operation differs from the ordinary one only in its input.
pub use super::super::schema::*;
use super::binding::{ForgedDay, ForgedScheduleBinding, ScheduleBinding, ScheduleInputBinding};
use super::records::{
    DocumentDueDateField, DocumentOpensAtField, DocumentPayloadField, DocumentTokenField,
};
use worth_foundational::facade::CanonicalDate;
pub use worth_query_host::facade::{
    application_contribution::WorthQueryApplicationContributionSetup,
    declaration::{application_program::*, application_schema::*},
    primary_graph::WorthQueryPrimaryGraphInstallationDenial,
    worth_query_operation,
};

worth_query_operation!(pub AssignSchedule for Schema: BodySchemaBinding, input ScheduleInputBinding<CanonicalDate>);
worth_query_operation!(pub AssignForgedSchedule for Schema: BodySchemaBinding, input ScheduleInputBinding<ForgedDay>);

// Every fixture destination instantiates these entry points; the inline-const
// gates keep the schedule operations, bindings and query out of every
// destination but its own. The `DocumentScheduleFacts` aspect itself is part of
// every destination's `Document`, as optional aspects are declared per entity.
pub fn declare<const MODE: u64>(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<MODE>>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema<MODE>> {
    if const { MODE != super::MODE } {
        return schema;
    }
    let ordinary = AssignSchedule::reference()
        .body()
        .assign_optional_input(DocumentDueDateField::at("DocumentDueDateField"), "due")
        .unwrap()
        .assign_optional_input(DocumentOpensAtField::at("DocumentOpensAtField"), "opens")
        .unwrap()
        .assign_optional_input(DocumentTokenField::at("DocumentTokenField"), "token")
        .unwrap()
        .assign_optional_input(DocumentPayloadField::at("DocumentPayloadField"), "payload")
        .unwrap()
        .finish()
        .unwrap();
    let forged = AssignForgedSchedule::reference()
        .body()
        .assign_optional_input(DocumentDueDateField::at("DocumentDueDateField"), "due")
        .unwrap()
        .finish()
        .unwrap();
    schema
        .operation(
            AssignSchedule::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(ordinary)
                .unwrap(),
        )
        .operation(
            AssignForgedSchedule::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish()
                .with_body(forged)
                .unwrap(),
        )
        .operation_projection_work_budget(AssignSchedule::reference(), 16)
        .operation_projection_work_budget(AssignForgedSchedule::reference(), 8)
        .operation_read_field(
            AssignSchedule::reference(),
            DocumentIdentityField::at("DocumentIdentityField"),
        )
        .operation_read_field(
            AssignForgedSchedule::reference(),
            DocumentIdentityField::at("DocumentIdentityField"),
        )
        .application_mutation_binding::<ScheduleBinding<MODE>>()
        .application_mutation_binding::<ForgedScheduleBinding<MODE>>()
        .application_query(super::query::schedule_query_definition::<MODE>())
        .application_query_binding::<super::query::ScheduleQueryBinding<MODE>>()
}

pub fn configure<const MODE: u64>(
    setup: &mut WorthQueryApplicationContributionSetup<'_, DocumentRetentionSchema<MODE>>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    if const { MODE == super::MODE } {
        setup.operation_body::<ScheduleBinding<MODE>>()?;
        setup.operation_body::<ForgedScheduleBinding<MODE>>()?;
    }
    Ok(())
}

pub fn actions<const MODE: u64, Feature: ApplicationFeature<DocumentRetentionSchema<MODE>>>(
    feature: ApplicationFeatureSpecBuilder<
        DocumentRetentionSchema<MODE>,
        ApplicationRootComposition,
        Feature,
    >,
) -> ApplicationFeatureSpecBuilder<DocumentRetentionSchema<MODE>, ApplicationRootComposition, Feature>
{
    if const { MODE != super::MODE } {
        return feature;
    }
    feature
        .native_action::<ScheduleBinding<MODE>>()
        .finish_action()
        .native_action::<ForgedScheduleBinding<MODE>>()
        .finish_action()
}
