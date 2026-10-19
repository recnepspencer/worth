//! A real installed mutation with a required preserved root and created family.
use super::*;
use crate::domain_computation::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use worth_query_declaration::facade::{application_operation::*, application_schema::*};

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize)]
pub enum MemberPlan {
    Create,
    Preserve,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct PreparedInput {
    pub key: String,
    pub values: Vec<String>,
    pub declared: usize,
    pub plan: MemberPlan,
}
worth_query_declaration::worth_query_portable_type!(PreparedInput => "test.prepared-candidate-input.v1");
worth_query_declaration::worth_query_structured_value_binding!(pub PreparedInputBinding for PreparedInput { identity: "test.prepared-candidate-input.v1" });
worth_query_declaration::worth_query_operation!(pub PreparedOperation for ReservationSchema, input PreparedInputBinding);
worth_query_declaration::worth_query_operation_requires!(PreparedOperation => [ViewAccount]);
worth_query_declaration::worth_query_operation_reads!(PreparedOperation => [Account, AccountStatus]);
worth_query_declaration::worth_query_operation_creates!(PreparedOperation => [Account]);
worth_query_declaration::worth_query_operation_writes!(PreparedOperation => [AccountStatus]);

pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) struct PreparedOutputs;
pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) struct PreparedRoot;
impl WorthQueryApplicationOutputRole for PreparedRoot {
    type Schema = ReservationSchema;
    type Contract = PreparedOutputs;
    type Entity = Account;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "root";
}
pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) struct PreparedMembers;
impl WorthQueryApplicationOutputRoleFamily for PreparedMembers {
    type Schema = ReservationSchema;
    type Contract = PreparedOutputs;
    type Entity = Account;
    const PREFIX: &'static str = "member.";
    const POSTURES: ApplicationMutationOutputPostureSet =
        ApplicationMutationOutputPostureSet::from_bits(3).expect("create and preserve");
    const MINIMUM: usize = 1;
}
impl ApplicationMutationOutputContract<ReservationSchema> for PreparedOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] = &[PreparedRoot::DESCRIPTOR];
    const ROLE_FAMILIES: &'static [ApplicationMutationOutputRoleFamilyDescriptor] =
        &[PreparedMembers::DESCRIPTOR];
}
pub(in crate::domain_computation::primary_graph::application_attempt::effect_program) struct PreparedBinding<
    const FIXED: bool,
>;
impl<const FIXED: bool> ApplicationMutationBinding<ReservationSchema> for PreparedBinding<FIXED> {
    type Input = PreparedInput;
    type InputBinding = PreparedInputBinding;
    type Result = PreparedInput;
    type ResultBinding = PreparedInputBinding;
    type IdempotencyKey = String;
    type Operation = PreparedOperation;
    type Decision = PreparedInput;
    type Denial = PreparedInput;
    type DenialBinding = PreparedInputBinding;
    type Output = PreparedOutputs;
    type ScopeBinding = ApplicationMutationFieldScope<
        ReservationSchema,
        Account,
        AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        NoApplicationUnit,
    >;
    type SourceExpectation = NoApplicationMutationSource;
    type PrincipalBinding = IdentityBinding;
    type Mapping = ExternalMapping;
    type Principal = Principal;
    type PrincipalIdentity = u64;
    type PrincipalIdentityBinding = U64ApplicationValueBinding;
    const IDENTITY: &'static str = if FIXED {
        "test.fixed-candidate.v1"
    } else {
        "test.prepared-candidate.v1"
    };
    const HANDLER_IDENTITY: &'static str = Self::IDENTITY;
    const IDEMPOTENCY_IDENTITY: &'static str = Self::IDENTITY;
    const CANDIDATES: ApplicationCandidateContract = if FIXED {
        ApplicationCandidateContract::fixed_cardinality(
            ApplicationCandidateCardinalityCeiling::fixed(1, 0, 0, 0, 1, 0),
        )
    } else {
        ApplicationCandidateContract::prepared()
    };
    fn workflow_execution_requirements() -> ApplicationWorkflowExecutionRequirements {
        ApplicationWorkflowExecutionRequirements::not_applicable()
    }
    fn scope_field() -> ApplicationFieldRef<
        ReservationSchema,
        Account,
        AccountPolicy,
        AccountStatus,
        String,
        ReadWrite,
        EqualityPredicate,
        NoApplicationUnit,
    > {
        AccountStatus::reference()
    }
    fn principal_binding() -> ApplicationPrincipalBindingRef<
        ReservationSchema,
        IdentityBinding,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        IdentityBinding::reference()
    }
}
impl<const FIXED: bool> ApplicationMutationAdmissionBinding<ReservationSchema>
    for PreparedBinding<FIXED>
{
    type Admission = ApplicationConventionalAdmission;
}

