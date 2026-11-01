//! Same two-child prefix with unrelated descendants; classify and page only that parent.
use super::*;
use std::num::NonZeroUsize;
use worth_query_host::facade::application_entry::{WorthQueryHistoryPageLimits, WorthQueryHistoryRedoChoice};
use worth_query_testkit_document_retention::facade::document_retention_model::document_history::navigation::Navigation;

fn checkout(
    host: &App,
    document: &Document<'_>,
    node: worth_query_host::facade::application_entry::WorthQueryHistoryNodeId,
    key: u128,
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut owner = document
        .navigate::<Navigation>()
        .checkout(node)
        .expected_selection(document.selection(&request).unwrap())
        .idempotency(key)
        .prepare(&request)
        .unwrap();
    for _ in 0..16 {
        owner = match owner.advance(&request) {
            worth_query_host::facade::application_entry::WorthQueryHistoryNavigationAdvance::Pending(owner) => owner,
            worth_query_host::facade::application_entry::WorthQueryHistoryNavigationAdvance::Selected(_) => return,
            worth_query_host::facade::application_entry::WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => panic!("lawful setup checkout: {cause:?}"),
        };
    }
    panic!("finite setup checkout")
}

#[test]
fn redo_classification_and_child_page_visits_stay_local_with_unrelated_futures() {
    let host = publish_on_first_program();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut samples = Vec::new();
    for count in [1, 15] {
        let key_base = 0x9182_f300 + (count as u64) * 0x100;
        let document = attach(&host, head(&host), DOCUMENT_IDENTITY);
        let root = document.current_step(&request).unwrap();
        let (document, first) = finish(
            &host,
            ready(
                &host,
                begin(&host, document, DOCUMENT_IDENTITY, 3, key_base),
            ),
            3,
        );
        checkout(&host, &document, root.id(), u128::from(key_base + 1));
        let (mut document, second) = finish(
            &host,
            ready(
                &host,
                begin(&host, document, DOCUMENT_IDENTITY, 5, key_base + 2),
            ),
            5,
        );
        for offset in 0..count {
            document = finish(
                &host,
                ready(
                    &host,
                    begin(
                        &host,
                        document,
                        DOCUMENT_IDENTITY,
                        3 + (offset % 2) as u64,
                        key_base + 0x10 + offset as u64,
                    ),
                ),
                3 + (offset % 2) as u64,
            )
            .0;
        }
        checkout(&host, &document, root.id(), u128::from(key_base + 3));
        let observed = document.observe_history_enrollment_for_test();
        let limits = WorthQueryHistoryPageLimits {
            maximum_entries: NonZeroUsize::MIN,
            maximum_examined_entries: NonZeroUsize::MIN,
            maximum_owned_bytes: NonZeroUsize::new(16 * 1024).unwrap(),
        };
        let page = document.redo_choices(&request, limits, None).unwrap();
        assert_eq!(page.choice(), WorthQueryHistoryRedoChoice::ChoiceRequired);
        assert_eq!(page.children().rows()[0].step(), &first);
        assert_eq!(page.children().examined_entries(), 1);
        let next = document
            .redo_choices(&request, limits, page.children().continuation().as_ref())
            .unwrap();
        assert_eq!(next.choice(), WorthQueryHistoryRedoChoice::ChoiceRequired);
        assert_eq!(next.children().rows()[0].step(), &second);
        assert_eq!(next.children().examined_entries(), 1);
        assert!(
            next.children().continuation().is_none(),
            "unrelated later ordinals cannot invent another child page"
        );
        let visits = observed.snapshot();
        assert!(visits.node_probes > 0 && visits.child_index_probes > 0);
        samples.push((
            visits.node_probes,
            visits.child_index_probes,
            page.children().admitted_backing_bytes(),
            next.children().admitted_backing_bytes(),
        ));
        drop(observed);
        close(&host, document, count + 5);
    }
    assert_eq!(
        samples[0], samples[1],
        "fixed classification and chosen child pages must not inspect unrelated retained steps"
    );
}
