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
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, requested) = super::approval_transition::requested_world(
            &host_custody,
            CapabilityElevationScenario::AlternateCurrentGrant,
        );
        revoke_current_capability(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            &world,
        );
        let approver =
            super::approval_transition::authenticated(&host_custody, &world, "bob", &request);
        let access =
            super::approval_transition::approval_access(&host_custody, &world, &approver, &request)
                .expect("the independent approval command grant remains current");

        let Err(denial) = world.application.authorize_elevation_approval(
            &host_custody.phase_for_application(&world.application),
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
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            &denial.into_requested(),
            approver.principal_entity_id(),
        );
    })
}

#[test]
fn exact_support_revocation_after_materialization_denies_provider_commit() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, requested) = super::approval_transition::requested_world(
            &host_custody,
            CapabilityElevationScenario::Active,
        );
        let approver =
            super::approval_transition::authenticated(&host_custody, &world, "bob", &request);
        let program = super::approval_transition::materialize_exact_approval(
            &host_custody,
            &world,
            &request,
            requested,
        );
        revoke_current_capability(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            &world,
        );

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
        assert_requested_without_approval(
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            &requested,
            approver.principal_entity_id(),
        );
    })
}

#[test]
fn unrelated_published_lifecycle_drift_stales_the_selected_product_mutation() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, requested) = super::approval_transition::requested_world(
            &host_custody,
            CapabilityElevationScenario::Active,
        );
        let program = super::approval_transition::materialize_exact_approval(
            &host_custody,
            &world,
            &request,
            requested,
        );
        super::mutation::set_status(
            &host_custody,
            &host_custody.borrowed_request_for_test(),
            &world,
            "elevation-1",
            CapabilityElevationStatus::Revoked,
        );

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
            super::terminal_state::elevation_status(
                &(host_custody.phase_for_application(&world.application)).test_host_request(),
                &host_custody.borrowed_request_for_test(),
                &world
            ),
            CapabilityElevationStatus::Requested
        );
    })
}

#[path = "approval_support_currentness/support_expiry_denies_approval_while_the_request_lifecycle_is_cu.rs"]
mod support_expiry_denies_approval_while_the_request_lifecycle_is_cu;

#[path = "approval_support_currentness/replacement_policy_path_for_the_same_grant_cannot_launder_reques.rs"]
mod replacement_policy_path_for_the_same_grant_cannot_launder_reques;

#[test]
fn replacement_policy_path_for_the_same_grant_cuts_off_approved_use() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let borrowed_execution = host_custody.borrowed_request_for_test();
        let execution = &borrowed_execution;

        let (world, request, approved) =
            super::approval_transition::exact_approved_world(&host_custody);
        let requester =
            super::approval_transition::authenticated(&host_custody, &world, "alice", &request);
        super::mutation::replace_support_grantor_with_custodian(
            &host_custody,
            execution,
            &world,
            requester.principal_entity_id(),
        );
        world.authorization_time.script([time(100)]);
        let capability = super::installed_capability(&world);

        let Err(denial) = world
            .selected_product(&host_custody)
            .admit_approved_elevation_access(
                &host_custody.phase_for_application(&world.application),
                &approved,
                &requester,
                &capability,
                super::elevated_input(Some("elevation-2")),
                &request,
            )
        else {
            panic!("a new allowing path must not replace approved support for active use");
        };

        assert_eq!(
            denial.kind(),
            WorthQueryOperationAuthorizationDenialKind::StaleAuthorization
        );
        assert_eq!(
            super::terminal_state::elevation_status(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world
            ),
            CapabilityElevationStatus::Approved
        );
    })
}

#[test]
fn request_expiry_after_approval_materialization_denies_provider_commit() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, requested) = super::approval_transition::requested_world(
            &host_custody,
            CapabilityElevationScenario::Active,
        );
        let approver =
            super::approval_transition::authenticated(&host_custody, &world, "bob", &request);
        let program = super::approval_transition::materialize_exact_approval(
            &host_custody,
            &world,
            &request,
            requested,
        );
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
        assert_requested_without_approval(
            &host_custody.phase_for_application(&world.application),
            &host_custody.borrowed_request_for_test(),
            &world,
            &requested,
            approver.principal_entity_id(),
        );
    })
}

#[test]
fn exact_approval_authorization_carries_three_graph_facts_and_derive_only_the_governed_input() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let (world, request, requested) = super::approval_transition::requested_world(
            &host_custody,
            CapabilityElevationScenario::Active,
        );
        let approver =
            super::approval_transition::authenticated(&host_custody, &world, "bob", &request);
        let access =
            super::approval_transition::approval_access(&host_custody, &world, &approver, &request)
                .unwrap();
        let admission = world
            .application
            .authorize_elevation_approval(
                &host_custody.phase_for_application(&world.application),
                requested,
                access,
                &super::approval_transition::approval_operation(&world),
                Default::default(),
            )
            .unwrap();

        assert_eq!(admission.authorization_decision_fact_count(), 3);
        assert_eq!(admission.graph_work_decision_fact_count(), 3);
        let work = admission.canonical_work().admission();
        // The one canonical derivation is the governed input the admission encoded.
        assert_eq!(work.basis_preparations(), 0);
        assert_eq!(work.digest_derivations(), 1);
        assert_eq!(work.canonical_encoded_bytes(), 68);
        assert_eq!(work.sha256_input_bytes(), 157);
        assert_eq!(work.sha256_compression_blocks(), 3);
        assert_eq!(work.digest_text_materializations(), 0);
    })
}

#[test]
fn request_command_resource_can_differ_from_the_governed_resource_it_progresses() {
    crate::domain_computation::primary_graph::with_test_advancement(|host_custody| {
        let borrowed_execution = host_custody.borrowed_request_for_test();
        let execution = &borrowed_execution;

        let mut input = super::request_transition::honest_input();
        input.account = "account-2".to_owned();
        assert_eq!(input.target_account, "account-1");
        assert_eq!(input.grant, "capability-1");
        let (world, request, requested) = super::approval_transition::requested_world_with_input(
            &host_custody,
            CapabilityElevationScenario::DistinctCommandResource,
            input,
        );
        let approver =
            super::approval_transition::authenticated(&host_custody, &world, "bob", &request);

        let approved = super::approval_transition::approve_exact_request(
            &host_custody,
            &world,
            &request,
            requested,
        );

        assert_eq!(approved.approver(), approver.principal_entity_id());
        assert_eq!(
            super::terminal_state::elevation_status(
                &host_custody,
                &host_custody.borrowed_request_for_test(),
                &world
            ),
            CapabilityElevationStatus::Approved
        );
        assert!(super::terminal_state::has_exact_approver(
            &host_custody,
            execution,
            &world,
            approver.principal_entity_id()
        ));
    })
}

fn assert_requested_without_approval(
    phase_custody: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
    execution: &worth_execution::ExecutionRequest<'_, '_>,
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
        super::terminal_state::elevation_status(
            &(phase_custody).test_host_request(),
            execution,
            world
        ),
        CapabilityElevationStatus::Requested
    );
    assert!(!super::terminal_state::has_exact_approver(
        &(phase_custody).test_host_request(),
        execution,
        world,
        approver
    ));
}
