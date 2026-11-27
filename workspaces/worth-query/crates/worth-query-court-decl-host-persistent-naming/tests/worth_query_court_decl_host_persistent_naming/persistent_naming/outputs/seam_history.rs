//! Slice 5D: `HistoryDocument` over installed-operation seam `Create` ->
//! `Split` -> `Merge` -> `Retire`. Every held seam name resolves to the seam
//! live at each navigated node, through `undo_step`, `redo_to` and
//! `checkout`.
use super::super::history::{navigate, probe, probe_unrecorded, record_edit, Navigate};
use super::seam_names::{ends, link, seam, seam_read, MEMBERS};
use super::*;

type SculptAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    sculpt::Binding,
    Completion,
>;

fn retired(label: &str) -> SeamObservation {
    SeamObservation::Retired {
        label: label.to_owned(),
    }
}

#[test]
fn held_seam_names_follow_split_merge_retire_undo_redo_and_checkout() {
    let token = "seam-history";
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
    let mut probes = 0;
    let mut held: Vec<(&str, WorthQueryName<Seam>)> = Vec::new();
    // A recorded probe naming the seams `named` and resolving every held one.
    macro_rules! seams {
        ($document:expr, $probe:ident, $named:expr) => {{
            probes += 1;
            let run = format!("{token}-probe-{probes}");
            for (label, from, to) in $named {
                probe::seams::name(&run, label, from, to);
            }
            for (label, name) in &held {
                probe::seams::hold(&run, label, *name, exact());
            }
            let probed = $probe(&host, $document, &request, &run, &MEMBERS, probes);
            (probed.0, probe::seams::take(&run))
        }};
    }
    let no_seams: [(&str, &str, &str); 0] = [];

    let document = sculpt!(
        document,
        Plan {
            creates: keys(&MEMBERS),
            steps: MEMBERS
                .iter()
                .map(|key| Step::Birth(
                    (*key).to_owned(),
                    birth(&format!("hist-{key}"), "authored")
                ))
                .collect(),
            ..Plan::default()
        },
    );
    let document = sculpt!(
        document,
        link("s", "m1", "m2", seam("hist-weld", "seam", 1))
    );
    let (document, observed) = seams!(document, probe, [("s", "m1", "m2")]);
    let s = seam_named(&observed, "s");
    held.push(("s", s));

    let document = sculpt!(
        document,
        Plan {
            reads: keys(&MEMBERS),
            seam_reads: vec![seam_read("s", "m1", "m2")],
            unlinks: vec![("m1".into(), "m2".into())],
            seams: vec![
                ("p1".into(), "m1".into(), "m3".into()),
                ("p2".into(), "m1".into(), "m4".into()),
            ],
            steps: vec![Step::SeamSplit(
                HeldSeam::Read("s".into()),
                vec![
                    ("p1".into(), seam("hist-cut", "seam_piece", 1)),
                    ("p2".into(), seam("hist-cut", "seam_piece", 2)),
                ],
            )],
            ..Plan::default()
        },
    );
    let split = document.current_step(&request).unwrap();
    let (document, observed) = seams!(
        document,
        probe_unrecorded,
        [("p1", "m1", "m3"), ("p2", "m1", "m4")]
    );
    let (p1, p2) = (seam_named(&observed, "p1"), seam_named(&observed, "p2"));
    let mut pieces = vec![p1, p2];
    pieces.sort();
    let ambiguous = SeamObservation::Ambiguous {
        label: "s".into(),
        candidates: pieces,
    };
    assert_eq!(seam_resolution(&observed, "s"), &ambiguous);
    held.extend([("p1", p1), ("p2", p2)]);

    let document = sculpt!(
        document,
        Plan {
            reads: keys(&["m1", "m3", "m4"]),
            seam_reads: vec![seam_read("p1", "m1", "m3"), seam_read("p2", "m1", "m4")],
            unlinks: vec![("m1".into(), "m4".into())],
            steps: vec![Step::SeamMerge(
                vec![HeldSeam::Read("p1".into()), HeldSeam::Read("p2".into())],
                SeamKeeper::InPlace(HeldSeam::Read("p1".into())),
            )],
            ..Plan::default()
        },
    );
    let merged = document.current_step(&request).unwrap();
    let survivor = |label: &str| seam_between(label, "m1", "m3");
    let (document, observed) = seams!(document, probe_unrecorded, no_seams);
    for label in ["s", "p1", "p2"] {
        assert_eq!(seam_resolution(&observed, label), &survivor(label));
    }

    let document = sculpt!(
        document,
        Plan {
            reads: keys(&["m1", "m3"]),
            seam_reads: vec![seam_read("p1", "m1", "m3")],
            unlinks: vec![("m1".into(), "m3".into())],
            steps: vec![Step::SeamRetire(HeldSeam::Read("p1".into()))],
            ..Plan::default()
        },
    );
    let retirement = document.current_step(&request).unwrap();
    let (document, observed) = seams!(document, probe_unrecorded, no_seams);
    for label in ["s", "p1", "p2"] {
        assert_eq!(seam_resolution(&observed, label), &retired(label));
    }

    // `undo_step` restores the merged survivor under its exact name.
    navigate(&document, &request, Navigate::Undo, 0x5d00_0001);
    assert_eq!(document.current_step(&request).unwrap().id(), merged.id());
    let (document, observed) = seams!(document, probe_unrecorded, [("p1", "m1", "m3")]);
    assert_eq!(seam_named(&observed, "p1"), p1);
    for label in ["s", "p1", "p2"] {
        assert_eq!(seam_resolution(&observed, label), &survivor(label));
    }

    // `redo_to` the retirement ends every name again.
    navigate(
        &document,
        &request,
        Navigate::Redo(retirement.id()),
        0x5d00_0002,
    );
    let (document, observed) = seams!(document, probe_unrecorded, no_seams);
    for label in ["s", "p1", "p2"] {
        assert_eq!(seam_resolution(&observed, label), &retired(label));
    }

    // `checkout` of the split node: both pieces live under their exact
    // names, and the parent's held name is ambiguous between them.
    navigate(
        &document,
        &request,
        Navigate::Checkout(split.id()),
        0x5d00_0003,
    );
    let (document, observed) = seams!(
        document,
        probe_unrecorded,
        [("p1", "m1", "m3"), ("p2", "m1", "m4")]
    );
    assert_eq!(seam_named(&observed, "p1"), p1);
    assert_eq!(
        seam_named(&observed, "p2"),
        p2,
        "the merged-away seam restores its name"
    );
    assert_eq!(seam_resolution(&observed, "s"), &ambiguous);
    assert_eq!(
        seam_resolution(&observed, "p1"),
        &SeamObservation::ResolvedTo {
            label: "p1".into(),
            ends: ends("m1", "m3"),
        }
    );
    assert_eq!(
        seam_resolution(&observed, "p2"),
        &SeamObservation::ResolvedTo {
            label: "p2".into(),
            ends: ends("m1", "m4"),
        }
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
