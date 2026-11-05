//! Public checkout/redo uses the same Native transfer boundary as ordinary edits.
use super::*;
#[test]
fn ordinary_recorded_transfer_is_reversible_on_the_same_public_history_tree() {
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let original = document.current_step(&request).unwrap();
    let identity = world::member_identity(
        &host,
        document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch(),
        "direct",
    );
    type Action = ApplicationActionReference<
        Schema,
        ApplicationRootComposition,
        Entry,
        ownership_change::Binding,
        Completion,
    >;
    let mut edit = document
        .edit(ownership_change::Change {
            from: "a".into(),
            to: "b".into(),
            member: "direct".into(),
            preserve_previous: false,
        })
        .idempotency_owned(&request, "public-transfer-before-navigation".into())
        .unwrap()
        .retain_in_program::<Program, Action>(&host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(&request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => {
                edit = next;
                break;
            }
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    }
    let WorthQueryHistoryEditFinish { document, outcome } = edit
        .finish(&request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    assert!(matches!(
        &outcome,
        WorthQueryHistoryEditOutcome::Recorded { .. }
    ));
    let transferred = document.current_step(&request).unwrap();
    drop(outcome);
    for (index, target) in [original.id(), transferred.id(), original.id()]
        .into_iter()
        .enumerate()
    {
        let prepare = document.navigate::<Navigation>();
        let prepare = if index == 1 {
            prepare.redo_to(target)
        } else {
            prepare.checkout(target)
        };
        let mut navigation = prepare
            .expected_selection(document.selection(&request).unwrap())
            .idempotency(0x1188_0000 + index as u128)
            .prepare(&request)
            .unwrap();
        let mut selected = false;
        for _ in 0..16 {
            navigation = match navigation.advance(&request) {
                WorthQueryHistoryNavigationAdvance::Pending(next) => next,
                WorthQueryHistoryNavigationAdvance::Selected(completion) => {
                    assert_eq!(completion.selection().step().id(), target);
                    selected = true;
                    break;
                }
                WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => panic!("{cause:?}"),
            };
        }
        assert!(selected);
        let branch = document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        assert_eq!(world::member_identity(&host, branch, "direct"), identity);
        assert_eq!(world::member_value(&host, branch, "direct"), 11);
        assert_eq!(world::member_value(&host, branch, "nested"), 22);
        assert_eq!(world::member_value(&host, branch, "foreign"), 33);
        assert_eq!(
            host.request(&principal, &scope)
                .on_branch(branch)
                .query(query::model::Read("b".into()))
                .execute()
                .unwrap()
                .rows()[0]
                .value,
            1
        );
    }
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
