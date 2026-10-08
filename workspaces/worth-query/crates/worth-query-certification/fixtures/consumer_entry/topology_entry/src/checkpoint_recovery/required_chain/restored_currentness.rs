//! A checkpoint root does not make an old output's retained facts current.

use super::*;
use worth_query_consumer_values::{PlanarDerivedOutput, PlanarOperation};
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

#[test]
fn output_changed_before_capture_cannot_become_unverified_restored_ready() {
    let _guard = checkpoint_recovery_test_guard();
    let application = support::install_program::<program::ChainProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let initial = (0..256)
        .find_map(|_| match demand.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("A initially settles through the actual producer");
    assert_eq!(initial.producer_contacts_in_this_demand(), 1);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    // Activate the existing program through its declared source operation.
    // The equal write preserves the source value and native field revision.
    request
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(1),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3300_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    drop(source);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-a".to_owned(),
                value: length(99),
            }),
            validator_work: 4_096,
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3301_u64)
        .execute_in_program::<program::ChainProgram>(&application)
        .expect("the independent output edit is admitted");
    let after_source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        source.rows(),
        after_source.rows(),
        "the source projection stays equal while the published output changes"
    );
    let changed = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(changed.rows()[0].value, length(99));
    drop((
        changed,
        source,
        after_source,
        initial,
        demand,
        principal,
        scope,
    ));
    let checkpoint = application
        .capture_application_checkpoint(CapturePolicy::SystemAllocation)
        .unwrap();
    drop(application);

    super::super::producer::reset_provider_contacts();
    let restored =
        support::install_program::<program::ChainProgram>(Some(checkpoint), Default::default());
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let before = request.retain_read().unwrap();
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&restored)
        .unwrap();
    let settlement = (0..256)
        .find_map(|_| match demand.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending => None,
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
        })
        .expect("the reopened demand settles after checking its old output");
    assert_eq!(
        settlement.producer_contacts_in_this_demand(),
        1,
        "stale retained output facts require actual fresh execution"
    );
    assert!(super::super::producer::provider_contacts() > 0);
    assert!(settlement.application_commit_receipt().is_some());
    let after = request.retain_read().unwrap();
    assert_ne!(before.selected_commit(), after.selected_commit());
    let output = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(output.rows()[0].value, length(2));
}
