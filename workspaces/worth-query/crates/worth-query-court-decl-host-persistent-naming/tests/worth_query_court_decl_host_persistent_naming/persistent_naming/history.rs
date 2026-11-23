//! `HistoryDocument` navigation keeps every held name meaning the same member:
//! deletion retires it, `undo_step`/`checkout` restores it at its exact
//! original identity, `redo_to` retires it again, and a branch edit after the
//! undo leaves it resolving on the live branch.
use super::*;
use probe::Observation;
use worth_query_decl::facade::application_naming::WorthQueryName;

pub(super) type ProbeAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    probe::Binding,
    Completion,
>;
pub(super) type DeleteAction = ApplicationActionReference<
    Schema,
    ApplicationRootComposition,
    Entry,
    delete::Binding,
    Completion,
>;
pub(super) type Request<'a, 'b, 'c> = WorthQueryApplicationRequest<'a, 'b, 'c, Schema>;

/// Records one authored edit on `document` and returns the advanced document.
macro_rules! record_edit {
    ($host:expr, $document:expr, $request:expr, $intent:expr, $action:ty, $key:expr) => {{
        let mut edit = $document
            .edit($intent)
            .idempotency_owned($request, $key.to_owned())
            .unwrap()
            .retain_in_program::<Program, $action>($host)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        loop {
            edit = match edit.advance($request) {
                WorthQueryHistoryEditAdvance::Progress(next) => next,
                WorthQueryHistoryEditAdvance::Ready(next) => {
                    edit = next;
                    break;
                }
                WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
            };
        }
        let WorthQueryHistoryEditFinish { document, outcome } = edit
            .finish($request)
            .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
        if let WorthQueryHistoryEditOutcome::NotRecorded { terminal, .. } = &outcome {
            panic!(
                "not recorded: {:?} {:?}",
                terminal.native_outcome(),
                terminal.native_admission_denial()
            );
        }
        drop(outcome);
        document
    }};
}

/// One navigation command against the document's current selection.
#[derive(Clone, Copy, Debug)]
pub(super) enum Navigate {
    Undo,
    Redo(WorthQueryHistoryNodeId),
    Checkout(WorthQueryHistoryNodeId),
}

pub(super) fn navigate(
    document: &world::Document<'_>,
    request: &Request<'_, '_, '_>,
    command: Navigate,
    key: u128,
) {
    let builder = document.navigate::<navigation::Navigation>();
    let prepare = match command {
        Navigate::Undo => builder.undo_step(),
        Navigate::Redo(step) => builder.redo_to(step),
        Navigate::Checkout(step) => builder.checkout(step),
    };
    let mut navigation = prepare
        .expected_selection(document.selection(request).unwrap())
        .idempotency(key)
        .prepare(request)
        .unwrap_or_else(|cause| panic!("{command:?}: {cause:?}"));
    for _ in 0..32 {
        navigation = match navigation.advance(request) {
            WorthQueryHistoryNavigationAdvance::Pending(next) => next,
            WorthQueryHistoryNavigationAdvance::Selected(_) => return,
            WorthQueryHistoryNavigationAdvance::Refused { cause, .. } => {
                panic!("{command:?}: {cause:?}")
            }
        };
    }
    panic!("bounded navigation {command:?} must select");
}

/// Runs a recorded probe over `keys` and returns what it observed.
pub(super) fn probe<'a>(
    host: &'a world::Host,
    document: world::Document<'a>,
    request: &Request<'_, '_, '_>,
    token: &str,
    keys: &[&str],
    ordinal: u64,
) -> (world::Document<'a>, Vec<Observation>) {
    let intent = probe::Intent {
        token: token.to_owned(),
        keys: keys.iter().map(|key| (*key).to_owned()).collect(),
        value: 100 + ordinal,
    };
    let document = record_edit!(
        host,
        document,
        request,
        intent,
        ProbeAction,
        format!("{token}:probe:{ordinal}")
    );
    (document, probe::take(token))
}

/// Runs a probe whose decision rests on an ended name: the decision still
/// observes, but the projection refuses to seal (an ended name has no
/// decision fact key), so the edit is discarded and history does not move.
pub(super) fn probe_refused<'a>(
    host: &'a world::Host,
    document: world::Document<'a>,
    request: &Request<'_, '_, '_>,
    token: &str,
    keys: &[&str],
    ordinal: u64,
) -> (world::Document<'a>, Vec<Observation>, String) {
    let intent = probe::Intent {
        token: token.to_owned(),
        keys: keys.iter().map(|key| (*key).to_owned()).collect(),
        value: 100 + ordinal,
    };
    let mut edit = document
        .edit(intent)
        .idempotency_owned(request, format!("{token}:refused-probe:{ordinal}"))
        .unwrap()
        .retain_in_program::<Program, ProbeAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    loop {
        edit = match edit.advance(request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(_) => {
                panic!("a decision resting on an ended name must not seal")
            }
            WorthQueryHistoryEditAdvance::Refused { edit, denial } => {
                let denial = format!("{denial:?}");
                let document = edit
                    .discard(request)
                    .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
                return (document, probe::take(token), denial);
            }
        };
    }
}

