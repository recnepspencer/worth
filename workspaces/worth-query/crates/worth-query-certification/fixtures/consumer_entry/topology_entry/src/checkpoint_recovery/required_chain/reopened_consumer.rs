//! An ordinary reopened downstream renews its actual checkpoint prerequisites.
use super::*;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn reopened_consumer_renews_a_current_checkpoint_root_without_a_separate_root_demand() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        32,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        source_world::seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut root = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let original_root = (0..64)
        .find_map(|_| match root.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("the original native root settles");
    assert_eq!(original_root.producer_contacts_in_this_demand(), 1);
    let mut consumer = request
        .demand(ChainDemand("anchor-b".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let original = (0..64)
        .find_map(|_| match consumer.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("the original native consumer settles");
    assert_eq!(original.producer_contacts_in_this_demand(), 1);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    // A real equal-value source revision activates the program checkpoint path.
    // Only the reopened demand below proves the recovered root Current.
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(1),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3380_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop((
        source,
        root,
        consumer,
        original_root,
        original,
        principal,
        scope,
    ));
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    drop(application);

    let reopened = support::install_program::<program::ChainProgram>(Some(checkpoint), profile);
    let (scope, principal) = authenticate(&reopened);
    let request = reopened.request(&principal, &scope);
    let before = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(before.rows()[0].value, length(16));
    let mut consumer = request
        .demand(ChainDemand("anchor-b".into()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&reopened)
        .unwrap();
    let renewed = (0..64)
        .find_map(|_| match consumer.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("the reopened downstream discharges its own native prerequisite");
    assert_eq!(renewed.producer_contacts_in_this_demand(), 1);
    let selected = request.retain_read().unwrap();
    request
        .at(&selected)
        .require_current_output_demand(&renewed, NonZeroUsize::new(1_000_000).unwrap())
        .unwrap();
    let after = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(after.rows()[0].value, length(16));
    let warm = (0..64)
        .find_map(|_| match consumer.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
        })
        .expect("warm reopened demand settles");
    assert_eq!(warm.producer_contacts_in_this_demand(), 0);

    // A real edit to the recovered output invalidates its sealed native witness.
    // Static custody cannot certify the old root or invent a refresh source.
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .mutate(crate::PlanarEdit(crate::PlanarMutation {
            scope_key: "anchor-a".into(),
            operation: worth_query_consumer_values::PlanarOperation::PublishDerivedOutput(
                worth_query_consumer_values::PlanarDerivedOutput {
                    body_key: "anchor-a".into(),
                    value: length(9),
                },
            ),
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3381_u64)
        .execute_in_program(
            &reopened,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let changed = request.retain_read().unwrap();
    assert!(request
        .at(&changed)
        .require_current_output_demand(&renewed, NonZeroUsize::new(1_000_000).unwrap())
        .is_err());
    assert!(matches!(consumer.advance(&request).unwrap(), WorthQueryApplicationOutputDemandProgress::Pending),
        "a changed static root waits for genuine source admission instead of certifying the old consumer");
    let after_denial = request
        .query(PlanarOutputRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        after_denial.rows(),
        after.rows(),
        "denial must not publish a consumer effect"
    );
    drop(consumer);
}
