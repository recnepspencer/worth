//! Deliberately inconsistent domain owners exercise ordinary write admission.
use super::schema::*;
use worth_query_host::facade::{declaration::application_schema::*, *};
mod binary;
mod canonical;
pub use binary::{BinaryCounter, BrokenBinaryBinding};
pub use canonical::{CanonicalBinding, CanonicalCounter, DelegatedBinding, DelegatedCounter};
mod operation;
pub use operation::{Binding, Encoding, Handler, Write};

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            operation::WriteOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation::WriteOperation::reference(), 8)
        .operation_read_field(operation::WriteOperation::reference(), Key::at("Key"))
        .operation_read_field(operation::WriteOperation::reference(), Binary::at("Binary"))
        .operation_read_field(
            operation::WriteOperation::reference(),
            Canonical::at("Canonical"),
        )
        .operation_read_field(
            operation::WriteOperation::reference(),
            Delegated::at("Delegated"),
        )
        .operation_write(operation::WriteOperation::reference(), Binary::at("Binary"))
        .operation_write(
            operation::WriteOperation::reference(),
            Canonical::at("Canonical"),
        )
        .operation_write(
            operation::WriteOperation::reference(),
            Delegated::at("Delegated"),
        )
        .application_mutation_binding::<Binding>()
}
pub fn feature(
) -> worth_query_host::facade::declaration::application_program::ApplicationFeatureSpec {
    use worth_query_host::facade::declaration::application_program::*;
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<Binding, Completion>()
        .finish_action()
        .finish()
}
pub struct Feature;
impl worth_query_host::facade::declaration::application_program::ApplicationFeature<NumericSchema>
    for Feature
{
    type Inputs =
        worth_query_host::facade::declaration::application_program::ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "domain-write.feature.v1";
}
pub struct Completion;
impl
    worth_query_host::facade::declaration::application_program::ApplicationActionDelivery<
        NumericSchema,
        Binding,
    > for Completion
{
    type Mode =
        worth_query_host::facade::declaration::application_program::ApplicationSourceCompletion;
    const IDENTITY: &'static str = "domain-write.completion.v1";
}

pub use crate::facade::certification::application_graph::correction::model::schema::storage_model::{WriteFacts, Binary, Canonical, Delegated};
