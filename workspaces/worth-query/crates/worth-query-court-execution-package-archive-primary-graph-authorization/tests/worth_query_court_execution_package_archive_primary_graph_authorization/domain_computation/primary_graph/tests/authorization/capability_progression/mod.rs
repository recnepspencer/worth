pub use std::time::Duration;
pub use std::time::UNIX_EPOCH;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationRefusalKind as AuthorizationRefusal;

pub use worth_foundational::facade::AspectValue;

pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability_world_installation::installed_capability_authorization_world;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Account;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityAction;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityPurpose;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityTouchInput;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::CapabilityTouchOperation;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::IdentityExecutionSchema;
pub use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::TouchAccountCapability;
pub use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationCapabilityAccess;
pub use worth_query_execution::facade::domain_computation::WorthQueryAdmittedApplicationOperation;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationAuthorizationExplanationCause;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
pub use worth_query_execution::facade::domain_computation::WorthQueryApplicationEffectProgram;
pub use worth_query_execution::facade::domain_computation::WorthQueryAuthenticatedPrincipal;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenial;
pub use worth_query_execution::facade::domain_computation::WorthQueryOperationAuthorizationDenialKind;

pub type World = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::authorization_world_installation::AuthorizationWorld;
pub type Principal = worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::Principal;
pub type Program = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    CapabilityTouchOperation,
    CapabilityTouchInput,
    Account,
>;
pub type Admission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    CapabilityTouchOperation,
    CapabilityTouchInput,
    Account,
>;
pub type CapabilityAccess = WorthQueryAdmittedApplicationCapabilityAccess<
    IdentityExecutionSchema,
    TouchAccountCapability,
    CapabilityTouchOperation,
    CapabilityTouchInput,
>;

#[test]
fn current_capability_progresses_through_the_real_application_commit() {
    let world = installed_capability_authorization_world();
    world.authorization_time.hold(time(100));
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");
    let (program, evidence) = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_program(&phase, &world, &principal, &request, "updated")
        })
        .expect("the fixture read owns its installed host call");

    assert_eq!(evidence.time_sample, AspectValue::UInt64(100));
    assert_eq!(evidence.authorization_fact_count, 2);
    assert_eq!(evidence.canonical_basis_preparations, 0);
    // The one canonical derivation is the governed input the admission encoded.
    assert_eq!(evidence.canonical_digest_derivations, 1);
    assert_eq!(evidence.canonical_encoded_bytes, 289);
    assert!(evidence.requires_capability);
    assert_eq!(evidence.ability_count, 0);

    let outcome = world.application.compare_and_commit_application(
        program,
        idempotency(41, 41),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    );
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "current exact capability authority must commit: {outcome:?}"
    );
}

#[test]
fn admitted_access_is_runtime_affine_at_operation_progression() {
    let source = installed_capability_authorization_world();
    source.authorization_time.script([time(100)]);
    let target = installed_capability_authorization_world();
    let request = live_scope();
    let access = source
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let principal = authenticated_principal(&host_custody, &source, &request);

            admitted_capability_access(&host_custody, &source, &principal, &request, 100).unwrap()
        })
        .expect("the source access read owns its host call");
    target
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            let target_operation = target
                .application
                .installed_schema()
                .installed_operation(CapabilityTouchOperation::reference())
                .unwrap();

            let Err(denial) = target.application.authorize_capability_operation(
                &host_custody.phase_for_application(&target.application),
                access,
                &target_operation,
                Default::default(),
            ) else {
                panic!("an access proof copied across runtimes cannot authorize an operation");
            };
            assert_eq!(
                denial.kind(),
                WorthQueryOperationAuthorizationDenialKind::Refused(
                    AuthorizationRefusal::ForeignRuntime
                )
            );
        })
        .expect("the fixture block owns its installed host call")
}

#[test]
fn missing_grant_membership_mints_no_access_authority() {
    let world = installed_authorization_world(false);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&(phase).test_host_request(), &world, &request)
        })
        .expect("the fixture read owns its installed host call");

    let Err(denial) = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_access(
                &(phase).test_host_request(),
                &world,
                &principal,
                &request,
                100,
            )
        })
        .expect("the fixture read owns its installed host call")
    else {
        panic!("a principal without grant membership cannot receive capability access");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing
        )
    );
    assert!(denial.identity().is_some());
    assert_eq!(
        denial.causes(),
        [WorthQueryOperationAuthorizationDenialKind::Refused(
            AuthorizationRefusal::CapabilityGrantMissing
        )]
    );
}

