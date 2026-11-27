//! `HistoryDocument` over installed-operation `Create` -> `Split` -> `Merge`
//! -> delete: every held name resolves to the member live at each navigated
//! node, through `undo_step`, `redo_to` and `checkout`.
use super::super::history::{navigate, probe, probe_unrecorded, record_edit, Navigate};
use super::*;
use probe::Observation;

type SculptAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    sculpt::Binding,
    Completion,
>;

fn retired(label: &str) -> Observation {
    Observation::Retired {
        label: label.to_owned(),
    }
}

#[test]
fn held_output_names_follow_split_merge_delete_undo_checkout_and_redo() {
    let token = "output-history";
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

    let document = sculpt!(
        document,
        Plan {
            creates: keys(&["h"]),
            steps: vec![Step::Birth("h".into(), birth("hist", "authored"))],
            ..Plan::default()
        },
    );
    let (document, observed) = probe(&host, document, &request, token, &["h"], 0);
    let h = named(&observed, "h");
    let document = sculpt!(
        document,
        Plan {
            reads: keys(&["h"]),
            deletes: keys(&["h"]),
            creates: keys(&["h1", "h2"]),
            steps: vec![Step::Split(
                read("h"),
                vec![
                    ("h1".into(), piece("cut", 1, None)),
                    ("h2".into(), piece("cut", 2, None))
                ],
            )],
            ..Plan::default()
        },
    );
    // The probe records its own node; the merge below is that node's child.
    let (document, observed) = probe(&host, document, &request, token, &["h1", "h2"], 1);
    let split = document.current_step(&request).unwrap();
    let (h1, h2) = (named(&observed, "h1"), named(&observed, "h2"));
    let mut pieces = vec![h1, h2];
    pieces.sort();
    let ambiguous = Observation::Ambiguous {
        label: "h".into(),
        candidates: pieces.clone(),
    };
    probe::hold(token, "h", h);

    let document = sculpt!(
        document,
        Plan {
            reads: keys(&["h1", "h2"]),
            deletes: keys(&["h2"]),
            steps: vec![Step::Merge(
                vec![read("h1"), read("h2")],
                Keeper::InPlace(read("h1"))
            )],
            ..Plan::default()
        },
    );
    let merged = document.current_step(&request).unwrap();
    probe::hold(token, "h2", h2);
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &["h1"], 2);
    assert_eq!(resolution(&observed, "h"), &resolved_to("h", "h1"));
    assert_eq!(resolution(&observed, "h2"), &resolved_to("h2", "h1"));

    let document = sculpt!(
        document,
        Plan {
            reads: keys(&["h1"]),
            deletes: keys(&["h1"]),
            ..Plan::default()
        },
    );
    let deleted = document.current_step(&request).unwrap();
    probe::hold(token, "h1", h1);
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &["h1"], 3);
    assert_eq!(resolution(&observed, "h1"), &retired("h1"));
    assert_eq!(resolution(&observed, "h"), &retired("h"));

    // `undo_step` restores the merged survivor under its exact name.
    navigate(&document, &request, Navigate::Undo, 0x5c00_0001);
    assert_eq!(document.current_step(&request).unwrap().id(), merged.id());
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &["h1"], 4);
    assert_eq!(named(&observed, "h1"), h1);
    assert_eq!(resolution(&observed, "h1"), &resolved_to("h1", "h1"));
    assert_eq!(resolution(&observed, "h2"), &resolved_to("h2", "h1"));

    // `redo_to` the deletion retires the survivor again.
    navigate(
        &document,
        &request,
        Navigate::Redo(deleted.id()),
        0x5c00_0002,
    );
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &["h1"], 5);
    assert_eq!(resolution(&observed, "h1"), &retired("h1"));
    assert_eq!(resolution(&observed, "h"), &retired("h"));

    // `checkout` of the split node: both pieces live under their exact names,
    // and the parent's held name is ambiguous between them.
    navigate(
        &document,
        &request,
        Navigate::Checkout(split.id()),
        0x5c00_0003,
    );
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &["h1", "h2"], 6);
    assert_eq!(named(&observed, "h1"), h1);
    assert_eq!(
        named(&observed, "h2"),
        h2,
        "the merged-away piece restores its name"
    );
    assert_eq!(resolution(&observed, "h"), &ambiguous);
    assert_eq!(resolution(&observed, "h1"), &resolved_to("h1", "h1"));
    assert_eq!(resolution(&observed, "h2"), &resolved_to("h2", "h2"));
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