pub(super) fn declare(
    schema: ApplicationSchemaDeclarationBuilder<ReservationSchema>,
) -> ApplicationSchemaDeclarationBuilder<ReservationSchema> {
    let operation = PreparedOperation::reference();
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(operation, 32)
        .operation_read_entity(operation, Account::reference())
        .operation_read_field(operation, AccountStatus::reference())
        .operation_requires_ability(operation, ViewAccount::reference())
        .operation_create(operation, Account::reference())
        .operation_write(operation, AccountStatus::reference())
        .application_mutation_binding::<PreparedBinding<false>>()
        .application_mutation_binding::<PreparedBinding<true>>()
}
pub(super) fn install(
    schema: &worth_query_installation::facade::WorthQueryInstalledApplicationSchema<
        ReservationSchema,
    >,
    bootstrap: &mut crate::domain_computation::primary_graph::WorthQueryPrimaryGraphBootstrap<
        ReservationSchema,
    >,
    authored: &Arc<AtomicUsize>,
) {
    bootstrap
        .install_handler(
            &schema
                .installed_mutation_binding::<PreparedBinding<false>>()
                .unwrap(),
            PreparedHandler(authored.clone()),
        )
        .unwrap();
    bootstrap
        .install_handler(
            &schema
                .installed_mutation_binding::<PreparedBinding<true>>()
                .unwrap(),
            PreparedHandler(authored.clone()),
        )
        .unwrap();
}
struct PreparedHandler(Arc<AtomicUsize>);
impl<const FIXED: bool> OperationHandler<ReservationSchema, PreparedBinding<FIXED>>
    for PreparedHandler
{
    fn decide(
        &self,
        input: &PreparedInput,
        reader: &mut DecisionReader<'_, '_, '_, ReservationSchema, PreparedBinding<FIXED>>,
    ) -> HandlerResult<PreparedInput, PreparedInput> {
        match reader.resolve_entity(AccountStatus::reference(), "open".to_owned()) {
            Ok(_) => HandlerResult::Completed(input.clone()),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn candidate_requirements(
        &self,
        _: &PreparedInput,
        decision: &PreparedInput,
    ) -> ApplicationCandidateRequirements {
        // The writer's encoded String copies have the source length; names,
        // locator and role strings are bounded separately for this fixed schema.
        let fresh = if decision.plan == MemberPlan::Create {
            decision.declared
        } else {
            0
        };
        let bytes = if decision.plan == MemberPlan::Preserve {
            decision
                .values
                .iter()
                .map(|suffix| "member.Account".len() + suffix.len())
                .sum::<usize>()
                + "rootAccountrootmember.".len()
        } else {
            decision
                .values
                .iter()
                .enumerate()
                .map(|(i, value)| {
                    let key = format!("{}-{i}", decision.key);
                    3 * key.len()
                        + value.len()
                        + "AccountAccountPolicyAccountStatusmember.".len()
                        + 128
                })
                .sum::<usize>()
                + "rootAccount".len()
        };
        ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(fresh, 0, 0, 0, fresh, 0),
            ApplicationCandidateResourceCeiling::representation_bytes(bytes),
        )
    }
    fn build_candidate(
        &self,
        _: &PreparedInput,
        decision: PreparedInput,
        writer: &mut CandidateWriter<'_, ReservationSchema, PreparedBinding<FIXED>>,
    ) -> HandlerResult<PreparedInput, PreparedInput> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let authored = (|| {
            let root = writer.resolve_entity(AccountStatus::reference(), "open".to_owned())?;
            writer.preserve_output::<PreparedRoot>(&root)?;
            for (i, value) in decision.values.iter().enumerate() {
                if decision.plan == MemberPlan::Preserve {
                    writer.preserve_member::<PreparedMembers>(value, &root)?;
                    continue;
                }
                let key = format!("{}-{i}", decision.key);
                let entity = writer.create_entity(
                    Account::reference(),
                    WorthQueryApplicationEntityKey::new(key.clone()).unwrap(),
                )?;
                writer.initialize_field(&entity, AccountStatus::reference(), value.clone())?;
                writer.create_member::<PreparedMembers>(&i.to_string(), &entity)?;
            }
            Ok::<_, crate::domain_computation::primary_graph::WorthQueryApplicationAttemptDenial>(())
        })();
        match authored {
            Ok(()) => HandlerResult::Completed(decision),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
