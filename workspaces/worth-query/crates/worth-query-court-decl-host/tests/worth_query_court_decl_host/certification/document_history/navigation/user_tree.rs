//! Generated public journeys are compared with an independently owned user-step tree.
use super::*;
mod model;
use model::{Command, UserTree};
use worth_query_host::facade::application_entry::{
    WorthQueryHistoryRedoChoice, WorthQueryHistoryStep,
};
use worth_query_testkit_document_retention::facade::document_retention_model::retention_entry::SetRetentionHandler;

fn limits(width: usize) -> WorthQueryHistoryPageLimits {
    WorthQueryHistoryPageLimits {
        maximum_entries: NonZeroUsize::new(width).unwrap(),
        maximum_examined_entries: NonZeroUsize::new(width).unwrap(),
        maximum_owned_bytes: NonZeroUsize::new(32 * 1024).unwrap(),
    }
}

fn verify(
    host: &Runtime,
    document: &Document<'_>,
    tree: &UserTree,
    issued: &[WorthQueryHistoryStep],
) {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    assert_eq!(value(host, document), tree.nodes[tree.selected].value);
    let selected = document.selection(&request).unwrap();
    assert_eq!(selected.step().id(), issued[tree.selected].id());
    assert_eq!(
        selected.step().parent(),
        tree.nodes[tree.selected]
            .parent
            .map(|parent| issued[parent].id())
    );
    let children = tree.children();
    let expected_choice = match children.as_slice() {
        [] => WorthQueryHistoryRedoChoice::NoChild,
        [child] => WorthQueryHistoryRedoChoice::Single(issued[*child].id()),
        _ => WorthQueryHistoryRedoChoice::ChoiceRequired,
    };
    let mut after = None;
    let mut actual = Vec::new();
    loop {
        let choices = document
            .redo_choices(&request, limits(1), after.as_ref())
            .unwrap();
        assert_eq!(choices.choice(), expected_choice);
        assert_eq!(
            choices.children().current_step_id(),
            issued[tree.selected].id()
        );
        actual.extend(
            choices
                .children()
                .rows()
                .iter()
                .map(|row| row.step().clone()),
        );
        after = choices.children().continuation();
        if after.is_none() {
            break;
        }
        assert!(
            actual.len() < children.len(),
            "no empty or repeated continuation"
        );
    }
    assert_eq!(
        actual,
        children
            .iter()
            .map(|index| issued[*index].clone())
            .collect::<Vec<_>>()
    );
}

#[test]
fn generated_edits_and_navigation_match_an_independent_tree_without_reentering_the_source_handler()
{
    let (host, contacts) =
        SetRetentionHandler::observe_installation_for_test(publish_on_first_program);
    let mut document = attach(&host, head(&host), DOCUMENT_IDENTITY);
    let sibling = attach(&host, head(&host), RELATED_DOCUMENT_IDENTITY);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut tree = UserTree::new(SEED_RETENTION);
    let mut issued = vec![document.current_step(&request).unwrap()];
    verify(&host, &document, &tree, &issued);
    // Three siblings force choice-required and page tails beyond the first2 probe.
    let prefix = [
        Command::Edit(3),
        Command::Undo,
        Command::Edit(5),
        Command::Undo,
        Command::Edit(4),
        Command::Undo,
        Command::Redo(1),
        Command::Checkout(0),
    ];
    let mut seed = 0x6c62_272e_07bb_0142_u64;
    let mut heads = 1;
    for index in 0..24 {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let command = if index < prefix.len() {
            prefix[index]
        } else {
            tree.generated(index - prefix.len(), seed)
        };
        let before_calls = contacts.snapshot();
        let before = document.selection(&request).unwrap();
        let parent = tree.selected;
        // Expected topology/value moves FIRST, without reading any SDK result.
        tree.apply(command);
        match command {
            Command::Edit(days) => {
                let (next, step) = finish(
                    &host,
                    ready(
                        &host,
                        begin(
                            &host,
                            document,
                            DOCUMENT_IDENTITY,
                            days,
                            0x9182_f400 + index as u64,
                        ),
                    ),
                    days,
                );
                document = next;
                assert_eq!(step.parent(), Some(issued[parent].id()));
                assert!(issued.iter().all(|old| old.id() != step.id()));
                issued.push(step);
                let after = contacts.snapshot();
                assert!(
                    after.0 > before_calls.0 && after.1 > before_calls.1,
                    "real ordinary edit must exercise both observed receiver stages"
                );
            }
            _ => {
                let builder = document.navigate::<Navigation>();
                let target = match command {
                    Command::Undo => builder.undo_step(),
                    Command::Redo(child) => builder.redo_to(issued[child].id()),
                    Command::Checkout(node) => builder.checkout(issued[node].id()),
                    Command::Edit(_) => unreachable!(),
                };
                let done = complete(
                    &host,
                    target
                        .expected_selection(before.clone())
                        .idempotency(0x9182_f400 + index as u128)
                        .prepare(&request)
                        .unwrap(),
                );
                assert_eq!(done.requested_node(), issued[tree.selected].id());
                assert_eq!(contacts.snapshot(), before_calls, "{command:?} must not call the ordinary retention receiver or its declared inverse target");
            }
        }
        heads += 1;
        assert_ne!(
            document.selection(&request).unwrap().physical_endpoint(),
            before.physical_endpoint()
        );
        verify(&host, &document, &tree, &issued);
        if !matches!(command, Command::Edit(_)) {
            assert_eq!(
                contacts.snapshot(),
                before_calls,
                "navigation and its metadata readback must not reenter the observed source handler"
            );
        }
    }
    let mut after = None;
    let mut all = Vec::new();
    loop {
        let page = document
            .history_page(&request, limits(3), after.as_ref())
            .unwrap();
        all.extend(page.rows().iter().map(|row| row.step().clone()));
        after = page.continuation();
        if after.is_none() {
            break;
        }
    }
    assert_eq!(
        all, issued,
        "only authored edits add nodes; navigation preserves every alternative"
    );
    for (index, step) in issued.iter().enumerate() {
        assert_eq!(
            step.parent(),
            tree.nodes[index].parent.map(|parent| issued[parent].id())
        );
        let preview = document.preview(&request, step.id()).unwrap();
        let read = preview.read(&document, &request).unwrap();
        assert_eq!(read.rows()[0].retention_days, tree.nodes[index].value);
        assert_eq!(read.source(), step.endpoint());
        read.close();
        preview.close();
    }
    assert_eq!(value(&host, &sibling), SEED_RETENTION);
    close(&host, document, heads);
    close(&host, sibling, 1);
}
