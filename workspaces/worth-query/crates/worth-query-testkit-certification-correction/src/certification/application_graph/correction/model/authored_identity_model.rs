//! Authored lifecycle commands share one logical key and ordinary native outputs.
use super::{
    edit::{DenialBinding, ResultBinding},
    schema::*,
};
use worth_query_decl::facade::worth_query_mutation_binding;
use worth_query_host::facade::{
    declaration::{application_operation::*, application_program::*, application_schema::*},
    primary_graph::*,
    *,
};

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Create {
    pub key: String,
    pub name: String,
    pub payload: u64,
    pub inverse_target: Option<String>,
    pub collateral: Option<String>,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Retire {
    pub key: String,
    pub target: String,
    pub wrong_restore_key: bool,
    pub collateral: Option<String>,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Restore {
    pub key: String,
    pub name: String,
    pub primary: u64,
    pub secondary: u64,
    pub read_enabled: bool,
    pub ceiling: u64,
    pub calculated: u64,
    pub collateral: Option<String>,
}
#[macro_export]
macro_rules! worlds_certification_application_graph_correction_model_authored_identity_model_rs_input {
    ($Input:ident, $Codec:ident, $Operation:ident, $id:literal) => {
        impl $Input { fn scope_key(input: &Self) -> String { input.key.clone() } }
        worth_query_structured_value_binding!(pub $Codec for $Input { identity: $id });
        worth_query_operation!(pub $Operation for NumericSchema, input $Codec);
    };
}
pub use crate::worlds_certification_application_graph_correction_model_authored_identity_model_rs_input as input;
input!(
    Create,
    CreateInput,
    CreateOperation,
    "authored-identity.create.input.v1"
);
input!(
    Retire,
    RetireInput,
    RetireAuthoredEntity,
    "authored-identity.retire.input.v1"
);
input!(
    Restore,
    RestoreInput,
    RestoreOperation,
    "authored-identity.restore.input.v1"
);
worth_query_operation_reads!(CreateOperation => [Key, Primary]);
worth_query_operation_writes!(CreateOperation => [Key, Primary, Secondary, ReadEnabled, Ceiling, Calculated]);
impl OperationCreates<CreateOperation> for Number {}
worth_query_relation!(pub AuthoredReference in NumericSchema, Number => Number; integrity = same_context_unbounded_retain_dangling);
impl OperationLinks<CreateOperation> for AuthoredReference {}
worth_query_operation_reads!(RestoreOperation => [Key, Primary]);
worth_query_operation_writes!(RestoreOperation => [Key, Primary, Secondary, ReadEnabled, Ceiling, Calculated]);
impl OperationCreates<RestoreOperation> for Number {}
worth_query_operation_reads!(RetireAuthoredEntity => [Number, Key, Primary, Secondary, ReadEnabled, Ceiling, Calculated]);
impl OperationDeletes<RetireAuthoredEntity> for Number {}

pub struct CreatedOutputs;
pub struct RetiredOutputs;
pub struct Created;
pub struct Retired;
pub struct CollateralCreated;
pub struct CollateralRetired;
#[macro_export]
macro_rules! worlds_certification_application_graph_correction_model_authored_identity_model_rs_output {
    ($Contract:ident, $Role:ident, $Action:ty, $name:literal, $Collateral:ty) => {
        impl ApplicationMutationOutputContract<NumericSchema> for $Contract {
            const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
                &[$Role::DESCRIPTOR, <$Collateral>::DESCRIPTOR];
        }
        impl WorthQueryApplicationOutputRole for $Role {
            type Schema = NumericSchema;
            type Contract = $Contract;
            type Entity = Number;
            type Action = $Action;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = $name;
        }
    };
}
pub use crate::worlds_certification_application_graph_correction_model_authored_identity_model_rs_output as output;
output!(
    CreatedOutputs,
    Created,
    WorthQueryCreateOutput,
    "authored-created",
    CollateralCreated
);
output!(
    RetiredOutputs,
    Retired,
    WorthQueryRetireOutput,
    "authored-retired",
    CollateralRetired
);
impl WorthQueryApplicationOutputRole for CollateralCreated {
    type Schema = NumericSchema;
    type Contract = CreatedOutputs;
    type Entity = Number;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "collateral-created";
}
impl WorthQueryApplicationOutputRole for CollateralRetired {
    type Schema = NumericSchema;
    type Contract = RetiredOutputs;
    type Entity = Number;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryAtMostOneOutput;
    const NAME: &'static str = "collateral-retired";
}
pub type Target = WorthQueryInvariantMutationTarget<NumericSchema, Number>;
pub struct Retirement {
    pub root: Target,
    pub collateral: Option<Target>,
}
#[macro_export]
macro_rules! worlds_certification_application_graph_correction_model_authored_identity_model_rs_binding {
    ($Binding:ident, $Input:ident, $Codec:ident, $Operation:ident, $Output:ty, $id:literal, $Decision:ty) => {
        worth_query_mutation_binding!(
            pub $Binding for $Input, schema NumericSchema,
            identity $id, input $Codec, operation $Operation, result ResultBinding,
            idempotency String, identity "authored-identity.command.v1",
            decision $Decision, denial DenialBinding,
            handler identity $id, outputs $Output,
            principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
            principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
            field Key::reference(), value $Input::scope_key,
        );
    }
}
pub use crate::worlds_certification_application_graph_correction_model_authored_identity_model_rs_binding as binding;
binding!(
    CreateBinding,
    Create,
    CreateInput,
    CreateOperation,
    CreatedOutputs,
    "authored-identity.create.v1",
    Target
);
binding!(
    RetireBinding,
    Retire,
    RetireInput,
    RetireAuthoredEntity,
    RetiredOutputs,
    "authored-identity.retire.v1",
    Retirement
);
binding!(
    RestoreBinding,
    Restore,
    RestoreInput,
    RestoreOperation,
    CreatedOutputs,
    "authored-identity.restore.v1",
    Target
);

#[macro_export]
macro_rules! worlds_certification_application_graph_correction_model_authored_identity_model_rs_creation_access {
    ($Operation:ty, $($link:tt)*) => {
        impl ApplicationDeclaredOperation<NumericSchema> for $Operation {
            fn declare_access(
                schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
            ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
                schema
                    .operation_read_field(Self::reference(), Key::reference())
                    .operation_read_field(Self::reference(), Primary::reference())
                    .operation_write(Self::reference(), Key::reference())
                    .operation_write(Self::reference(), Primary::reference())
                    .operation_write(Self::reference(), Secondary::reference())
                    .operation_write(Self::reference(), ReadEnabled::reference())
                    .operation_write(Self::reference(), Ceiling::reference())
                    .operation_write(Self::reference(), Calculated::reference())
                    .operation_create(Self::reference(), Number::reference())
                    $($link)*
            }
        }
    };
}
pub use crate::worlds_certification_application_graph_correction_model_authored_identity_model_rs_creation_access as creation_access;
creation_access!(CreateOperation, .operation_link(CreateOperation::reference(), AuthoredReference::reference()));
creation_access!(RestoreOperation,);
impl ApplicationDeclaredOperation<NumericSchema> for RetireAuthoredEntity {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_read_entity(Self::reference(), Number::reference())
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Primary::reference())
            .operation_read_field(Self::reference(), Secondary::reference())
            .operation_read_field(Self::reference(), ReadEnabled::reference())
            .operation_read_field(Self::reference(), Ceiling::reference())
            .operation_read_field(Self::reference(), Calculated::reference())
            .operation_delete(Self::reference(), Number::reference())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    RestoreOperation::declare_access(
        schema.operation(
            RestoreOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        ),
    )
    .application_mutation_binding::<RestoreBinding>()
    .relation(
        AuthoredReference::reference(),
        Number::reference(),
        Number::reference(),
    )
}
pub struct Feature;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "authored-identity.feature.v1";
}
pub struct Completion;
#[macro_export]
macro_rules! worlds_certification_application_graph_correction_model_authored_identity_model_rs_delivery {
    ($Binding:ty, $id:literal) => {
        impl ApplicationActionDelivery<NumericSchema, $Binding> for Completion {
            type Mode = ApplicationSourceCompletion;
            const IDENTITY: &'static str = $id;
        }
    };
}
pub use crate::worlds_certification_application_graph_correction_model_authored_identity_model_rs_delivery as delivery;
delivery!(CreateBinding, "authored-identity.create.completion.v1");
delivery!(RetireBinding, "authored-identity.retire.completion.v1");
delivery!(RestoreBinding, "authored-identity.restore.completion.v1");
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<super::observed_clone::Binding, Completion>()
        .finish_action()
        .action::<CreateBinding, Completion>()
        .finish_action()
        .action::<RetireBinding, Completion>()
        .finish_action()
        .action::<RestoreBinding, Completion>()
        .finish_action()
        .finish()
}
