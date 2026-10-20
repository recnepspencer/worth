//! One real handler carries measured installed metadata into the actual writer.
use super::*;
use crate::domain_computation::primary_graph::application_entry::mutation::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryInvariantMutationTarget,
};
use worth_query_declaration::facade::{application_operation::*, application_schema::*};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct QuotedInput {
    pub value: String,
    pub underquote: bool,
    pub write_order: QuotedWriteOrder,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum QuotedWriteOrder {
    RequiredOnly,
    RequiredThenOptional,
    OptionalThenRequiredAndReplacement,
    OptionalClearThenSet,
    OptionalClearThenClear,
}
impl QuotedWriteOrder {
    fn has_optional(self) -> bool {
        self != Self::RequiredOnly
    }
    fn replaces_required(self) -> bool {
        self == Self::OptionalThenRequiredAndReplacement
    }
    fn clears_optional(self) -> bool {
        matches!(
            self,
            Self::OptionalClearThenSet | Self::OptionalClearThenClear
        )
    }
    fn write_count(self) -> usize {
        match self {
            Self::RequiredOnly => 3,
            Self::RequiredThenOptional => 4,
            Self::OptionalThenRequiredAndReplacement
            | Self::OptionalClearThenSet
            | Self::OptionalClearThenClear => 5,
        }
    }
}
worth_query_declaration::worth_query_portable_type!(QuotedInput => "worth.query.test.installed-quote-input.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub QuotedInputBinding for QuotedInput {
    identity: "worth.query.test.installed-quote-input.v1"
});
worth_query_declaration::worth_query_operation!(pub QuotedOperation for ReservationSchema, input QuotedInputBinding);
worth_query_declaration::worth_query_operation_requires!(QuotedOperation => [ViewAccount]);
worth_query_declaration::worth_query_operation_reads!(QuotedOperation => [Account, AccountStatus, AccountNote]);
worth_query_declaration::worth_query_operation_expects_fact!(QuotedOperation => [AccountStatus, AccountNote]);
worth_query_declaration::worth_query_operation_creates!(QuotedOperation => [Principal]);
worth_query_declaration::worth_query_operation_writes!(QuotedOperation => [PrincipalIdentityField, PrincipalNote, AccountStatus, AccountNote]);
worth_query_declaration::worth_query_operation_links!(QuotedOperation => [AccountOwner]);

pub struct QuotedOutputs;
impl ApplicationMutationOutputContract<ReservationSchema> for QuotedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[CreatedPrincipal::DESCRIPTOR, PreservedAccount::DESCRIPTOR];
}
pub struct CreatedPrincipal;
impl WorthQueryApplicationOutputRole for CreatedPrincipal {
    type Schema = ReservationSchema;
    type Contract = QuotedOutputs;
    type Entity = Principal;
    type Action = WorthQueryCreateOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "created-principal";
}
pub struct PreservedAccount;
impl WorthQueryApplicationOutputRole for PreservedAccount {
    type Schema = ReservationSchema;
    type Contract = QuotedOutputs;
    type Entity = Account;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "preserved-account";
}

pub struct QuotedDecision {
    pub(super) demand: ApplicationCandidateRequirements,
    key: WorthQueryApplicationEntityKey<ReservationSchema, Principal>,
    link_key: String,
    account: WorthQueryInvariantMutationTarget<ReservationSchema, Account>,
    replacement: Option<String>,
}
worth_query_declaration::worth_query_mutation_binding!(
    pub QuotedBinding for QuotedInput, schema ReservationSchema,
    identity "worth.query.test.installed-candidate-quote.v1",
    input QuotedInputBinding, operation QuotedOperation, result QuotedInputBinding,
    idempotency String, identity "worth.query.test.installed-quote-key.v1",
    decision QuotedDecision, denial QuotedInputBinding,
    handler identity "worth.query.test.installed-quote-handler.v1",
    outputs QuotedOutputs,
    principal IdentityBinding, mapping ExternalMapping, principal_entity Principal,
        principal_identity u64, identity_binding U64ApplicationValueBinding,
    scope Account, AccountPolicy, AccountStatus, String, ReadWrite, NoApplicationUnit,
    field AccountStatus::reference(), value scope,
    candidates policy ApplicationCandidatePolicy::new(None, None)
);
fn scope(_: &QuotedInput) -> String {
    "open".to_owned()
}

