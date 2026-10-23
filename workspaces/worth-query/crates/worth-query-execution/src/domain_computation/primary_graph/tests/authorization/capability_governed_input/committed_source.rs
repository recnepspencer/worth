//! Performed occurrence meaning survives disposal of caller-owned receipts.
use super::*;

#[test]
fn original_governed_input_is_owned_by_the_committed_occurrence() {
    let world = installed_capability_authorization_world();
    world.authorization_time.script([time(100); 24]);
    let request = live_scope();
    let principal = world
        .application
        .with_host_advancement(|phase| {
            authenticated_principal(&phase.test_host_request(), &world, &request)
        })
        .expect("the installed host admits principal resolution");
    let (program, _) = world
        .application
        .with_host_advancement(|phase| {
            admitted_capability_program(&phase, &world, &principal, &request, "recorded-source")
        })
        .expect("the original capability and decision are admitted");
    let WorthQueryApplicationCommitOutcome::Committed(receipt) =
        world.application.compare_and_commit_application(
            program,
            idempotency(97, 97),
            crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
    else {
        panic!("the original operation must actually publish");
    };
    let source_commit = receipt.commit_reference().clone();
    let source_publication = receipt
        .committed_product_publication()
        .composite_commit()
        .clone();
    let source_operation = *receipt.installed_operation();
    let source_key = receipt.idempotency_binding();
    let copy = receipt.clone();
    drop(receipt);
    drop(copy);

    let occurrence = world
        .application
        .primary_provider
        .observe_completed_application(&source_commit)
        .expect("the provider retains the performed source independently of receipts");
    assert_eq!(
        occurrence
            .committed_product_publication()
            .composite_commit(),
        &source_publication
    );
    let original = occurrence.commit_evidence().original_authority();
    assert_eq!(original.installed_operation(), &source_operation);
    assert_eq!(original.idempotency_binding(), source_key);
    let input = original
        .retained_governed_input::<CapabilityTouchInput>()
        .expect("original input belongs to committed source evidence");
    assert_eq!(input.account, "account-1");
    assert_eq!(input.amount, 50);
    assert_eq!(input.caller_time, 100);
    assert!(original.retained_governed_input::<String>().is_none());
    assert!(original
        .retained_governed_input_carrier()
        .expect("the input retains its canonical association")
        .has_governed_identity());
}
