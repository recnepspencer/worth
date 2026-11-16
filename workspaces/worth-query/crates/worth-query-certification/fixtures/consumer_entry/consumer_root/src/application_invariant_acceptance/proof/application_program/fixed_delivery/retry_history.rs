use super::*;

#[test]
fn fixed_delivery_result_contract_retry_unrelated_progress_and_consumed_fact_history() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    let world = installation::install(&crate::installed_schema());
    let app = &world.application;
    let adapter = authentication::admit(app.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = app.request(&principal, &scope);
    let original = PlanarSourceAdjustment {
        scope_key: "anchor-a".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&original);
    let initial_contacts = app.producer_contacts_on_this_thread_for_test();
    let settled = settle!(
        request,
        world,
        ConsumerProgram,
        ConsumerFixedAction,
        original.clone(),
        10_121
    );
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test() - initial_contacts,
        5
    );
    let head = settled.settlement().observation().selected_commit().clone();
    let outputs = settled.settlement();
    let source = outputs.performed_source().unwrap();
    let source_receipt = source.receipt().retained_receipt().unwrap().clone();
    let fulfillment = source.root_fulfillment();
    assert!(fulfillment.matches_source(&source_receipt));
    let publication = source_receipt.committed_product_publication();
    assert_eq!(fulfillment.source_commit(), publication.composite_commit());
    assert_eq!(
        fulfillment.output_commit(),
        outputs.root_observation().selected_commit()
    );
    assert_eq!(fulfillment.posture(), outputs.root_settlement().posture());
    let contacts = app.producer_contacts_on_this_thread_for_test();
    let policy = WorthQueryOperationDeliveryPolicy::default();
    assert!(matches!(settled.read_current_with_policy(&request, policy.query(NonZeroUsize::new(2).unwrap())), Err(WorthQueryOperationDeliveryDenial::Query(worth_query_host::facade::application_entry::WorthQueryApplicationRequestQueryDenial::Limit(_)))));
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let request = app.request(&fresh_principal, &fresh_scope);
    let read = settled.read_current_with_policy(&request, policy).unwrap();
    assert_eq!(read.rows()[0].value, length(3));
    drop(read);
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        &head,
        "delivery refusals and retry publish no source or output"
    );
    assert_eq!(outputs.root_producer_contacts_in_this_demand(), 0);
    assert_eq!(app.producer_contacts_on_this_thread_for_test(), contacts);
    let unrelated = settle!(
        request,
        world,
        ConsumerProgram,
        ConsumerFixedAction,
        PlanarSourceAdjustment {
            scope_key: "remote-a".to_owned(),
            replacement_y: length(42)
        },
        10_122
    );
    unrelated
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("unrelated close: {denial:?}"));
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(3)
    );
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
    assert!(
        source
            .receipt()
            .retained_receipt()
            .unwrap()
            .is_same_authoritative_commit(&source_receipt),
        "later output progress cannot replace original action provenance"
    );
    // Correlate actual handler entries with source roots, including the two
    // different producer families at anchor-a. The alternate summary's entire
    // final:anchor-b ring is unchanged, so its current output can be retained.
    let roots = ["anchor-a", "anchor-b", "final:anchor-a", "final:anchor-b"].map(|key| {
        let source = request
            .query(PlanarRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        let root = source.observed_sources()[0].root_entity_for_test();
        (
            root,
            app.producer_contacts_at_root_on_this_thread_for_test(root),
        )
    });
    let (_, old_summary) = outputs
        .outputs_for::<ConsumerSchema, PlanarAlternateFinalToSummaryConnection>()
        .next()
        .unwrap();
    let old_summary_anchor = old_summary
        .outputs_of::<FinalPlanarOutputs>()
        .unwrap()
        .entity::<FinalAnchorOutput<ConsumerSchema>>()
        .unwrap()
        .entity_id();
    let changed = settle!(
        request,
        world,
        ConsumerProgram,
        ConsumerFixedAction,
        PlanarSourceAdjustment {
            scope_key: "anchor-a".to_owned(),
            replacement_y: length(4)
        },
        10_123
    );
    for ((root, before), expected) in roots.into_iter().zip([2, 1, 1, 0]) {
        assert_eq!(
            app.producer_contacts_at_root_on_this_thread_for_test(root) - before,
            expected,
            "changed routes execute; the verified alternate summary does not"
        );
    }
    let changed_outputs = changed.settlement();
    let required = changed_outputs
        .outputs_for::<ConsumerSchema, PlanarOutputToFinalConnection>()
        .chain(
            changed_outputs.outputs_for::<ConsumerSchema, PlanarOutputToAlternateFinalConnection>(),
        )
        .chain(changed_outputs.outputs_for::<ConsumerSchema, PlanarFinalToSummaryConnection>())
        .chain(
            changed_outputs
                .outputs_for::<ConsumerSchema, PlanarAlternateFinalToSummaryConnection>(),
        )
        .collect::<Vec<_>>();
    assert_eq!(
        required
            .iter()
            .map(|(d, _)| d.body_key())
            .collect::<Vec<_>>(),
        ["anchor-a", "anchor-b", "final:anchor-a", "final:anchor-b"]
    );
    let (_, summary) = required[3];
    assert_eq!(
        summary
            .outputs_of::<FinalPlanarOutputs>()
            .unwrap()
            .entity::<FinalAnchorOutput<ConsumerSchema>>()
            .unwrap()
            .entity_id(),
        old_summary_anchor,
        "verified reuse retains the actual original output correspondence"
    );
    let before_reads = app.producer_contacts_on_this_thread_for_test();
    let current = request.retain_current_read().unwrap();
    // Independent y+1/y+2 arithmetic, including both transitive summaries.
    for (key, expected) in [
        ("anchor-a", 5),
        ("final:anchor-a", 6),
        ("final:final:anchor-a", 8),
        ("final:anchor-b", 3),
        ("final:final:anchor-b", 5),
    ] {
        let read = request
            .query(PlanarOutputRead {
                body_key: key.into(),
            })
            .execute()
            .unwrap();
        assert_eq!(read.rows()[0].body_key, key);
        assert_eq!(read.rows()[0].value, length(expected));
    }
    assert_eq!(
        changed.read_current(&request).unwrap().rows()[0].value,
        length(5)
    );
    assert_eq!(
        app.producer_contacts_on_this_thread_for_test(),
        before_reads
    );
    assert_eq!(
        request.retain_current_read().unwrap().selected_commit(),
        current.selected_commit(),
        "whole-tree current delivery and known-answer reads publish nothing"
    );
    changed
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("changed close: {denial:?}"));
    assert!(matches!(
        settled.read_current(&request),
        Err(WorthQueryOperationDeliveryDenial::Stale(_))
    ));
    assert_eq!(
        settled.read_historical(&request).unwrap().rows()[0].value,
        length(3)
    );
    settled
        .close(&request)
        .unwrap_or_else(|(_, denial)| panic!("stale close: {denial:?}"));
    assert_eq!(planar_source_adjustment_contacts(&original), (1, 1));
}
