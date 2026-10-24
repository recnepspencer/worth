//! A generated authored output and a same-shaped decoy under real output roles.
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
pub struct Issue {
    pub key: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Issued {
    pub key: String,
    pub decoy: String,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Record {
    pub key: String,
    pub original_key: String,
    pub payload: u64,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Recorded {
    pub original_key: String,
    pub payload: u64,
}
#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct Retire {
    pub key: String,
    pub output_key: String,
}
impl Issue {
    fn scope_key(input: &Self) -> String {
        input.key.clone()
    }
}
impl Record {
    fn scope_key(input: &Self) -> String {
        input.key.clone()
    }
}
impl Retire {
    fn scope_key(input: &Self) -> String {
        input.key.clone()
    }
}
worth_query_structured_value_binding!(pub IssueInput for Issue { identity: "source-output.issue.input.v1" });
worth_query_structured_value_binding!(pub IssueResult for Issued { identity: "source-output.issue.result.v1" });
worth_query_structured_value_binding!(pub RecordInput for Record { identity: "source-output.record.input.v1" });
worth_query_structured_value_binding!(pub RecordResult for Recorded { identity: "source-output.record.result.v1" });
worth_query_structured_value_binding!(pub RetireInput for Retire { identity: "source-output.retire.input.v1" });
worth_query_operation!(pub IssueOperation for NumericSchema, input IssueInput);
worth_query_operation!(pub RecordOperation for NumericSchema, input RecordInput);
worth_query_operation!(pub RetireOperation for NumericSchema, input RetireInput);
worth_query_operation_reads!(IssueOperation => [Key, Secondary]);
worth_query_operation_writes!(IssueOperation => [Key, Primary, Secondary, ReadEnabled, Ceiling, Calculated]);
impl OperationCreates<IssueOperation> for Number {}
worth_query_operation_reads!(RecordOperation => [Key, Primary]);
worth_query_operation_writes!(RecordOperation => [Primary]);
worth_query_operation_reads!(RetireOperation => [Number, Key, Primary, Secondary, ReadEnabled, Ceiling, Calculated]);
impl OperationDeletes<RetireOperation> for Number {}

pub struct IssueDecision {
    pub target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
    pub serial: u64,
}
pub struct Outputs;
impl ApplicationMutationOutputContract<NumericSchema> for Outputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[Actual::DESCRIPTOR, Decoy::DESCRIPTOR];
}
pub struct Actual;
pub struct Decoy;
macro_rules! role {
    ($Role:ident, $name:literal) => {
        impl WorthQueryApplicationOutputRole for $Role {
            type Schema = NumericSchema;
            type Contract = Outputs;
            type Entity = Number;
            type Action = WorthQueryCreateOutput;
            type Cardinality = WorthQueryExactlyOneOutput;
            const NAME: &'static str = $name;
        }
    };
}
role!(Actual, "issued-record");
role!(Decoy, "decoy-record");
pub struct RetiredOutputs;
pub struct Retired;
impl ApplicationMutationOutputContract<NumericSchema> for RetiredOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[Retired::DESCRIPTOR];
}
impl WorthQueryApplicationOutputRole for Retired {
    type Schema = NumericSchema;
    type Contract = RetiredOutputs;
    type Entity = Number;
    type Action = WorthQueryRetireOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "retired-record";
}
macro_rules! binding {
    ($Binding:ident, $Input:ident, $Codec:ident, $Operation:ident, $Result:ident, $Decision:ty, $Output:ty, $id:literal) => {
        worth_query_mutation_binding!(
            pub $Binding for $Input, schema NumericSchema,
            identity $id, input $Codec, operation $Operation, result $Result,
            idempotency String, identity "source-output.command.v1",
            decision $Decision, denial DenialBinding,
            handler identity $id, outputs $Output,
            principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
            principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
            field Key::reference(), value $Input::scope_key,
        );
    }
}
binding!(
    IssueBinding,
    Issue,
    IssueInput,
    IssueOperation,
    IssueResult,
    IssueDecision,
    Outputs,
    "source-output.issue.v1"
);
binding!(RecordBinding, Record, RecordInput, RecordOperation, RecordResult, WorthQueryInvariantMutationTarget<NumericSchema, Number>, NoApplicationMutationOutputs, "source-output.record.v1");
binding!(RetireBinding, Retire, RetireInput, RetireOperation, ResultBinding, WorthQueryInvariantMutationTarget<NumericSchema, Number>, RetiredOutputs, "source-output.retire.v1");

impl ApplicationDeclaredOperation<NumericSchema> for IssueOperation {
    fn declare_access(
        schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
    ) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
        schema
            .operation_read_field(Self::reference(), Key::reference())
            .operation_read_field(Self::reference(), Secondary::reference())
            .operation_write(Self::reference(), Key::reference())
            .operation_write(Self::reference(), Primary::reference())
            .operation_write(Self::reference(), Secondary::reference())
            .operation_write(Self::reference(), ReadEnabled::reference())
            .operation_write(Self::reference(), Ceiling::reference())
            .operation_write(Self::reference(), Calculated::reference())
            .operation_create(Self::reference(), Number::reference())
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .operation(
            RecordOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(RecordOperation::reference(), Key::reference())
        .operation_read_field(RecordOperation::reference(), Primary::reference())
        .operation_write(RecordOperation::reference(), Primary::reference())
        .application_mutation_binding::<RecordBinding>()
        .operation(
            RetireOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_entity(RetireOperation::reference(), Number::reference())
        .operation_read_field(RetireOperation::reference(), Key::reference())
        .operation_read_field(RetireOperation::reference(), Primary::reference())
        .operation_read_field(RetireOperation::reference(), Secondary::reference())
        .operation_read_field(RetireOperation::reference(), ReadEnabled::reference())
        .operation_read_field(RetireOperation::reference(), Ceiling::reference())
        .operation_read_field(RetireOperation::reference(), Calculated::reference())
        .operation_delete(RetireOperation::reference(), Number::reference())
        .application_mutation_binding::<RetireBinding>()
}

pub struct Feature;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "source-output.feature.v1";
}
pub struct Completion;
macro_rules! delivery {
    ($B:ty, $id:literal) => {
        impl ApplicationActionDelivery<NumericSchema, $B> for Completion {
            type Mode = ApplicationSourceCompletion;
            const IDENTITY: &'static str = $id;
        }
    };
}
delivery!(IssueBinding, "source-output.issue.completion.v1");
delivery!(RecordBinding, "source-output.record.completion.v1");
delivery!(RetireBinding, "source-output.retire.completion.v1");
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<IssueBinding, Completion>()
        .finish_action()
        .action::<RecordBinding, Completion>()
        .finish_action()
        .action::<RetireBinding, Completion>()
        .finish_action()
        .finish()
}