pub(super) struct QuotedHandler;
impl QuotedHandler {
    fn prepare(
        input: &QuotedInput,
        reader: &mut DecisionReader<'_, '_, '_, ReservationSchema, QuotedBinding>,
    ) -> Result<QuotedDecision, HandlerExecutionDenial> {
        let mut key = String::with_capacity(64);
        key.push_str("quoted-principal");
        let key = WorthQueryApplicationEntityKey::new(key).unwrap();
        let mut link_key = String::with_capacity(80);
        link_key.push_str("quoted-owner");
        let account = reader.resolve_entity(AccountStatus::reference(), "open".to_owned())?;
        assert_eq!(
            reader.field(&account, AccountStatus::reference())?,
            Some("open".to_owned())
        );
        assert_eq!(reader.field(&account, AccountNote::reference())?, None);
        let account = reader.mutation_target(&account)?;
        let value = StringApplicationValueBinding::encode(&input.value).unwrap();
        let replacement = input
            .write_order
            .replaces_required()
            .then(|| format!("{}-replacement", input.value));
        let required_value_bytes =
            replacement
                .as_ref()
                .map_or(value.owned_allocation_capacity_bytes(), |replacement| {
                    value.owned_allocation_capacity_bytes().max(
                        StringApplicationValueBinding::encode(replacement)
                            .unwrap()
                            .owned_allocation_capacity_bytes(),
                    )
                });
        let optional_value_bytes = if input.write_order == QuotedWriteOrder::OptionalClearThenClear
        {
            0 // This actual sequence retains metadata but never a value payload.
        } else {
            value.owned_allocation_capacity_bytes()
        };
        let identity = U64ApplicationValueBinding::encode(&42).unwrap();
        // The value binding owns the encoded heap measurement, the SDK owns metadata.
        let components = (|| -> Result<[usize; 11], WorthQueryApplicationAttemptDenial> {
            Ok([
                reader.candidate_create_bytes(Principal::reference(), &key)?,
                reader.candidate_initial_field_bytes(
                    PrincipalIdentityField::reference(),
                    identity.owned_allocation_capacity_bytes(),
                )?,
                reader.candidate_initial_field_bytes(
                    PrincipalNote::reference(),
                    value.owned_allocation_capacity_bytes(),
                )?,
                reader.candidate_update_bytes(Account::reference()),
                reader.candidate_write_field_bytes(
                    AccountStatus::reference(),
                    required_value_bytes,
                )?,
                if input.write_order.has_optional() {
                    reader.candidate_write_field_bytes(
                        AccountNote::reference(),
                        optional_value_bytes,
                    )?
                } else {
                    0
                },
                if input.write_order.has_optional() {
                    reader.candidate_field_promotion_bytes(AccountStatus::reference())?
                } else {
                    0
                },
                reader.candidate_link_bytes(
                    AccountOwner::reference(),
                    &link_key,
                    Some(&key),
                    None,
                )?,
                reader.candidate_output_contract_bytes()?,
                reader.candidate_output_bytes::<CreatedPrincipal>(Some(&key))?,
                reader.candidate_output_bytes::<PreservedAccount>(None)?,
            ])
        })()
        .map_err(HandlerExecutionDenial::new)?;
        assert!(
            reader
                .candidate_write_field_bytes(
                    PrincipalNote::reference(),
                    value.owned_allocation_capacity_bytes()
                )
                .unwrap()
                > reader
                    .candidate_initial_field_bytes(
                        PrincipalNote::reference(),
                        value.owned_allocation_capacity_bytes()
                    )
                    .unwrap(),
            "an existing optional write additionally retains the original portable contract"
        );
        assert_eq!(
            reader
                .candidate_write_field_bytes(AccountStatus::reference(), usize::MAX)
                .unwrap_err()
                .kind(),
            WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded
        );
        let bytes = components
            .into_iter()
            .try_fold(0_usize, usize::checked_add)
            .ok_or_else(|| {
                HandlerExecutionDenial::new(WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::CandidateReservationExceeded,
                    "actual quote overflow",
                ))
            })?;
        let bytes = if input.underquote {
            bytes.checked_sub(1).unwrap()
        } else {
            bytes
        };
        Ok(QuotedDecision {
            demand: ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(
                    1,
                    0,
                    1,
                    0,
                    input.write_order.write_count(),
                    0,
                ),
                ApplicationCandidateResourceCeiling::representation_bytes(bytes),
            ),
            key,
            link_key,
            account,
            replacement,
        })
    }

    fn write(
        input: &QuotedInput,
        decision: QuotedDecision,
        writer: &mut CandidateWriter<'_, ReservationSchema, QuotedBinding>,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let account = writer.projected_entity(&decision.account)?;
        let principal = writer.create_entity(Principal::reference(), decision.key)?;
        writer.initialize_field(&principal, PrincipalIdentityField::reference(), 42)?;
        writer.initialize_field(&principal, PrincipalNote::reference(), input.value.clone())?;
        if input.write_order.clears_optional() {
            writer.write_optional_field(&account, AccountNote::reference(), None)?;
            let next = if input.write_order == QuotedWriteOrder::OptionalClearThenSet {
                Some(input.value.clone())
            } else {
                None
            };
            writer.write_optional_field(&account, AccountNote::reference(), next)?;
            writer.write_field(&account, AccountStatus::reference(), input.value.clone())?;
        } else if input.write_order.replaces_required() {
            writer.write_optional_field(
                &account,
                AccountNote::reference(),
                Some(input.value.clone()),
            )?;
            writer.write_field(&account, AccountStatus::reference(), input.value.clone())?;
            writer.write_field(
                &account,
                AccountStatus::reference(),
                decision
                    .replacement
                    .expect("prepared replacement for this actual write order"),
            )?;
        } else {
            writer.write_field(&account, AccountStatus::reference(), input.value.clone())?;
            if input.write_order.has_optional() {
                writer.write_optional_field(
                    &account,
                    AccountNote::reference(),
                    Some(input.value.clone()),
                )?;
            }
        }
        writer.link(
            AccountOwner::reference(),
            decision.link_key,
            &principal,
            &account,
        )?;
        writer.create_output::<CreatedPrincipal>(&principal)?;
        writer.preserve_output::<PreservedAccount>(&account)
    }
}
impl OperationHandler<ReservationSchema, QuotedBinding> for QuotedHandler {
    fn decide(
        &self,
        input: &QuotedInput,
        reader: &mut DecisionReader<'_, '_, '_, ReservationSchema, QuotedBinding>,
    ) -> HandlerResult<QuotedDecision, QuotedInput> {
        match Self::prepare(input, reader) {
            Ok(decision) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn candidate_requirements(
        &self,
        _: &QuotedInput,
        decision: &QuotedDecision,
    ) -> ApplicationCandidateRequirements {
        decision.demand
    }
    fn build_candidate(
        &self,
        input: &QuotedInput,
        decision: QuotedDecision,
        writer: &mut CandidateWriter<'_, ReservationSchema, QuotedBinding>,
    ) -> HandlerResult<QuotedInput, QuotedInput> {
        match Self::write(input, decision, writer) {
            Ok(()) => HandlerResult::Completed(input.clone()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
