//! Separate ordinary credit and debit actions with an explicit compensation binding.
use super::{
    edit::{DenialBinding, Edited, MissingValue, ResultBinding},
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

pub struct BalanceDecision {
    target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
    next: u64,
}
pub fn debit_contacts() -> usize {
    DEBITS.load(std::sync::atomic::Ordering::Relaxed)
}

macro_rules! movement {
    ($Input:ident, $Codec:ident, $Operation:ident, $Binding:ident, $Handler:ident, $Counter:ident,
     $arithmetic:ident, $input_id:literal, $binding_id:literal, $handler_id:literal) => {
        #[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
        pub struct $Input { pub key: String, pub amount: u64 }
        impl $Input {
            fn scope_key(input: &Self) -> String { input.key.clone() }
        }
        worth_query_structured_value_binding!(pub $Codec for $Input { identity: $input_id });
        worth_query_operation!(pub $Operation for NumericSchema, input $Codec);
        worth_query_operation_reads!($Operation => [Key, Primary]);
        worth_query_operation_writes!($Operation => [Primary]);
        impl ApplicationDeclaredOperation<NumericSchema> for $Operation {
            fn declare_access(schema: ApplicationSchemaDeclarationBuilder<NumericSchema>) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
                schema.operation_projection_work_budget(Self::reference(), 8)
                    .operation_read_field(Self::reference(), Key::reference())
                    .operation_read_field(Self::reference(), Primary::reference())
                    .operation_write(Self::reference(), Primary::reference())
            }
        }
        worth_query_mutation_binding!(
            pub $Binding for $Input, schema NumericSchema,
            identity $binding_id, input $Codec, operation $Operation, result ResultBinding,
            idempotency String, identity "numeric-compensation.command.v1",
            decision BalanceDecision, denial DenialBinding,
            handler identity $handler_id, outputs NoApplicationMutationOutputs,
            principal ActorBinding, mapping ExternalMapping, principal_entity Principal,
            principal_identity u64, identity_binding U64ApplicationValueBinding,
            scope Number, NumberFacts, Key, String, ReadOnly, NoApplicationUnit,
            field Key::reference(), value $Input::scope_key,
        );
        pub struct $Handler;
        static $Counter: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        impl OperationHandler<NumericSchema, $Binding> for $Handler {
            fn decide(&self, input: &$Input, reader: &mut DecisionReader<'_, '_, '_, NumericSchema, $Binding>) -> HandlerResult<BalanceDecision, MissingValue> {
                $Counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                let root = match reader.resolve_entity(Key::reference(), input.key.clone()) {
                    Ok(root) => root, Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                let balance = match reader.field(&root, Primary::reference()) {
                    Ok(Some(balance)) => balance,
                    Ok(None) => return HandlerResult::DomainDenied(MissingValue),
                    Err(error) => return HandlerResult::ExecutionDenied(error),
                };
                let Some(next) = balance.$arithmetic(input.amount) else { return HandlerResult::DomainDenied(MissingValue) };
                match reader.mutation_target(&root) {
                    Ok(target) => HandlerResult::Completed(BalanceDecision { target, next }),
                    Err(error) => HandlerResult::ExecutionDenied(error),
                }
            }
            fn build_candidate(&self, _: &$Input, decision: BalanceDecision, writer: &mut CandidateWriter<'_, NumericSchema, $Binding>) -> HandlerResult<Edited, MissingValue> {
                let target = match writer.projected_entity(&decision.target) {
                    Ok(target) => target,
                    Err(error) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
                };
                match writer.write_field(&target, Primary::reference(), decision.next) {
                    Ok(()) => HandlerResult::Completed(Edited),
                    Err(error) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(error)),
                }
            }
        }
    }
}
movement!(
    Credit,
    CreditInput,
    CreditOperation,
    CreditBinding,
    CreditHandler,
    CREDITS,
    checked_add,
    "numeric-compensation.credit.input.v1",
    "numeric-compensation.credit.binding.v1",
    "numeric-compensation.credit.handler.v1"
);
movement!(
    Debit,
    DebitInput,
    DebitOperation,
    DebitBinding,
    DebitHandler,
    DEBITS,
    checked_sub,
    "numeric-compensation.debit.input.v1",
    "numeric-compensation.debit.binding.v1",
    "numeric-compensation.debit.handler.v1"
);

pub struct NeutralizeCredit;
impl ApplicationCorrectionBinding<NumericSchema> for NeutralizeCredit {
    const IDENTITY: &'static str = "numeric-compensation.neutralize-credit.v1";
    type Source = CreditBinding;
    type InverseIntent = Debit;
    type Disclosure = query::ReadBinding;
    fn disclosure_parameters(
        _: &Credit,
    ) -> Result<ApplicationQueryParameterSet<query::ReadQuery>, ApplicationCorrectionDerivationDenial>
    {
        Ok(query::parameters())
    }
    fn source_definition(
    ) -> ApplicationOperationDefinitionBuilder<NumericSchema, CreditOperation, Credit, true, false>
    {
        CreditOperation::reference()
            .definition()
            .no_external_effect()
    }
    fn mechanism() -> DeclaredCorrectionMechanism<NumericSchema> {
        DeclaredCorrectionMechanism::compensation_binding::<Self>(
            DeclaredAftermathPostcondition::BusinessPostcondition {
                identity: "original-credit-neutralized".to_owned(),
            },
        )
        .expect("the numeric compensation declares a business postcondition")
    }
    fn derive_correction(
        original: &Credit,
        material: ApplicationCorrectionSourceMaterial<'_, NumericSchema>,
    ) -> Result<Debit, ApplicationCorrectionDerivationDenial> {
        if !matches!(
            material,
            ApplicationCorrectionSourceMaterial::Compensation(_)
        ) {
            return Err(ApplicationCorrectionDerivationDenial::UnsupportedSourceMeaning);
        }
        Ok(Debit {
            key: original.key.clone(),
            amount: original.amount,
        })
    }
    fn inverse_key(key: &str) -> Result<String, ApplicationValueEncodeDenial> {
        Ok(format!("compensating-debit:{key}"))
    }
    fn visit<V: ApplicationCorrectionVisitor<NumericSchema>>(
        dispatch: ApplicationCorrectionDispatch<NumericSchema, Self, V>,
    ) -> V::Output {
        dispatch.conventional()
    }
}

pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<NumericSchema>,
) -> ApplicationSchemaDeclarationBuilder<NumericSchema> {
    schema
        .declared_operation(
            DebitOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .application_mutation_binding::<DebitBinding>()
}

pub struct Feature;
impl ApplicationFeature<NumericSchema> for Feature {
    type Inputs = ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "numeric-compensation.feature.v1";
}
pub struct Completion;
impl ApplicationActionDelivery<NumericSchema, CreditBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-compensation.credit.completion.v1";
}
impl ApplicationActionDelivery<NumericSchema, DebitBinding> for Completion {
    type Mode = ApplicationSourceCompletion;
    const IDENTITY: &'static str = "numeric-compensation.debit.completion.v1";
}
pub fn feature() -> ApplicationFeatureSpec {
    ApplicationFeatureSpec::root::<NumericSchema, Feature>()
        .action::<CreditBinding, Completion>()
        .finish_action()
        .action::<DebitBinding, Completion>()
        .finish_action()
        .finish()
}