pub(super) fn named(observed: &[Observation], key: &str) -> WorthQueryName<Member> {
    observed
        .iter()
        .find_map(|observation| match observation {
            Observation::Named { key: named, name } if named == key => Some(*name),
            _ => None,
        })
        .unwrap_or_else(|| panic!("`{key}` must carry its seeded name: {observed:?}"))
}

pub(super) fn resolution<'o>(observed: &'o [Observation], label: &str) -> &'o Observation {
    observed
        .iter()
        .find(|observation| match observation {
            Observation::ResolvedTo { label: l, .. }
            | Observation::ResolvedElsewhere { label: l }
            | Observation::Retired { label: l }
            | Observation::ResolveRefused { label: l, .. } => l == label,
            _ => false,
        })
        .unwrap_or_else(|| panic!("`{label}` must be resolved: {observed:?}"))
}

pub(super) use record_edit;

fn resolved_to(label: &str, key: &str) -> Observation {
    Observation::ResolvedTo {
        label: label.to_owned(),
        key: key.to_owned(),
    }
}

fn retired(label: &str) -> Observation {
    Observation::Retired {
        label: label.to_owned(),
    }
}

#[test]
fn held_names_follow_delete_undo_branch_edit_checkout_and_redo() {
    let token = "history-continuity";
    let keys = ["direct", "nested"];
    let host = world::install();
    let document = world::attach(&host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let branch = |document: &world::Document<'_>| {
        document
            .selection(&request)
            .unwrap()
            .physical_endpoint()
            .product_branch()
    };
    let ended = |denial: &str| {
        assert!(
            denial.contains("PersistentName(UntrackedDependency)"),
            "{denial}"
        )
    };

    let (document, observed) = probe(&host, document, &request, token, &keys, 0);
    let direct = named(&observed, "direct");
    let nested = named(&observed, "nested");
    assert_ne!(direct, nested, "independent seeds keep distinct names");
    probe::hold(token, "direct", direct);
    probe::hold(token, "nested", nested);
    let original = world::member_identity(&host, branch(&document), "direct").unwrap();
    let live = document.current_step(&request).unwrap();

    // Deletion retires the name; the deleted record's handle no longer resolves.
    let document = record_edit!(
        &host,
        document,
        &request,
        delete::Intent {
            key: "direct".into()
        },
        DeleteAction,
        "delete-direct"
    );
    let deleted = document.current_step(&request).unwrap();
    assert_eq!(
        world::member_identity(&host, branch(&document), "direct"),
        None
    );
    let (document, observed, denial) = probe_refused(&host, document, &request, token, &keys, 1);
    ended(&denial);
    assert!(observed.contains(&Observation::Absent {
        key: "direct".into()
    }));
    assert_eq!(resolution(&observed, "direct"), &retired("direct"));
    assert_eq!(
        resolution(&observed, "nested"),
        &resolved_to("nested", "nested")
    );

    // `checkout` of the live node restores `direct` at its exact original
    // full-generation identity, and the held name resolves to it again.
    navigate(
        &document,
        &request,
        Navigate::Checkout(live.id()),
        0x5a71_0001,
    );
    assert_eq!(
        world::member_identity(&host, branch(&document), "direct"),
        Some(original)
    );
    let (document, observed) = probe(&host, document, &request, token, &keys, 2);
    assert_eq!(
        named(&observed, "direct"),
        direct,
        "restore keeps the exact name"
    );
    assert_eq!(
        resolution(&observed, "direct"),
        &resolved_to("direct", "direct")
    );
    assert_eq!(
        resolution(&observed, "nested"),
        &resolved_to("nested", "nested")
    );
    let branch_edit = document.current_step(&request).unwrap();

    // `checkout` of the deleted node retires it again.
    navigate(
        &document,
        &request,
        Navigate::Checkout(deleted.id()),
        0x5a71_0002,
    );
    assert_eq!(
        world::member_identity(&host, branch(&document), "direct"),
        None
    );
    let (document, observed, denial) = probe_refused(&host, document, &request, token, &keys, 3);
    ended(&denial);
    assert_eq!(resolution(&observed, "direct"), &retired("direct"));

    // `undo_step` from the deletion restores it once more.
    navigate(&document, &request, Navigate::Undo, 0x5a71_0003);
    assert_eq!(document.current_step(&request).unwrap().id(), live.id());
    assert_eq!(
        world::member_identity(&host, branch(&document), "direct"),
        Some(original)
    );

    // `redo_to` the branch edit: same name, same member, still distinct.
    navigate(
        &document,
        &request,
        Navigate::Redo(branch_edit.id()),
        0x5a71_0004,
    );
    let (document, observed) = probe(&host, document, &request, token, &keys, 4);
    assert_eq!(named(&observed, "direct"), direct);
    assert_eq!(named(&observed, "nested"), nested);
    assert_eq!(
        resolution(&observed, "direct"),
        &resolved_to("direct", "direct")
    );
    assert_eq!(
        resolution(&observed, "nested"),
        &resolved_to("nested", "nested")
    );
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}
