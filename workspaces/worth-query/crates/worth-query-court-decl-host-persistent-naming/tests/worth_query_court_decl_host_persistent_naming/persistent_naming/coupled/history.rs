//! `HistoryDocument` records each edit envelope as one step: held names its
//! stages minted, split, merged and retired follow `undo_step`, `redo_to` and
//! `checkout` exactly as for single-operation steps.
use super::super::history::{navigate, probe, probe_unrecorded, Navigate};
use super::sculpt::Held;
use super::*;
use probe::Observation;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

type Document<'a> = world::Document<'a>;

/// Records `stages` as one envelope step of `document`.
fn record<'a>(
    host: &'a world::Host,
    document: Document<'a>,
    request: &super::super::history::Request<'_, '_, '_>,
    key: &str,
    stages: Vec<Stage>,
) -> Document<'a> {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let branch = document
        .selection(request)
        .unwrap()
        .physical_endpoint()
        .product_branch();
    let request = host.request(&principal, &scope).on_branch(branch);
    let mut envelope = document
        .edit_envelope(host, &request, key.to_owned())
        .unwrap_or_else(|failure| panic!("{failure:?}"))
        .ordered_authored()
        .unwrap_or_else(|(_, denial)| panic!("{denial:?}"));
    for Stage { token, plan } in stages {
        sculpt::author(&token, plan);
        envelope
            .push_stage::<ApplicationNoOutputGraph, _>(sculpt::Intent { token })
            .unwrap();
    }
    match envelope.execute_recorded(
        WorthQueryHistoryStepMetadata::from_bytes(key.as_bytes().to_vec()),
        ExecutionAllocationPolicy::SystemAllocation,
    ) {
        Ok(WorthQueryHistoryEditEnvelopeOutcome::Recorded { document, .. }) => document,
        Ok(other) => panic!("{key} must record: {other:?}"),
        Err(failure) => panic!("{key} must record: {failure:?}"),
    }
}

fn resolved(label: &str, key: &str) -> Observation {
    outputs::resolved_to(label, key)
}

fn retired(label: &str) -> Observation {
    Observation::Retired {
        label: label.to_owned(),
    }
}

#[test]
fn envelope_named_outputs_follow_split_merge_retire_undo_checkout_and_redo() {
    let token = "coupled-history";
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let named_plan = |key: &str| Plan {
        creates: keys(&[key]),
        steps: vec![Step::Birth(
            key.into(),
            birth(&format!("{token}-{key}"), "authored"),
        )],
        ..Plan::default()
    };
    // One envelope: each stage births one member.
    let document = record(
        &host,
        document,
        &request,
        "coupled-history-born",
        vec![
            stage("coupled-history-born-1", named_plan("h")),
            stage("coupled-history-born-2", named_plan("k")),
        ],
    );
    let born = document.current_step(&request).unwrap();
    let (document, observed) = probe(&host, document, &request, token, &["h", "k"], 0);
    let (h, k) = (named(&observed, "h"), named(&observed, "k"));

    // One envelope: the first stage splits `h`, the second retires `k`.
    let document = record(
        &host,
        document,
        &request,
        "coupled-history-split",
        vec![
            stage(
                "coupled-history-split-1",
                Plan {
                    plain_reads: keys(&["h"]),
                    deletes: keys(&["h"]),
                    creates: keys(&["h1", "h2"]),
                    steps: vec![Step::Split(
                        Held::Name(h),
                        vec![
                            ("h1".into(), piece("coupled-history-cut", 1, None)),
                            ("h2".into(), piece("coupled-history-cut", 2, None)),
                        ],
                    )],
                    ..Plan::default()
                },
            ),
            stage(
                "coupled-history-split-2",
                Plan {
                    plain_reads: keys(&["k"]),
                    deletes: keys(&["k"]),
                    steps: vec![Step::Retire(Held::Name(k))],
                    ..Plan::default()
                },
            ),
        ],
    );
    // The probe records its own node; the merge below is that node's child.
    let (document, observed) = probe(&host, document, &request, token, &["h1", "h2"], 1);
    let split = document.current_step(&request).unwrap();
    let (h1, h2) = (named(&observed, "h1"), named(&observed, "h2"));
    let (kept, merged_away, kept_name, merged_name) = if h1 < h2 {
        ("h1", "h2", h1, h2)
    } else {
        ("h2", "h1", h2, h1)
    };

    // One envelope merging the pieces back, least keeper in place.
    let document = record(
        &host,
        document,
        &request,
        "coupled-history-merge",
        vec![stage(
            "coupled-history-merge-1",
            Plan {
                plain_reads: keys(&["h1", "h2"]),
                deletes: keys(&[merged_away]),
                steps: vec![scoped(
                    "coupled-history-merge",
                    vec![ScopeStep::MergeLeast(
                        vec![Held::Name(h2), Held::Name(h1)],
                        None,
                    )],
                )],
                ..Plan::default()
            },
        )],
    );
    probe::hold(token, "h", h);
    probe::hold(token, "k", k);
    probe::hold(token, merged_away, merged_name);
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &[kept], 2);
    assert_eq!(outputs::resolution(&observed, "h"), &resolved("h", kept));
    assert_eq!(
        outputs::resolution(&observed, merged_away),
        &resolved(merged_away, kept)
    );
    assert_eq!(outputs::resolution(&observed, "k"), &retired("k"));
    let merged = document.current_step(&request).unwrap();

    // `undo_step` returns to the split: both pieces live under their names.
    navigate(&document, &request, Navigate::Undo, 0x3b20_0001);
    assert_eq!(document.current_step(&request).unwrap().id(), split.id());
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &["h1", "h2"], 3);
    assert_eq!(named(&observed, kept), kept_name);
    assert_eq!(named(&observed, merged_away), merged_name);
    assert_eq!(
        outputs::resolution(&observed, merged_away),
        &resolved(merged_away, merged_away)
    );
    assert!(matches!(
        outputs::resolution(&observed, "h"),
        Observation::Ambiguous { .. }
    ));

    // `redo_to` the merge keeps the least piece under its exact name; like
    // any navigated step, it retires the merged-away piece's record (Native
    // appends `Retire` for the records the selection retires).
    navigate(
        &document,
        &request,
        Navigate::Redo(merged.id()),
        0x3b20_0002,
    );
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &[kept], 4);
    assert_eq!(named(&observed, kept), kept_name);
    assert_eq!(outputs::resolution(&observed, "h"), &resolved("h", kept));
    assert_eq!(
        outputs::resolution(&observed, merged_away),
        &retired(merged_away)
    );

    // `checkout` of the first envelope restores both births exactly.
    navigate(
        &document,
        &request,
        Navigate::Checkout(born.id()),
        0x3b20_0003,
    );
    let (document, observed) = probe_unrecorded(&host, document, &request, token, &["h", "k"], 5);
    assert_eq!((named(&observed, "h"), named(&observed, "k")), (h, k));
    assert_eq!(outputs::resolution(&observed, "h"), &resolved("h", "h"));
    assert_eq!(outputs::resolution(&observed, "k"), &resolved("k", "k"));
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
