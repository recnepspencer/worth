//! Bypass only authoring inside its owner, then exercise public commit lowering.
use worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_attempt::effect_program::model::WorthQueryApplicationRealizedEffect;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::program_fixture::admitted_program;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::authenticated_principal;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::idempotency;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::application_attempt::resolved_account;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::world_installation::installed_authorization_world;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::schema_types::live_scope;
use worth_query_testkit_execution_primary_graph_application_contribution::domain_computation::primary_graph::tests::fixture::AccountLabel;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialKind;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitDenialStage;
use worth_query_execution::facade::domain_computation::WorthQueryApplicationCommitOutcome;
use std::collections::BTreeMap;
use worth_execution::ExecutionAllocationPolicy;
use worth_foundational::facade::AspectValue;

#[test]
fn unobserved_effect_keeps_coordinates_at_public_lowering_without_idempotency_residue() {
    let world = installed_authorization_world(
        true,
        worth_query_admission::facade::authenticated_principal::WorthQueryAdmissionClock::system(),
    );
    let request = live_scope();
    let (principal, account, mut program, before) = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let principal = authenticated_principal(&host, &world, &request);
            let account = resolved_account(&host, &world, "open", &request);
            let program =
                admitted_program(&phase, &world, &principal, &account, &request, "changed");
            let before = world
                .selected_product(&host)
                .product()
                .observation_for_test()
                .clone();
            (principal, account, program, before)
        })
        .unwrap();
    let field = AccountLabel::reference();
    let locator = (*(*(*(*world.application.runtime_for_test()).primary_provider_for_test())
        .graph_for_test())
    .layout_for_test())
    .field_locator_for_test(field.entity(), field.aspect(), field.field())
    .unwrap();
    // The actual installed operation permits Label, but this genuine completed
    // decision observed only Status. No public raw-effect door is introduced.
    (*program.effects_mut_for_test()).push(WorthQueryApplicationRealizedEffect::UpdateEntity {
        entity: field.entity().to_owned(),
        entity_id: account.entity_id_for_test(),
        fields: BTreeMap::from([(
            locator.clone(),
            AspectValue::String("unobserved-label".into()),
        )]),
    });
    let WorthQueryApplicationCommitOutcome::Denied(denial) =
        world.application.compare_and_commit_application(
            program,
            idempotency(14, 14),
            ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("public lowering must refuse the unobserved field");
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::UnobservedWrite
    );
    assert_eq!(
        denial.stage(),
        WorthQueryApplicationCommitDenialStage::EffectLowering
    );
    let cause = denial
        .unobserved_write()
        .expect("the exact original lowering coordinates");
    assert_eq!(cause.entity_id(), account.entity_id_for_test());
    assert_eq!(cause.locator(), locator);
    let after = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request_for_test();
            let unchanged = resolved_account(&host, &world, "open", &request);
            assert_eq!(
                unchanged.entity_id_for_test(),
                account.entity_id_for_test(),
                "even the preceding Status effect did not publish"
            );
            world
                .selected_product(&host)
                .product()
                .observation_for_test()
                .clone()
        })
        .unwrap();
    assert_eq!(before, after, "effect lowering did not move truth");
    let lawful = world
        .application
        .with_host_advancement(|phase| {
            admitted_program(&phase, &world, &principal, &account, &request, "changed")
        })
        .unwrap();
    assert!(
        matches!(
            world.application.compare_and_commit_application(
                lawful,
                idempotency(14, 14),
                ExecutionAllocationPolicy::SystemAllocation,
            ),
            WorthQueryApplicationCommitOutcome::Committed(_)
        ),
        "the refused lowering left no idempotency record"
    );
}
