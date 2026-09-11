use std::time::{Duration, UNIX_EPOCH};

use worth_foundational::facade::AspectValue;

use super::super::application_attempt::{authenticated_principal, idempotency, resolved_account};
use super::super::fixture::{
    installed_authorization_world, installed_capability_authorization_world, live_scope, Account,
    AccountLabel, CapabilityAction, CapabilityGovernedInputIdentity, CapabilityPurpose,
    CapabilityTouchInput, CapabilityTouchOperation, IdentityExecutionSchema,
    TouchAccountCapability,
};
use crate::domain_computation::primary_graph::{
    WorthQueryAdmittedApplicationCapabilityAccess, WorthQueryAdmittedApplicationOperation,
    WorthQueryApplicationAuthorizationExplanationCause, WorthQueryApplicationCommitOutcome,
    WorthQueryApplicationEffectProgram, WorthQueryAuthenticatedPrincipal,
    WorthQueryOperationAuthorizationDenial, WorthQueryOperationAuthorizationDenialKind,
};

type World = super::super::fixture::AuthorizationWorld;
type Principal = super::super::fixture::Principal;
pub(super) type Program = WorthQueryApplicationEffectProgram<
    IdentityExecutionSchema,
    CapabilityTouchOperation,
    CapabilityTouchInput,
    Account,
>;
pub(super) type Admission = WorthQueryAdmittedApplicationOperation<
    IdentityExecutionSchema,
    CapabilityTouchOperation,
    CapabilityTouchInput,
    Account,
>;
type CapabilityAccess = WorthQueryAdmittedApplicationCapabilityAccess<
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
    let principal = authenticated_principal(&world, &request);
    let (program, evidence) = admitted_capability_program(&world, &principal, &request, "updated");

    assert_eq!(evidence.time_sample, AspectValue::UInt64(100));
    assert_eq!(evidence.authorization_fact_count, 2);
    assert_eq!(evidence.canonical_basis_preparations, 0);
    assert_eq!(evidence.canonical_digest_derivations, 0);
    assert_eq!(evidence.canonical_encoded_bytes, 0);
    assert!(evidence.requires_capability);
    assert_eq!(evidence.ability_count, 0);

    let outcome = world
        .application
        .compare_and_commit_application(program, idempotency(41, 41));
    assert!(
        matches!(outcome, WorthQueryApplicationCommitOutcome::Committed(_)),
        "current exact capability authority must commit: {outcome:?}"
    );
}

#[test]
fn caller_time_cannot_substitute_for_the_query_owned_sample() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(300)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let Err(denial) = admitted_capability_access(&world, &principal, &request, 100) else {
        panic!("a descriptive in-range caller time cannot revive an expired grant");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing
    );
    assert!(denial.identity().is_some());
    assert_eq!(
        denial.causes(),
        [WorthQueryOperationAuthorizationDenialKind::CapabilityAuthorizationMissing]
    );
}

#[test]
fn explicit_purpose_mismatch_preserves_its_exact_explanation_cause() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    let mut input = capability_input(100, CapabilityGovernedInputIdentity::None);
    input.purpose = CapabilityPurpose::Audit;

    let Err(denial) =
        world
            .selected_product()
            .admit_capability_access(&principal, &capability, input, &request)
    else {
        panic!("an explicit purpose mismatch must not mint capability authority")
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::PurposeMismatch
    );
    assert_eq!(
        denial.explanation_cause(),
        Some(WorthQueryApplicationAuthorizationExplanationCause::PurposeMismatch)
    );
}

#[test]
fn capability_governed_operation_rejects_conventional_authorization() {
    let world = installed_capability_authorization_world();
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);
    let account = resolved_account(&world, "open", &request);
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();

    assert!(operation.contracts().authorization().requires_capability());
    assert!(operation.contracts().ability_requirements().is_empty());
    let Err(denial) = world.selected_product().authorize_operation(
        &principal,
        &account,
        &operation,
        Default::default(),
        &request,
    ) else {
        panic!("principal authority cannot substitute for capability access");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityRequired
    );
}

#[test]
fn admitted_access_is_runtime_affine_at_operation_progression() {
    let source = installed_capability_authorization_world();
    source.authorization_time.script([time(100)]);
    let target = installed_capability_authorization_world();
    let request = live_scope();
    let principal = authenticated_principal(&source, &request);
    let access = admitted_capability_access(&source, &principal, &request, 100).unwrap();
    let target_operation = target
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();

    let Err(denial) = target.application.authorize_capability_operation(
        access,
        &target_operation,
        Default::default(),
    ) else {
        panic!("an access proof copied across runtimes cannot authorize an operation");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::ForeignRuntime
    );
}

