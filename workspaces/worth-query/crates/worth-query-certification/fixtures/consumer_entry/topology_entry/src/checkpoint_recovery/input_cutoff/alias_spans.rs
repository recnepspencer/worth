//! Ordinary eligible aliases checkpoint the latest source suffix, not origin facts.
use super::*;
use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::application_entry::WorthQueryOutputSettlementPosture;

#[test]
fn latest_stable_alias_checkpoint_restores_output_and_source_change_requires_fresh_execution() {
    let _guard = checkpoint_recovery_test_guard();
    let application = install(None);
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let first = direct_demand(&request, &application, "anchor-a");
    assert_eq!(
        first.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    let original = first
        .outputs_of::<PlanarOutputs>()
        .unwrap()
        .entity::<PlanarAnchorOutput<CheckpointSchema>>()
        .unwrap()
        .entity_id();
    for (value, command) in [(2, 0x32a1_u64), (3, 0x32a2)] {
        let source = request
            .query(PlanarRead {
                body_key: "anchor-b".into(),
            })
            .execute()
            .unwrap();
        request
            .advanced()
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-b".into(),
                replacement_y: length(value),
            })
            .expect_source(source.observed_sources()[0].clone())
            .idempotency(&command)
            .execute_performed::<CheckpointProgram, CheckpointRoot>(
                &application,
                worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap();
        drop(source);
        let before = support::retain_read(&application, &request).unwrap();
        let alias = direct_demand(&request, &application, "anchor-a");
        let after = support::retain_read(&application, &request).unwrap();
        assert_eq!(
            alias.posture(),
            WorthQueryOutputSettlementPosture::StableReused
        );
        assert_eq!(alias.producer_contacts_in_this_demand(), 0);
        assert!(alias.application_commit_receipt().is_none());
        assert_eq!(before.selected_commit(), after.selected_commit());
        assert_eq!(
            alias
                .outputs_of::<PlanarOutputs>()
                .unwrap()
                .entity::<PlanarAnchorOutput<CheckpointSchema>>()
                .unwrap()
                .entity_id(),
            original
        );
        let input = request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        let output = request
            .query(PlanarOutputRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap();
        assert_eq!(
            PositiveLength::get(&output.rows()[0].value),
            PositiveLength::get(&input.rows()[0].y) + 1
        );
    }
    drop(first);
    let checkpoint = application
        .with_host_advancement(|phase| {
            application
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    // Keep the captured alias checkpoint, then invalidate that actual live
    // alias. This fresh execution must not replace the checkpoint subject.
    require_fresh_after_root_edit(&application, &request, 0x32a3);
    drop(principal);
    drop(scope);
    drop(application);
    let restored = install(Some(checkpoint));
    let (scope, principal) = authenticate(&restored);
    let request = restored.request(&principal, &scope);
    let before = support::retain_read(&restored, &request).unwrap();
    let recovered = direct_demand(&request, &restored, "anchor-a");
    let after = support::retain_read(&restored, &request).unwrap();
    // Ordinary capture retains native prior custody; its reuse payload is
    // best effort. Fresh execution and native revalidated alias recovery have
    // different observable consequences, and neither reconstructs live proof.
    match recovered.posture() {
        WorthQueryOutputSettlementPosture::Performed => {
            assert_eq!(recovered.producer_contacts_in_this_demand(), 1);
            let receipt = recovered.application_commit_receipt().unwrap();
            assert_eq!(
                receipt.committed_product_publication().composite_commit(),
                after.selected_commit()
            );
            assert_ne!(before.selected_commit(), after.selected_commit());
            assert_eq!(
                recovered
                    .readiness_delivery()
                    .unwrap()
                    .producer_contact_count(),
                1
            );
        }
        WorthQueryOutputSettlementPosture::RecoveredStableReused => {
            assert_eq!(recovered.producer_contacts_in_this_demand(), 0);
            assert!(recovered.application_commit_receipt().is_none());
            assert!(recovered.readiness_delivery().is_none());
            assert_eq!(before.selected_commit(), after.selected_commit());
            assert!(recovered.checkpoint_readmission_work_units() > 0);
        }
        posture => panic!("alias B recovery returned an unexpected posture: {posture:?}"),
    }
    assert_eq!(
        recovered
            .outputs_of::<PlanarOutputs>()
            .unwrap()
            .entity::<PlanarAnchorOutput<CheckpointSchema>>()
            .unwrap()
            .entity_id(),
        original
    );
    let input = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    let successor = request
        .query(PlanarRead {
            body_key: "anchor-b".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(input.rows()[0].y, length(1));
    assert_eq!(successor.rows()[0].y, length(3));
    let output = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(
        PositiveLength::get(&output.rows()[0].value),
        PositiveLength::get(&input.rows()[0].y) + 1
    );
    request
        .at(&after)
        .require_current_output_demand(&recovered, Some(support::currentness_work_limit().get()))
        .unwrap();
    require_fresh_after_root_edit(&restored, &request, 0x32a4);
}

fn require_fresh_after_root_edit<'application>(
    application: &'application support::Application,
    request: &worth_query_host::facade::application_entry::WorthQueryApplicationRequest<
        'application,
        '_,
        '_,
        CheckpointSchema,
    >,
    command: u64,
) {
    let source = request
        .query(PlanarRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    request
        .advanced()
        .mutate(PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(7),
        })
        .expect_source(source.observed_sources()[0].clone())
        .idempotency(&command)
        .execute_performed::<CheckpointProgram, CheckpointRoot>(
            application,
            worth_query_host::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    drop(source);
    let fresh = direct_demand(request, application, "anchor-a");
    assert_eq!(
        fresh.posture(),
        WorthQueryOutputSettlementPosture::Performed
    );
    assert_eq!(fresh.producer_contacts_in_this_demand(), 1);
    let output = request
        .query(PlanarOutputRead {
            body_key: "anchor-a".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(PositiveLength::get(&output.rows()[0].value), 8);
    let current = support::retain_read(application, request).unwrap();
    request
        .at(&current)
        .require_current_output_demand(&fresh, Some(support::currentness_work_limit().get()))
        .unwrap();
}
