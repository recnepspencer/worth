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
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => None,
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
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(1),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3300_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(source);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarEdit(PlanarMutation {
            scope_key: "anchor-a".to_owned(),
            operation: PlanarOperation::PublishDerivedOutput(PlanarDerivedOutput {
                body_key: "anchor-a".to_owned(),
                value: length(99),
            }),
        }))
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_3301_u64)
        .execute_in_program::<program::ChainProgram>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
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
        .with_host_advancement(|host_phase| {
            application
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .expect("installed host call opens");
    drop(application);

    super::super::producer::reset_provider_contacts();
    let restored =
        support::install_program::<program::ChainProgram>(Some(checkpoint), Default::default());
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let before = crate::checkpoint_recovery::support::retain_read(&restored, &request).unwrap();
    let mut demand = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&restored)
        .unwrap();
    let settlement = (0..256)
        .find_map(|_| match demand.advance(&request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Pending
            | WorthQueryApplicationOutputDemandProgress::Progressed => None,
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
    let after = crate::checkpoint_recovery::support::retain_read(&restored, &request).unwrap();
    assert_ne!(before.selected_commit(), after.selected_commit());
    let output = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(output.rows()[0].value, length(2));
}

/// A restored dependent keeps its original native owner across a real source edit.
#[test]
fn restored_dependent_refresh_keeps_selected_ready_custody() {
    let _guard = checkpoint_recovery_test_guard();
    macro_rules! settle {
        ($owner:ident, $request:ident) => {
            (0..256)
                .find_map(|_| {
                    match $owner.advance(&$request).unwrap_or_else(|stop| {
                        panic!("{} native advance: {stop:?}", stringify!($owner))
                    }) {
                        WorthQueryApplicationOutputDemandProgress::Pending
                        | WorthQueryApplicationOutputDemandProgress::Progressed => None,
                        WorthQueryApplicationOutputDemandProgress::Settled(value) => Some(value),
                    }
                })
                .expect("the original native chain owner settles")
        };
    }
    let application = support::install_program::<program::ChainProgram>(None, Default::default());
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(a, request);
    settle!(b, request);
    settle!(c, request);
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows()[0].y, length(1));
    let edit = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(5),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_6905_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(edit, worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)), "the first source edit performs");
    drop(edit);
    settle!(c, request);
    // seed_cycle authors B.Y=1 and C.Y=10; their handlers preserve Y+1.
    for (key, expected) in [("anchor-a", 6), ("anchor-b", 2), ("anchor-c", 11)] {
        let rows = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            length(expected),
            "authored pre-capture truth {key}"
        );
    }
    let checkpoint = application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    a.close();
    b.close();
    c.close();
    drop((source, a, b, c, principal, scope));
    drop(application);

    let restored =
        support::install_program::<program::ChainProgram>(Some(checkpoint), Default::default());
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    // Restore qualifies every upstream through its own real owner before C,
    // just as the original current graph did; serialized facts issue no live read authority.
    let mut a = request
        .demand(PlanarOutputDemand::new("anchor-a"))
        .controls(input_cutoff::controls())
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&restored)
        .unwrap();
    settle!(a, request);
    let mut b = request
        .demand(ChainDemand("anchor-b".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&restored)
        .unwrap();
    settle!(b, request);
    let mut c = request
        .demand(ChainDemand("anchor-c".into()))
        .controls(input_cutoff::controls())
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&restored)
        .unwrap();
    let original = settle!(c, request);
    let original_entity = original
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    for (key, expected) in [("anchor-a", 6), ("anchor-b", 2), ("anchor-c", 11)] {
        let rows = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            length(expected),
            "restored current truth {key}"
        );
    }
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows()[0].y, length(5));
    let edit = request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(6),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&0x9176_6906_u64)
        .execute_performed::<program::ChainProgram, program::ChainRoot>(
            &restored,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    assert!(matches!(edit, worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome::Performed(_)), "the restored source edit performs");
    drop(edit);
    let after_source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        after_source.rows()[0].y,
        length(6),
        "source publication really performed"
    );
    let settled = settle!(c, request);
    assert_eq!(
        settled
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        original_entity
    );
    for (key, expected) in [("anchor-a", 7), ("anchor-b", 2), ("anchor-c", 11)] {
        let rows = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            rows.rows()[0].value,
            length(expected),
            "independent post-edit truth {key}"
        );
    }
    a.close();
    b.close();
    c.close();
}
