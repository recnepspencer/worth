//! Clone consumes parent fields; its recorded inverse retires the created child.
use super::{
    authored_identity_correction::Family,
    authored_identity_model::{
        Completion, Created, CreatedOutputs, Restore, Retire, Retired, Target,
    },
    edit::{DenialBinding, ResultBinding},
    query,
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{
        application_aftermath::*, application_operation::*, application_program::*,
        application_query::*, application_schema::*,
    },
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct CloneNumber {
    pub key: String,
    pub name: String,
}
impl CloneNumber {
    fn scope_key(input: &Self) -> String {
        input.key.clone()
    }
}
worth_query_structured_value_binding!(pub InputBinding for CloneNumber { identity: "numeric-observed-clone.input.v1" });
worth_query_operation!(pub CloneOperation for NumericSchema, input InputBinding);
worth_query_operation_reads!(CloneOperation => [Number]);

impl OperationCreates<CloneOperation> for Number {}

pub struct Snapshot {
    pub(super) parent: Target,
    pub(super) image: Restore,
}
worth_query_mutation_binding!(
    pub Binding for CloneNumber, schema NumericSchema,
    identity "numeric-observed-clone.binding.v1", input InputBinding,
    operation CloneOperation, result ResultBinding,
    idempotency String, identity "numeric-observed-clone.command.v1",
    decision Snapshot, denial DenialBinding,
    handler identity "numeric-observed-clone.handler.v1", outputs CreatedOutputs,
    principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
    principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Key,
    field Key::at("Key"), value CloneNumber::scope_key,
);
impl ApplicationDeclaredOperation<NumericSchema> for CloneOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_projection_work_budget(Self::reference(), 16)
            .operation_read_entity(Self::reference(), Number::reference())
            .operation_read_field(Self::reference(), Key::at("Key"))
            .operation_read_field(Self::reference(), Primary::at("Primary"))
            .operation_read_field(Self::reference(), Secondary::at("Secondary"))
            .operation_read_field(Self::reference(), ReadEnabled::at("ReadEnabled"))
            .operation_read_field(Self::reference(), Ceiling::at("Ceiling"))
            .operation_read_field(Self::reference(), Calculated::at("Calculated"))
            .operation_write(Self::reference(), Key::at("Key"))
            .operation_write(Self::reference(), Primary::at("Primary"))
            .operation_write(Self::reference(), Secondary::at("Secondary"))
            .operation_write(Self::reference(), ReadEnabled::at("ReadEnabled"))
            .operation_write(Self::reference(), Ceiling::at("Ceiling"))
            .operation_write(Self::reference(), Calculated::at("Calculated"))
            .operation_create(Self::reference(), Number::reference())
    }
}
impl ApplicationActionDelivery<NumericSchema, Binding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-observed-clone.completion.v1";
}

pub struct RegisterClone;
impl worth_query_decl::facade::application_schema::fragment::Registration<NumericSchema> for RegisterClone {
    fn register(
        schema: worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> Result<
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationBuilder<NumericSchema>,
        worth_query_decl::facade::application_schema::ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .declared_operation(
                CloneOperation::reference()
            .definition()
            .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .application_mutation_binding::<Binding>())
    }
}
