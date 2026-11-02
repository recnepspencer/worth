//! Deliberately inconsistent domain owners exercise ordinary write admission.
use super::schema::*;
use worth_query_host::facade::{declaration::application_schema::*, *};
mod binary;
mod canonical;
pub use binary::BrokenBinaryBinding;
pub use canonical::{CanonicalBinding, CanonicalCounter, DelegatedBinding};
mod operation;
pub use operation::{Binding, Encoding, Handler, Write};

worth_query_aspect!(pub WriteFacts for NumericSchema, Number; identity = AspectIdentity(0x91800109), revision = AspectContractRevision(1));
worth_query_field!(pub Binary for NumericSchema, Number, WriteFacts: u64 => BrokenBinaryBinding, read_write, no_equality);

worth_query_field!(pub Canonical for NumericSchema, Number, WriteFacts: CanonicalCounter => CanonicalBinding, read_write, no_equality);
worth_query_field!(pub Delegated for NumericSchema, Number, WriteFacts: u64 => DelegatedBinding, read_write, no_equality);

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .optional_aspect(Number::reference(), WriteFacts::reference())
        .field(Number::reference(), Binary::reference())
        .field(Number::reference(), Canonical::reference())
        .field(Number::reference(), Delegated::reference())
        .operation(
            operation::WriteOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation::WriteOperation::reference(), 8)
        .operation_read_field(operation::WriteOperation::reference(), Key::reference())
        .operation_read_field(operation::WriteOperation::reference(), Binary::reference())
        .operation_read_field(
            operation::WriteOperation::reference(),
            Canonical::reference(),
        )
        .operation_read_field(
            operation::WriteOperation::reference(),
            Delegated::reference(),
        )
        .operation_write(operation::WriteOperation::reference(), Binary::reference())
        .operation_write(
            operation::WriteOperation::reference(),
            Canonical::reference(),
        )
        .operation_write(
            operation::WriteOperation::reference(),
            Delegated::reference(),
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
