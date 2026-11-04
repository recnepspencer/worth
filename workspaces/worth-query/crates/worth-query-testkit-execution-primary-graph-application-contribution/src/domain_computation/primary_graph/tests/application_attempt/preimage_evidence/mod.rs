//! Exact reusable commit evidence and demanded-work breadth proofs.

use worth_query_declaration::facade::application_operation::ApplicationMutationOutputContract;
use worth_query_declaration::facade::application_operation::ApplicationMutationOutputRoleDescriptor;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationDeclaredOutputRole;
use worth_query_declaration::facade::application_operation::WorthQueryApplicationOutputRole;
use worth_query_declaration::facade::application_operation::WorthQueryExactlyOneOutput;
use worth_query_declaration::facade::application_operation::WorthQueryPreserveOutput;
use worth_query_declaration::facade::application_schema::TypedMutationPreconditions;
use crate::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use crate::domain_computation::primary_graph::tests::application_attempt::idempotency;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use crate::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::output_correspondence::role_use::OutputRoleUse;
use crate::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use crate::domain_computation::primary_graph::tests::fixture::Account;
use crate::domain_computation::primary_graph::tests::fixture::AccountLabel;
use crate::domain_computation::primary_graph::tests::fixture::AccountStatus;
use crate::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::ExactStatusRetentionInput;
use crate::domain_computation::primary_graph::tests::fixture::ExactStatusRetentionOperation;
use crate::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
use crate::domain_computation::primary_graph::tests::fixture::Principal;
use crate::domain_computation::primary_graph::tests::fixture::RetainedStatusEffect;
use crate::domain_computation::primary_graph::tests::fixture::schema_types::RetainedStatusNotice;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationEntityIdentity;
use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
pub struct RetentionOutputs;
impl ApplicationMutationOutputContract<IdentityExecutionSchema> for RetentionOutputs {
    const ROLES: &'static [ApplicationMutationOutputRoleDescriptor] =
        &[<RetainedAccount as WorthQueryApplicationDeclaredOutputRole>::DESCRIPTOR];
}
pub struct RetainedAccount;
impl WorthQueryApplicationOutputRole for RetainedAccount {
    type Schema = IdentityExecutionSchema;
    type Contract = RetentionOutputs;
    type Entity = Account;
    type Action = WorthQueryPreserveOutput;
    type Cardinality = WorthQueryExactlyOneOutput;
    const NAME: &'static str = "retained-account";
}
pub fn demanded_mutation_work(
    breadth: RetentionMutationBreadth,
    key: u8,
) -> worth_query_execution::facade::domain_computation::WorthQueryPrimaryMutationWorkEvidence {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase.test_host_request_for_test();

            authenticated_principal(
                &(phase_custody).test_host_request_for_test(),
                &world,
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let account = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase.test_host_request_for_test();

            resolved_account(
                &(phase_custody).test_host_request_for_test(),
                &world,
                "open",
                &request,
            )
        })
        .expect("the fixture read owns its installed host call");
    let program = world
        .application
        .with_host_advancement(|phase| {
            let phase_custody = &phase.test_host_request_for_test();

            retained_status_program(
                phase_custody,
                &world,
                &principal,
                &account,
                &request,
                "frozen",
                breadth,
            )
        })
        .expect("the fixture read owns its installed host call");
    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(key, key),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    let WorthQueryApplicationCommitOutcome::Committed(receipt) = outcome else {
        panic!("demanded mutation commits: {outcome:?}");
    };
    receipt.mutation_work().unwrap().clone()
}
pub fn assert_retained_status(
    receipt: &worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitReceipt,
    expected: &str,
) {
    let retained = receipt
        .retained_preimage_for_test()
        .expect("recorded inverse retains prior truth");
    assert!(receipt.has_retained_preimage());
    assert!(
        !format!("{retained:?}").contains(expected),
        "receipt debugging must not disclose the historical value"
    );
    assert_eq!(
        retained
            .field_for(AccountStatus::reference())
            .unwrap()
            .value(),
        &worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from(expected)
        )
    );
}
#[derive(Clone, Copy)]
pub enum RetentionMutationBreadth {
    Narrow,
    CrossRecordLabel,
}
pub fn retained_status_program(
    phase_custody: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &AuthorizationWorld,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    account: &WorthQueryApplicationEntityIdentity<IdentityExecutionSchema, Account>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
    breadth: RetentionMutationBreadth,
) -> WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    ExactStatusRetentionOperation,
    ExactStatusRetentionInput,
    Account,
> {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(ExactStatusRetentionOperation::reference())
        .unwrap();
    let admission = world
        .selected_product(&(phase_custody).test_host_request_for_test())
        .authorize_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            principal,
            account,
            &operation,
            TypedMutationPreconditions::new(),
            request,
        )
        .unwrap();
    let other = matches!(breadth, RetentionMutationBreadth::CrossRecordLabel).then(|| {
        resolved_account(
            &(phase_custody).test_host_request_for_test(),
            world,
            "unrelated",
            request,
        )
    });
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            &phase_custody.phase_for_application_for_test(&world.application),
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountStatus::reference())
                    .unwrap();
                match breadth {
                    RetentionMutationBreadth::Narrow => {}
                    RetentionMutationBreadth::CrossRecordLabel => {
                        let other = reader
                            .resolve_entity(AccountStatus::reference(), "unrelated".to_owned())
                            .unwrap();
                        reader
                            .require_decision_field(&other, AccountLabel::reference())
                            .unwrap();
                    }
                }
            },
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(
            &phase_custody.phase_for_application_for_test(&world.application),
            admission,
            projection,
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies(
            &phase_custody.phase_for_application_for_test(&world.application),
            worth_execution::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap()
        .begin_effect_program();
    world.declare_output_contract::<RetentionOutputs>("RetentionOutputs");
    effects.prepare_output_contract_for_test::<RetentionOutputs>();
    let account = effects.existing_entity(account).unwrap();
    effects
        .bind_output_for_test(OutputRoleUse::fixed_for_test::<RetainedAccount>(), &account)
        .expect("retained account output belongs to this effect program");
    effects
        .write_field(&account, AccountStatus::reference(), replacement.to_owned())
        .unwrap();
    if let Some(other) = other {
        let other = effects.existing_entity(&other).unwrap();
        effects
            .write_field(
                &other,
                AccountLabel::reference(),
                "retention-breadth".to_owned(),
            )
            .unwrap();
    }
    effects
        .emit_external(
            RetainedStatusEffect::reference(),
            RetainedStatusNotice(format!("status-retention:{replacement}")),
        )
        .unwrap();
    effects.finish().unwrap()
}
