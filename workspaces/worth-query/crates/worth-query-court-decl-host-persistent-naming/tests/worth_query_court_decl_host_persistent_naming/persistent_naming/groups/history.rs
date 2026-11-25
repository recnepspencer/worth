//! `HistoryDocument` navigation restores group names exactly: `undo_step`
//! returns a split-off member to the loop it left under that loop's exact
//! name, and `redo_to` / `checkout` bring the piece back under its own.
use super::super::history::{navigate, record_edit, Navigate};
use super::super::named_read::{self, GroupedMember};
use super::super::outputs::sculpt;
use super::*;
use GroupRole::{Host, Member as Plain};

type SculptAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    sculpt::Binding,
    Completion,
>;

#[test]
fn group_names_follow_split_undo_redo_and_checkout() {
    let token = "group-history";
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let mut edits = 0;
    macro_rules! sculpt {
        ($document:expr, $plan:expr $(,)?) => {{
            edits += 1;
            let step = format!("{token}-{edits}");
            sculpt::author(&step, $plan);
            let intent = sculpt::Intent {
                token: step.clone(),
            };
            record_edit!(&host, $document, &request, intent, SculptAction, step)
        }};
    }
    // The row each member projects on the document's selected branch.
    let rows = |document: &world::Document<'_>| -> Vec<GroupedMember> {
        let branch = document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch();
        ["h", "x", "y"]
            .iter()
            .flat_map(|key| {
                host.request(&principal, &scope)
                    .on_branch(branch.clone())
                    .query(named_read::grouped_member::Read((*key).into()))
                    .execute()
                    .unwrap_or_else(|cause| panic!("{key}: {cause:?}"))
                    .rows()
                    .to_vec()
            })
            .collect()
    };

    // One edit creates three members and groups them by their same-commit
    // names: the loop is born in the commit that names its members.
    let created = |key: &str| GroupMember::Created(key.to_owned());
    let document = sculpt!(
        document,
        Plan {
            creates: keys(&["h", "x", "y"]),
            steps: vec![
                Step::Birth("h".into(), birth("hist-h", "authored")),
                Step::Birth("x".into(), birth("hist-x", "authored")),
                Step::Birth("y".into(), birth("hist-y", "authored")),
                Step::Groups(loops(
                    vec![vec![
                        (Host, created("h")),
                        (Plain, created("x")),
                        (Plain, created("y"))
                    ]],
                    &[],
                )),
            ],
            ..Plan::default()
        },
    );
    let grouped = document.current_step(&request).unwrap();
    let before = rows(&document);
    let whole = before[0]
        .in_loop
        .expect("the loop is born with its members");
    assert!(
        before.iter().all(|row| row.in_loop == Some(whole)),
        "{before:?}"
    );
    let y = before[2].name;

    // `y` leaves as its own loop.
    let document = sculpt!(
        document,
        Plan {
            steps: vec![Step::Groups(loops(vec![group(&[(Host, y)])], &[]))],
            ..Plan::default()
        }
    );
    let split = document.current_step(&request).unwrap();
    let after = rows(&document);
    let piece = after[2].in_loop.expect("y is its own loop");
    assert_ne!(piece, whole);
    assert_eq!(after[1].in_loop, Some(whole));

    // Undo returns y to the whole loop under its exact name.
    navigate(&document, &request, Navigate::Undo, 0x6c00_0001);
    assert_eq!(document.current_step(&request).unwrap().id(), grouped.id());
    assert_eq!(rows(&document), before);

    // Redo brings the piece back under its exact name.
    navigate(&document, &request, Navigate::Redo(split.id()), 0x6c00_0002);
    assert_eq!(rows(&document), after);

    // Checkout of the grouped node again restores the whole loop.
    navigate(
        &document,
        &request,
        Navigate::Checkout(grouped.id()),
        0x6c00_0003,
    );
    assert_eq!(rows(&document), before);
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
