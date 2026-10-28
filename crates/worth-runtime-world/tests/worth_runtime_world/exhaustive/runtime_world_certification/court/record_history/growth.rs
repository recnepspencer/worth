// Literal retained-source growth under identical finite native page limits.
pub(crate) use super::*;
pub(crate) use worth_execution::{
    CancellationSource, ExecutionRequest, ExecutionWorkCeiling, SerialMemoryBudget, SerialRequest,
};
pub(crate) use worth_relational::facade::publication::RecordStructuralChange;

#[test]
fn ten_and_ten_thousand_unrelated_changes_keep_the_same_finite_page_work_and_storage() {
    let small = observe_growth(10);
    let large = observe_growth(10_000);
    assert_eq!(
        large, small,
        "page work and backing do not grow with retained source size"
    );
}

pub(crate) fn observe_growth(unrelated: usize) -> (u64, usize) {
    let setup = crate::test_execution_policy::serial_request();
    let setup_request = ExecutionRequest::serial(&setup);
    let court = NativeHistoryCourt::install_with_records(setup_request, unrelated + 1);
    assert_eq!(court.entities.len(), unrelated + 1);
    // Native's canonical record key orders target before structural rank.
    // This selects the final genuine Entity Updated fragment, not a commit order.
    let target = *court.entities.iter().max().unwrap();
    let original = court.basis();
    let subject = court
        .runtime
        .retain_record_history_subject(setup_request, &original, RecordRef::Entity(target))
        .unwrap();
    court.update_all(setup_request);
    let updated = court.basis();
    let observation = updated.observation();
    let selected_aspect = court.contracts[0].key();
    let mut cursor = court.cursor(
        setup_request,
        &updated,
        &[subject],
        std::slice::from_ref(selected_aspect),
    );
    // Explicit accounting lets the existing request report measure actual page
    // checkpoints. It imposes no guessed memory or work ceiling on the corpus.
    let measured = SerialRequest::from_memory(
        SerialMemoryBudget::new(None),
        CancellationSource::new().token(),
        None,
    );
    let request = ExecutionRequest::serial(&measured);
    assert!(request.has_accounting());
    let bounds = limits(1, 1);
    let mut maximum_work = 0;
    let mut maximum_backing = 0;
    let mut total_visits = 0;
    let mut matches = 0;
    for position in 0..=unrelated {
        let (page, report) = request
            .run(ExecutionWorkCeiling::new(None), |_| {
                cursor.next_page(&court.runtime, request, bounds)
            })
            .unwrap();
        let page = page.unwrap();
        assert!(report.charged_work() > 0);
        maximum_work = maximum_work.max(report.charged_work());
        maximum_backing = maximum_backing.max(page.owned_backing_bytes());
        assert_eq!(page.source_visits(), 1);
        assert!(page.entries().len() <= bounds.maximum_entries.get());
        assert!(page.owned_backing_bytes() <= bounds.maximum_owned_bytes.get());
        total_visits += page.source_visits();
        assert_eq!(page.is_complete(), position == unrelated);
        if position < unrelated {
            assert_eq!(
                page.entries().len(),
                0,
                "unrelated prefix remains incomplete"
            );
        } else {
            let event = page
                .entries()
                .next()
                .expect("the genuine last target is retained");
            matches += page.entries().len();
            assert_eq!(event.record(), &RecordRef::Entity(target));
            assert_eq!(event.commit_id(), observation.commit_id().unwrap());
            assert_eq!(event.version_id(), observation.version_id());
            assert_eq!(event.branch_id(), observation.identity().branch_id());
            assert_eq!(event.structural_change(), RecordStructuralChange::Updated);
            let mut aspects = event.changed_aspects();
            assert_eq!(aspects.next(), Some(selected_aspect));
            assert_eq!(aspects.next(), None);
        }
    }
    assert_eq!(total_visits, unrelated + 1);
    assert_eq!(matches, 1);
    (maximum_work, maximum_backing)
}
