use super::*;
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

#[test]
fn recorded_fixed_steps_preview_the_qualified_old_output_after_later_edits() {
    let world = install();
    let application = &world.application;
    warm(application, "anchor-a");
    warm(application, "remote-a");
    let mut document = attach(application, "anchor-a");
    let sibling = attach(application, "remote-a");
    let untouched = with_request(application, |request| {
        sibling.read(request).unwrap().rows().to_vec()
    });
    let mut first = None;
    for (index, y) in [2, 5, 8].into_iter().enumerate() {
        let contacts = PlanarSourceAdjustment {
            scope_key: "anchor-a".into(),
            replacement_y: length(y),
        };
        reset_planar_source_adjustment_contacts(&contacts);
        let edit = to_selection(
            begin(
                document,
                application,
                "anchor-a",
                y,
                0x9182_d100 + index as u64,
            ),
            application,
        );
        let completed = edit
            .completed_source()
            .expect("actual original Fixed settlement");
        let source = completed.settlement().performed_source().unwrap();
        let description = source.receipt().descriptive_copy();
        assert_eq!(
            description.idempotency_binding(),
            source.receipt().idempotency_binding()
        );
        assert_eq!(
            description.committed_product_publication(),
            source.receipt().committed_product_publication()
        );
        assert!(
            source.root_fulfillment().matches_source(source.receipt()),
            "a descriptive copy does not consume the original source owner"
        );
        drop(description);
        assert_eq!(
            with_branch_request(application, completed.branch(), |request| request
                .query(PlanarRead {
                    body_key: "anchor-a".into()
                })
                .execute()
                .unwrap()
                .rows()[0]
                .y),
            length(y)
        );
        let action = source
            .receipt()
            .committed_product_publication()
            .composite_commit()
            .clone();
        let output = source.root_fulfillment().output_commit().clone();
        assert!(source.root_fulfillment().matches_source(source.receipt()));
        assert_ne!(
            action, output,
            "a real changed output publishes after this source action"
        );
        assert_eq!(
            source.root_fulfillment().posture(),
            WorthQueryOutputSettlementPosture::Performed
        );
        let finished = finish(edit, application);
        let WorthQueryFixedHistoryEditOutcome::Recorded { step, delivery } = finished.outcome
        else {
            panic!("this original edit must be recorded")
        };
        assert_eq!(step.endpoint().selected_commit(), &output);
        assert_eq!(
            delivery.settlement().root_observation().selected_commit(),
            &output
        );
        assert!(delivery.close_evidence().output_unavailable().is_none());
        if index == 0 {
            first = Some(step.clone());
        }
        document = finished.document;
        assert_eq!(
            with_request(application, |request| document
                .read(request)
                .unwrap()
                .rows()[0]
                .value),
            length(y + 1)
        );
        assert_eq!(
            with_request(application, |request| sibling
                .read(request)
                .unwrap()
                .rows()
                .to_vec()),
            untouched
        );
        assert_eq!(planar_source_adjustment_contacts(&contacts), (1, 1));
        drop(delivery);
    }
    let step = first.unwrap();
    let one = with_request(application, |request| {
        document.preview(request, step.id()).unwrap()
    });
    let two = with_request(application, |request| {
        document.preview(request, step.id()).unwrap()
    });
    let contacts = PlanarSourceAdjustment {
        scope_key: "anchor-a".into(),
        replacement_y: length(2),
    };
    let before = planar_source_adjustment_contacts(&contacts);
    with_request(application, |request| {
        let old = one.read(&document, request).unwrap();
        assert_eq!(old.source(), step.endpoint());
        assert_eq!(
            old.required_root_posture(),
            Some(WorthQueryOutputSettlementPosture::Performed)
        );
        assert_eq!(old.rows()[0].body_key, "anchor-a");
        assert_eq!(old.rows()[0].value, length(3));
        assert_eq!(document.read(request).unwrap().rows()[0].value, length(9));
        old.close();
    });
    one.close();
    with_request(application, |request| {
        let surviving = two.read(&document, request).unwrap();
        assert_eq!(surviving.rows()[0].value, length(3));
        assert_eq!(surviving.source(), step.endpoint());
        surviving.close();
    });
    two.close();
    assert_eq!(
        planar_source_adjustment_contacts(&contacts),
        before,
        "preview executes no original command"
    );
    let retried = finish(
        to_selection(
            begin(document, application, "anchor-a", 2, 0x9182_d100),
            application,
        ),
        application,
    );
    let WorthQueryFixedHistoryEditOutcome::AlreadyRecorded { step: original, .. } = retried.outcome
    else {
        panic!("same attempt returns its original step")
    };
    assert_eq!(original.id(), step.id());
    document = retried.document;
    assert_eq!(
        with_request(application, |request| document
            .read(request)
            .unwrap()
            .rows()[0]
            .value),
        length(9)
    );
    let drift = begin(document, application, "anchor-a", 7, 0x9182_d100);
    let drift=match with_request(application,|request|drift.advance(request)) {
        WorthQueryHistoryEditAdvance::Refused{edit,denial:WorthQueryHistoryEditDenial::History(
            WorthQueryApplicationHistoryDenial::Document(worth_query_host::facade::application_entry::WorthQueryDocumentHistoryDenial::IntentDrift))}=>edit,
        _=>panic!("changed input refuses at the original journal intent gate"),
    };
    document = with_request(application, |request| {
        drift
            .discard(request)
            .unwrap_or_else(|(_, cause)| panic!("initial drift close: {cause:?}"))
    });
    close(sibling, application);
    close(document, application);
}
