//! Stopped discovered handoffs retain source/output ownership; abandonment is not close.
use super::*;
use worth_query_host::facade::admission::authenticated_principal::{
    WorthQueryCancellationSource, WorthQueryRequestScope,
};
use worth_query_host::facade::application_contribution::WorthQueryAdvancementDenial;
use worth_query_host::facade::primary_graph::WorthQueryExecutionObservation;
#[test]
fn discovered_ordinary_stopped_source_and_output_handoffs_keep_original_owner() {
    let runtime = installation::install(1, 0);
    let adapter = authentication::admit(runtime.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = runtime.request(&principal, &scope);
    let cancellation = WorthQueryCancellationSource::new();
    let stopped = WorthQueryRequestScope::new(None, cancellation.token(), worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system());
    let stopped_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &stopped,
    ))
    .unwrap();
    let stopped_request = runtime.request(&stopped_principal, &stopped);
    let original = input("focus-a");
    reset_planar_source_adjustment_contacts(&original);
    let owner = retain(&runtime, &request, "focus-a", 222_001);
    let branch = owner.branch();
    let before = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    cancellation.cancel();
    let owner = refuse(owner, &stopped_request);
    assert_eq!(owner.branch(), branch);
    assert_eq!(planar_source_adjustment_contacts(&original), (0, 0));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "focus-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(1)
    );
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: population::target("focus-a", 0)
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(1)
    );
    let owner = match owner.advance(&request) {
        WorthQueryOperationAdvance::Progress {
            operation,
            progress: WorthQueryOperationProgress::SourcePerformed,
        } => operation,
        _ => panic!("fresh source admission must perform the original source once"),
    };
    let committed = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert_ne!(committed, before);
    assert_eq!(owner.handler_result().unwrap().changed_vertices, 1);
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: "focus-a".into()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
    // SourcePerformed has already co-published the required target (seed y=1, output y+1).
    assert_eq!(
        request
            .query(PlanarOutputRead {
                body_key: population::target("focus-a", 0)
            })
            .execute()
            .unwrap()
            .rows()[0]
            .value,
        length(2)
    );
    let owner = refuse(owner, &stopped_request);
    assert_eq!(owner.branch(), branch);
    assert_eq!(owner.handler_result().unwrap().changed_vertices, 1);
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &committed
    );
    let (owner, denial) = match owner.close(&request) {
        Err(refusal) => refusal,
        Ok(_) => {
            panic!("retained native output custody still requires lawful settlement and close")
        }
    };
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::NativeObligationRetained
    ));
    let owner = match owner.advance(&request) {
        WorthQueryOperationAdvance::Progress {
            operation,
            progress: WorthQueryOperationProgress::OutputsStarted,
        } => operation,
        _ => panic!("original native performed source must start its outputs"),
    };
    let before_output_stop = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert_eq!(before_output_stop, committed);
    let owner = refuse(owner, &stopped_request);
    assert_eq!(owner.branch(), branch);
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &before_output_stop,
        "canceled output advancement publishes no product change"
    );
    let published = request
        .query(PlanarOutputRead {
            body_key: population::target("focus-a", 0),
        })
        .execute()
        .unwrap();
    assert_eq!(
        published.rows()[0].value,
        length(2),
        "cancellation preserves the already co-published target"
    );
    drop(published);
    let settled = settle(owner, &request);
    assert_eq!(settled.branch(), branch);
    assert_eq!(settled.handler_result().unwrap().changed_vertices, 1);
    let tree = settled.settlement();
    assert_eq!(tree.source_observation().selected_commit(), &committed);
    assert_eq!(tree.observation().selected_commit(), &committed);
    assert_eq!(tree.root_outputs().count(), 1);
    assert_eq!(tree.output_count(), 2);
    assert_eq!(tree.superseded_roots().count(), 0);
    for (_, root) in tree.root_outputs() {
        assert_eq!(root.observation().selected_commit(), &committed);
        assert_eq!(root.posture(), WorthQueryOutputSettlementPosture::Performed);
        assert_eq!(root.producer_contacts_in_this_demand(), 0);
    }
    let (_, child) = tree
        .outputs_for::<Schema, PlanarOutputToLateFinalConnection>()
        .next()
        .unwrap();
    assert_eq!(child.observation().selected_commit(), &committed);
    assert_eq!(child.producer_contacts_in_this_demand(), 0);
    let quiet_head = request
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    assert_eq!(quiet_head, committed);
    let delivery = settled.read_current(&request).unwrap();
    assert_eq!(delivery.rows().len(), 1);
    assert_eq!(
        delivery.rows()[0].value,
        length(2),
        "fresh authorized delivery reuses the original required output"
    );
    let (settled, denial) = match settled.close(&stopped_request) {
        Err(refusal) => refusal,
        Ok(_) => panic!("stopped fresh close must preserve the settled owner"),
    };
    assert!(matches!(
        denial,
        WorthQueryOperationCloseDenial::Execution(WorthQueryAdvancementDenial::Interrupted(_))
    ));
    let retried = settled.read_current(&request).unwrap();
    assert_eq!(delivery.rows(), retried.rows());
    let first_receipt = delivery.receipt().inspect();
    let second_receipt = retried.receipt().inspect();
    let first_basis = first_receipt.basis();
    let second_basis = second_receipt.basis();
    assert_eq!(
        (
            first_basis.runtime_instance(),
            first_basis.branch(),
            first_basis.version(),
            first_basis.posture()
        ),
        (
            second_basis.runtime_instance(),
            second_basis.branch(),
            second_basis.version(),
            second_basis.posture()
        )
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &quiet_head,
        "quiet reads and refused close cannot change the World commit"
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, d)| panic!("fresh lawful close: {d:?}"));
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &committed
    );
}
fn refuse(owner: Operation, request: &Request<'_>) -> Operation {
    match owner.advance(request) {
        WorthQueryOperationAdvance::Refused {
            operation,
            denial:
                WorthQueryOperationAdvanceDenial::Execution(WorthQueryAdvancementDenial::Interrupted(_)),
        } => operation,
        _ => panic!("native cancellation must return original owner before progression"),
    }
}
#[test]
fn discovered_ordinary_drop_and_forget_add_no_progress_or_publication() {
    for performed in [false, true] {
        for forget in [false, true] {
            let runtime = installation::install(1, 0);
            let adapter = authentication::admit(runtime.installed_schema());
            let scope = authentication::request_scope();
            let principal = authentication::block_on(adapter.authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ))
            .unwrap();
            let request = runtime.request(&principal, &scope);
            let original = input("focus-a");
            reset_planar_source_adjustment_contacts(&original);
            let owner = retain(&runtime, &request, "focus-a", 222_002);
            let unperformed = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            let branch = owner.branch();
            let owner = if performed {
                match owner.advance(&request) {
                    WorthQueryOperationAdvance::Progress {
                        operation,
                        progress: WorthQueryOperationProgress::SourcePerformed,
                    } => operation,
                    _ => panic!("real original effect must precede abandonment"),
                }
            } else {
                owner
            };
            let before = request
                .retain_current_read()
                .unwrap()
                .selected_commit()
                .clone();
            assert_eq!(owner.branch(), branch);
            if performed {
                assert_ne!(before, unperformed);
                assert_eq!(owner.handler_result().unwrap().changed_vertices, 1);
            } else {
                assert_eq!(before, unperformed);
                assert!(owner.handler_result().is_none());
            }
            assert_eq!(
                planar_source_adjustment_contacts(&original),
                if performed { (1, 1) } else { (0, 0) }
            );
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: "focus-a".into()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(if performed { 2 } else { 1 })
            );
            assert_eq!(
                request.query(PlanarOutputRead { body_key: population::target("focus-a", 0) }).execute().unwrap().rows()[0].value,
                length(if performed { 2 } else { 1 }),
                "the original source transition alone publishes required output y+1 from seeded y=1"
            );
            let _observation = WorthQueryExecutionObservation::install_on_this_thread();
            worth_query_host::facade::primary_graph::advancement_observations_on_this_thread_for_test();
            if forget {
                std::mem::forget(owner);
            } else {
                drop(owner);
            }
            let reports = worth_query_host::facade::primary_graph::advancement_observations_on_this_thread_for_test();
            assert!(
                reports.is_empty(),
                "abandonment emits no funded progression/cleanup phase or completion event"
            );
            assert_eq!(
                request.retain_current_read().unwrap().selected_commit(),
                &before
            );
            assert_eq!(
                planar_source_adjustment_contacts(&original),
                if performed { (1, 1) } else { (0, 0) }
            );
            assert_eq!(
                request
                    .query(PlanarRead {
                        body_key: "focus-a".to_owned()
                    })
                    .execute()
                    .unwrap()
                    .rows()[0]
                    .y,
                length(if performed { 2 } else { 1 })
            );
            let preserved = request
                .query(PlanarOutputRead {
                    body_key: population::target("focus-a", 0),
                })
                .execute()
                .unwrap();
            assert_eq!(
                preserved.rows()[0].value,
                length(if performed { 2 } else { 1 }),
                "abandonment neither manufactures completion nor rolls back an original co-publication"
            );
            // Forget deliberately abandons wrapper-only custody. No durability,
            // physical reclamation, cleanup-success receipt or rollback is promised.
        }
    }
}
