use super::super::super::application_attempt::idempotency;
use super::super::super::fixture::{
    revoke_current_capability, CapabilityElevationScenario, CapabilityElevationStatus,
};
use super::super::capability_progression::time;
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryElevationApprovalOutcome, WorthQueryOperationAuthorizationDenialKind,
    WorthQueryRequestedElevation,
};

#[test]
fn exact_support_revocation_denies_approval_despite_an_equivalent_current_grant() {
    let (world, request, requested) = super::approval_transition::requested_world(
        CapabilityElevationScenario::AlternateCurrentGrant,
    );
    revoke_current_capability(&world);
    let approver = super::approval_transition::authenticated(&world, "bob", &request);
    let access = super::approval_transition::approval_access(&world, &approver, &request)
        .expect("the independent approval command grant remains current");

    let Err(denial) = world.application.authorize_elevation_approval(
        requested,
        access,
        &super::approval_transition::approval_operation(&world),
        Default::default(),
    ) else {
        panic!("an equivalent grant must not replace the request's exact support lineage");
    };

    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    assert!(denial.denial().identity().is_some());
    assert_eq!(
        denial.denial().causes(),
        [WorthQueryOperationAuthorizationDenialKind::StaleAuthorization]
    );
    assert_requested_without_approval(
        &world,
        &denial.into_requested(),
        approver.principal_entity_id(),
    );
}

#[test]
fn exact_support_revocation_after_materialization_denies_provider_commit() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let approver = super::approval_transition::authenticated(&world, "bob", &request);
    let program =
        super::approval_transition::materialize_exact_approval(&world, &request, requested);
    revoke_current_capability(&world);

    let WorthQueryElevationApprovalOutcome::Denied(denial, requested) = world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(175, 175))
    else {
        panic!("support loss after materialization must deny before approval effects");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::DecisionReadSet
    );
    assert_requested_without_approval(&world, &requested, approver.principal_entity_id());
}

#[test]
fn unrelated_published_lifecycle_drift_stales_the_selected_product_mutation() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let program =
        super::approval_transition::materialize_exact_approval(&world, &request, requested);
    super::mutation::set_status(&world, "elevation-1", CapabilityElevationStatus::Revoked);

    let outcome = world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(176, 176));
    let WorthQueryElevationApprovalOutcome::Denied(denial, _requested) = outcome else {
        panic!("the mutation bound to the older product must deny before effects: {outcome:?}");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProductBasisStale
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::InvariantExecution
    );
    assert_eq!(
        super::terminal_state::elevation_status(&world),
        CapabilityElevationStatus::Requested
    );
}

#[test]
fn support_expiry_denies_approval_while_the_request_lifecycle_is_current() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::ExpiringSupport);
    world
        .authorization_time
        .script(std::iter::repeat_n(time(103), 8));
    let approver = super::approval_transition::authenticated(&world, "bob", &request);
    let access = super::approval_transition::approval_access(&world, &approver, &request)
        .expect("the approval command and request lifecycle remain current at time 103");

    let Err(denial) = world.application.authorize_elevation_approval(
        requested,
        access,
        &super::approval_transition::approval_operation(&world),
        Default::default(),
    ) else {
        panic!("the exact support expired at time 102 and must be reobserved");
    };

    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing
    );
    assert!(denial.denial().identity().is_some());
    assert_eq!(
        denial.denial().causes(),
        [WorthQueryOperationAuthorizationDenialKind::CapabilityGrantMissing]
    );
    let requested = denial.into_requested();
    assert_eq!(
        requested.expires_at(),
        &worth_foundational::facade::AspectValue::UInt64(105)
    );
    assert_requested_without_approval(&world, &requested, approver.principal_entity_id());
}

#[test]
fn replacement_policy_path_for_the_same_grant_cannot_launder_request_support() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let requester = super::approval_transition::authenticated(&world, "alice", &request);
    super::mutation::replace_support_grantor_with_custodian(
        &world,
        requester.principal_entity_id(),
    );
    let approver = super::approval_transition::authenticated(&world, "bob", &request);
    let access = super::approval_transition::approval_access(&world, &approver, &request)
        .expect("the independent approval command grant remains current");

    let Err(denial) = world.application.authorize_elevation_approval(
        requested,
        access,
        &super::approval_transition::approval_operation(&world),
        Default::default(),
    ) else {
        panic!("a new allowing path must not replace the request receipt's original path");
    };

    assert_eq!(
        denial.denial().kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    assert_requested_without_approval(
        &world,
        &denial.into_requested(),
        approver.principal_entity_id(),
    );
}

