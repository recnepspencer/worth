//! Bypass only authoring inside its owner, then exercise public commit lowering.
use crate::domain_computation::primary_graph::{
    application_attempt::effect_program::WorthQueryApplicationRealizedEffect,
    tests::{
        application_attempt::{
            admitted_program, authenticated_principal, idempotency, resolved_account,
        },
        fixture::{installed_authorization_world, live_scope, AccountLabel},
    },
    WorthQueryApplicationCommitDenialKind, WorthQueryApplicationCommitDenialStage,
    WorthQueryApplicationCommitOutcome,
};
use std::collections::BTreeMap;
use worth_execution::ExecutionAllocationPolicy;
use worth_foundational::facade::AspectValue;

#[test]
fn unobserved_effect_keeps_coordinates_at_public_lowering_without_idempotency_residue() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (principal, account, mut program, before) = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let principal = authenticated_principal(&host, &world, &request);
            let account = resolved_account(&host, &world, "open", &request);
            let program =
                admitted_program(&phase, &world, &principal, &account, &request, "changed");
            let before = world
                .selected_product(&host)
                .product()
                .observation()
                .clone();
            (principal, account, program, before)
        })
        .unwrap();
    let field = AccountLabel::reference();
    let locator = world
        .application
        .primary_provider
        .graph
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .unwrap();
    // The actual installed operation permits Label, but this genuine completed
    // decision observed only Status. No public raw-effect door is introduced.
    program
        .effects
        .push(WorthQueryApplicationRealizedEffect::UpdateEntity {
            entity: field.entity().to_owned(),
            entity_id: account.entity_id(),
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
    assert_eq!(cause.entity_id(), account.entity_id());
    assert_eq!(cause.locator(), locator);
    let after = world
        .application
        .with_host_advancement(|phase| {
            let host = phase.test_host_request();
            let unchanged = resolved_account(&host, &world, "open", &request);
            assert_eq!(
                unchanged.entity_id(),
                account.entity_id(),
                "even the preceding Status effect did not publish"
            );
            world
                .selected_product(&host)
                .product()
                .observation()
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
