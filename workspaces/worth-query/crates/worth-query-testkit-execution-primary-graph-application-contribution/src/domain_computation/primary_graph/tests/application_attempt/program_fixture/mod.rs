use worth_query_declaration::facade::application_schema::ApplicationEncodedScalarValue;
use worth_query_declaration::facade::application_schema::StringApplicationValueBinding;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;

use crate::domain_computation::primary_graph::tests::application_attempt::AccountStatus;
use crate::domain_computation::primary_graph::tests::application_attempt::TouchAccountOperation;
use crate::domain_computation::primary_graph::tests::fixture::program_required_binding::ProgramRequiredInput;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::MutationFreeEmitInput;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::MutationFreeNotice;
use crate::domain_computation::primary_graph::tests::fixture::MutationFreeEmitOperation;
use crate::domain_computation::primary_graph::tests::fixture::MutationFreeExternalEffect;
use crate::domain_computation::primary_graph::tests::fixture::ProgramRequiredOperation;
use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
use worth_query_execution::facade::domain_computation::WorthQuerySelectedProductOperation;

pub type Schema = crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub type Principal = crate::domain_computation::primary_graph::tests::fixture::Principal;
pub type Account = crate::domain_computation::primary_graph::tests::fixture::Account;
pub type Input =
    crate::domain_computation::primary_graph::tests::fixture::schema_types::TouchAccountInput;
pub type World = crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub type Preconditions = TypedMutationPreconditions<Schema, TouchAccountOperation, Account>;
pub type Program =
    WorthQueryApplicationEffectProgram<Schema, TouchAccountOperation, Input, Account>;

pub fn admitted_operation(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> WorthQueryAdmittedApplicationOperation<Schema, TouchAccountOperation, Input, Account> {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    selected
        .authorize_operation(
            phase_custody,
            principal,
            account,
            &operation,
            Preconditions::new(),
            request,
        )
        .unwrap()
}
pub type MutationFreeProgram = WorthQueryApplicationEffectProgram<
    Schema,
    MutationFreeEmitOperation,
    MutationFreeEmitInput,
    Account,
>;
pub type ProgramRequiredProgram = WorthQueryApplicationEffectProgram<
    Schema,
    ProgramRequiredOperation,
    ProgramRequiredInput,
    Account,
>;

/// An admitted effect program for the one operation the fixture schema marks
/// program-required, which is the operation the fixture roster acts through.
pub fn admitted_program_required_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
) -> ProgramRequiredProgram {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .authorize_operation(
            phase_custody,
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountStatus::at("AccountStatus"))
                    .unwrap();
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            phase_custody,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(account).unwrap();
    effects
        .write_field(
            &account,
            AccountStatus::at("AccountStatus"),
            replacement.to_owned(),
        )
        .unwrap();
    effects.finish().unwrap()
}

pub fn admitted_mutation_free_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> MutationFreeProgram {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(MutationFreeEmitOperation::reference())
        .unwrap();
    assert!(operation
        .contracts()
        .invariant_execution()
        .requirements()
        .is_empty());
    let admission = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .authorize_operation(
            phase_custody,
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |_, _| {},
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            phase_custody,
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            phase_custody,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    effects
        .emit_external(
            MutationFreeExternalEffect::reference(),
            MutationFreeNotice(13),
        )
        .unwrap();
    effects.finish().unwrap()
}

pub fn admitted_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
) -> Program {
    admitted_program_with_emit(
        phase_custody,
        world,
        principal,
        account,
        request,
        replacement,
        None,
    )
}

pub fn admitted_program_with_emit(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
    emission: Option<&str>,
) -> Program {
    admitted_program_with_emissions(
        phase_custody,
        world,
        principal,
        account,
        request,
        replacement,
        emission,
    )
}

pub fn admitted_program_with_emissions<'a>(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
    emissions: impl IntoIterator<Item = &'a str>,
) -> Program {
    admitted_program_from_options(
        phase_custody,
        world,
        principal,
        account,
        request,
        ProgramOptions {
            replacement,
            emissions: emissions.into_iter().collect(),
            preconditions: Preconditions::new(),
        },
    )
}

pub fn admitted_program_with_expected_status(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    status_and_replacement: (&str, &str),
) -> Program {
    admitted_program_from_options(
        phase_custody,
        world,
        principal,
        account,
        request,
        ProgramOptions {
            replacement: status_and_replacement.1,
            emissions: Vec::new(),
            preconditions: Preconditions::new()
                .expect_fact(
                    AccountStatus::at("AccountStatus"),
                    ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                        status_and_replacement.0.to_owned(),
                    )
                    .expect("fixture account status must encode"),
                )
                .expect("record field contract"),
        },
    )
}

pub fn admitted_program_on_selected(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    selected: &WorthQuerySelectedProductOperation<'_, Schema>,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
) -> Program {
    admitted_program_from_selected(
        phase_custody,
        world,
        selected,
        principal,
        account,
        request,
        ProgramOptions {
            replacement,
            emissions: Vec::new(),
            preconditions: Preconditions::new(),
        },
    )
}

pub struct ProgramOptions<'a> {
    pub replacement: &'a str,
    pub emissions: Vec<&'a str>,
    pub preconditions: Preconditions,
}

pub fn admitted_program_from_options(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<Schema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<Schema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    options: ProgramOptions<'_>,
) -> Program {
    let selected = world.selected_product(&(phase_custody).test_host_request_for_test());
    admitted_program_from_selected(
        phase_custody,
        world,
        &selected,
        principal,
        account,
        request,
        options,
    )
}
#[path = "selected_program/mod.rs"]
pub mod selected_program;
use selected_program::admitted_program_from_selected;