#[test]
fn missing_grant_membership_mints_no_access_authority() {
    let world = installed_authorization_world(false);
    world.authorization_time.script([time(100)]);
    let request = live_scope();
    let principal = authenticated_principal(&world, &request);

    let Err(denial) = admitted_capability_access(&world, &principal, &request, 100) else {
        panic!("a principal without grant membership cannot receive capability access");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing
    );
    assert!(denial.identity().is_some());
    assert_eq!(
        denial.causes(),
        [WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing]
    );
}

pub(super) struct AdmissionEvidence {
    time_sample: AspectValue,
    authorization_fact_count: usize,
    canonical_basis_preparations: u32,
    canonical_digest_derivations: u32,
    canonical_encoded_bytes: usize,
    requires_capability: bool,
    ability_count: usize,
    pub(super) canonical_work: worth_query_installation::facade::WorthQueryCanonicalWorkEvidence,
}

pub(super) fn admitted_capability_program(
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
) -> (Program, AdmissionEvidence) {
    admitted_capability_program_with_governed_input(
        world,
        principal,
        request,
        replacement,
        CapabilityGovernedInputIdentity::None,
    )
}

pub(super) fn admitted_capability_program_with_governed_input(
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    replacement: &str,
    governed_input_identity: CapabilityGovernedInputIdentity,
) -> (Program, AdmissionEvidence) {
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();
    let access = admitted_capability_access_with_governed_input(
        world,
        principal,
        request,
        100,
        governed_input_identity,
    )
    .unwrap();
    let evidence = capture_admission_evidence(
        &access,
        operation.contracts().authorization().requires_capability(),
        operation.contracts().ability_requirements().len(),
    );
    let admission = world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .unwrap();
    (
        build_touch_program(world, request, admission, replacement),
        evidence,
    )
}

fn capture_admission_evidence(
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

fn build_touch_program(
    world: &World,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    admission: Admission,
    replacement: &str,
) -> Program {
    let account = resolved_account(world, "open", request);
    let (_, projection, _) = world
        .invariant
        .project_admitted_operation(&admission, |reader, projected| {
            reader
                .require_decision_field(projected, AccountLabel::reference())
                .unwrap();
        })
        .unwrap()
        .into_parts();
    let reads = world
        .application
        .begin_projected_application_read_attempt(admission, projection)
        .unwrap();
    let mut effects = reads
        .complete_projected_dependencies()
        .unwrap()
        .begin_effect_program();
    let account = effects.existing_entity(&account).unwrap();
    effects
        .write_field(&account, AccountLabel::reference(), replacement.to_owned())
        .unwrap();
    effects.finish().unwrap()
}

pub(super) fn admitted_capability_access(
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    caller_time: u64,
) -> Result<CapabilityAccess, WorthQueryOperationAuthorizationDenial> {
    admitted_capability_access_with_governed_input(
        world,
        principal,
        request,
        caller_time,
        CapabilityGovernedInputIdentity::None,
    )
}

pub(super) fn admitted_capability_access_with_governed_input(
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    caller_time: u64,
    governed_input_identity: CapabilityGovernedInputIdentity,
) -> Result<CapabilityAccess, WorthQueryOperationAuthorizationDenial> {
    let capability = world
        .application
        .installed_schema()
        .capability(
            TouchAccountCapability::reference(),
            CapabilityTouchOperation::reference(),
        )
        .unwrap();
    world.selected_product().admit_capability_access(
        principal,
        &capability,
        capability_input(caller_time, governed_input_identity),
        request,
    )
}

pub(super) fn capability_input(
    caller_time: u64,
    governed_input_identity: CapabilityGovernedInputIdentity,
) -> CapabilityTouchInput {
    CapabilityTouchInput {
        account: "account-1".to_owned(),
        action: CapabilityAction::Touch,
        purpose: CapabilityPurpose::AccountMaintenance,
        disclosure: super::super::fixture::CapabilityDisclosure::AccountActivity,
        related_account: "account-2".to_owned(),
        amount: 50,
        caller_time,
        request_record: "selected-request".to_owned(),
        prior_record: "selected-prior".to_owned(),
        governed_input_identity,
    }
}

pub(super) fn admitted_capability_operation(
    world: &World,
    principal: &WorthQueryAuthenticatedPrincipal<IdentityExecutionSchema, Principal, u64>,
    request: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
) -> Admission {
    let access = admitted_capability_access(world, principal, request, 100).unwrap();
    let operation = world
        .application
        .installed_schema()
        .installed_operation(CapabilityTouchOperation::reference())
        .unwrap();
    world
        .application
        .authorize_capability_operation(access, &operation, Default::default())
        .unwrap()
}

pub(super) fn time(seconds: u64) -> std::time::SystemTime {
    UNIX_EPOCH + Duration::from_secs(seconds)
}