pub struct AdmissionEvidence {
    pub time_sample: AspectValue,
    pub authorization_fact_count: usize,
    pub canonical_basis_preparations: u32,
    pub canonical_digest_derivations: u32,
    pub canonical_encoded_bytes: usize,
    pub requires_capability: bool,
    pub ability_count: usize,
    pub canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

pub fn admitted_capability_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
) -> (Program, AdmissionEvidence) {
    admitted_capability_program_for_input(
        phase_custody,
        world,
        principal,
        request,
        replacement,
        capability_input(100),
    )
}

pub fn admitted_capability_program_for_input(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
    input: CapabilityTouchInput,
) -> (Program, AdmissionEvidence) {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();
    let access = admitted_capability_access_for_input(
        &(phase_custody).test_host_request(),
        world,
        principal,
        request,
        input,
    )
    .unwrap();
    let evidence = capture_admission_evidence(
        &access,
        operation.contracts().authorization().requires_capability(),
        operation.contracts().ability_requirements().len(),
    );
    let admission = world
        .application
        .authorize_capability_operation(phase_custody, access, &operation, Default::default())
        .unwrap();
    (
        build_touch_program(phase_custody, world, request, admission, replacement),
        evidence,
    )
}

pub fn capture_admission_evidence(
    access: &CapabilityAccess,
    requires_capability: bool,
    ability_count: usize,
) -> AdmissionEvidence {
    let work = access.admission_canonical_work();
    AdmissionEvidence {
        time_sample: access.capability_time_sample().clone(),
        authorization_fact_count: access.authorization_decision_fact_count(),
        canonical_basis_preparations: work.basis_preparations(),
        canonical_digest_derivations: work.digest_derivations(),
        canonical_encoded_bytes: work.canonical_encoded_bytes(),
        requires_capability,
        ability_count,
        canonical_work: work,
    }
}

pub fn build_touch_program(
    phase_custody: &worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase<'_>,
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    admission: Admission,
    replacement: &str,
) -> Program {
    let account = resolved_account(&(phase_custody).test_host_request(), world, "open", request);
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(
            phase_custody,
            &admission,
            |reader, projected| {
                reader
                    .require_decision_field(projected, AccountLabel::reference())
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
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&account, AccountLabel::reference(), replacement.to_owned())
        .unwrap();
    effects.finish().unwrap()
}

pub fn admitted_capability_access(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    caller_time: u64,
) -> Result<CapabilityAccess, WorthQueryOperationAuthorizationDenial> {
    admitted_capability_access_for_input(
        phase_custody,
        world,
        principal,
        request,
        capability_input(caller_time),
    )
}

pub fn admitted_capability_access_for_input(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    input: CapabilityTouchInput,
) -> Result<CapabilityAccess, WorthQueryOperationAuthorizationDenial> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    world
        .selected_product(&(phase_custody).test_host_request())
        .admit_capability_access(
            &phase_custody.phase_for_application(&world.application),
            principal,
            &capability,
            input,
            request,
        )
}

pub fn capability_input(caller_time: u64) -> CapabilityTouchInput {
    CapabilityTouchInput {
        account: "account-1".to_owned(),
        action: CapabilityAction::Touch,
        purpose: CapabilityPurpose::AccountMaintenance,
        disclosure: worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::capability::declaration::CapabilityDisclosure::AccountActivity,
        related_account: "account-2".to_owned(),
        amount: 50,
        caller_time,
        request_record: "selected-request".to_owned(),
        prior_record: "selected-prior".to_owned(),
    }
}

pub fn admitted_capability_operation(
    phase_custody: &worth_query_execution::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Admission {
    let access = admitted_capability_access(phase_custody, world, principal, request, 100).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();
    world
        .application
        .authorize_capability_operation(
            &phase_custody.phase_for_application(&world.application),
            access,
            &operation,
            Default::default(),
        )
        .unwrap()
}

pub fn time(seconds: u64) -> std::time::SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}

#[path = "caller_time_cannot_substitute_for_the_query_owned_sample/mod.rs"]
pub mod caller_time_cannot_substitute_for_the_query_owned_sample;
#[path = "capability_governed_operation_rejects_conventional_authorization/mod.rs"]
pub mod capability_governed_operation_rejects_conventional_authorization;
#[path = "explicit_purpose_mismatch_preserves_its_exact_explanation_cause/mod.rs"]
pub mod explicit_purpose_mismatch_preserves_its_exact_explanation_cause;