#[test]
fn replacement_policy_path_for_the_same_grant_cuts_off_approved_use() {
    let (world, request, approved) = super::approval_transition::exact_approved_world();
    let requester = super::approval_transition::authenticated(&world, "alice", &request);
    super::mutation::replace_support_grantor_with_custodian(
        &world,
        requester.principal_entity_id(),
    );
    world.authorization_time.script([time(100)]);
    let capability = super::installed_capability(&world);

    let Err(denial) = world.selected_product().admit_approved_elevation_access(
        &approved,
        &requester,
        &capability,
        super::elevated_input(Some("elevation-2")),
        &request,
    ) else {
        panic!("a new allowing path must not replace approved support for active use");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
    );
    assert_eq!(
        super::terminal_state::elevation_status(&world),
        CapabilityElevationStatus::Approved
    );
}

#[test]
fn request_expiry_after_approval_materialization_denies_provider_commit() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let approver = super::approval_transition::authenticated(&world, "bob", &request);
    let program =
        super::approval_transition::materialize_exact_approval(&world, &request, requested);
    world
        .authorization_time
        .script(std::iter::repeat_n(time(106), 4));

    let WorthQueryElevationApprovalOutcome::Denied(denial, requested) = world
        .application
        .compare_and_commit_elevation_approval(program, idempotency(177, 177))
    else {
        panic!("a request that expired after materialization must not commit approval effects");
    };

    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::DecisionReadSet
    );
    assert_requested_without_approval(&world, &requested, approver.principal_entity_id());
}

#[test]
fn exact_approval_authorization_carries_three_graph_facts_without_canonical_admission_work() {
    let (world, request, requested) =
        super::approval_transition::requested_world(CapabilityElevationScenario::Active);
    let approver = super::approval_transition::authenticated(&world, "bob", &request);
    let access = super::approval_transition::approval_access(&world, &approver, &request).unwrap();
    let admission = world
        .application
        .authorize_elevation_approval(
            requested,
            access,
            &super::approval_transition::approval_operation(&world),
            Default::default(),
        )
        .unwrap();

    assert_eq!(admission.authorization_decision_fact_count(), 3);
    assert_eq!(admission.graph_work_decision_fact_count(), 3);
    let work = admission.canonical_work().admission();
    assert_eq!(work.basis_preparations(), 0);
    assert_eq!(work.digest_derivations(), 0);
    assert_eq!(work.digest_text_materializations(), 0);
}

#[test]
fn request_command_resource_can_differ_from_the_governed_resource_it_progresses() {
    let mut input = super::request_transition::honest_input();
    input.account = "account-2".to_owned();
    assert_eq!(input.target_account, "account-1");
    assert_eq!(input.grant, "capability-1");
    let (world, request, requested) = super::approval_transition::requested_world_with_input(
        CapabilityElevationScenario::DistinctCommandResource,
        input,
    );
    let approver = super::approval_transition::authenticated(&world, "bob", &request);

    let approved = super::approval_transition::approve_exact_request(&world, &request, requested);

    assert_eq!(approved.approver(), approver.principal_entity_id());
    assert_eq!(
        super::terminal_state::elevation_status(&world),
        CapabilityElevationStatus::Approved
    );
    assert!(super::terminal_state::has_exact_approver(
        &world,
        approver.principal_entity_id()
    ));
}

fn assert_requested_without_approval(
    world: &super::approval_transition::World,
    requested: &WorthQueryRequestedElevation,
    approver: worth_relational::facade::identity::EntityId,
) {
    assert_eq!(
        requested.elevation_identity(),
        &worth_foundational::facade::AspectValue::String(
            worth_foundational::facade::InternedString::from("elevation-2"),
        )
    );
    assert_eq!(
        super::terminal_state::elevation_status(world),
        CapabilityElevationStatus::Requested
    );
    assert!(!super::terminal_state::has_exact_approver(world, approver));
}
