//! `HistoryDocument` navigation is an owned-graph selection: its commit
//! rematerializes records at their original identities and retires others,
//! and Relational derives the names from the selection alone. The watch
//! reads them in the proposal exactly as the commit seals them: checkout,
//! undo and redo restore the member's name, and retirement ends it.
use super::courts::{evaluated, gone};
use super::world::WatchProgram;
use super::*;
use crate::persistent_naming::history::{navigate, DeleteAction, Navigate};
use crate::persistent_naming::world::{attach_in, member_identity};

#[test]
fn history_navigation_restores_and_ends_the_names_the_invariant_reads() {
    let world = WatchWorld::install();
    let [direct] = world.names("watch-history", &["direct"], 0)[..] else {
        unreachable!()
    };
    let host = &world.host;
    let document = attach_in::<History, WatchProgram>(host);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let branch = |document: &world::DocumentOf<'_>| {
        let selection = document.selection(&request).unwrap();
        selection.physical_endpoint().product_branch()
    };
    let live = document.current_step(&request).unwrap();
    world.take();

    // Deleting `direct` retires its name.
    let document = delete(host, document, &request, "direct");
    let deleted = document.current_step(&request).unwrap();
    let (before, reobserved, _) = gone(&world.take(), "direct");
    assert_eq!((before, reobserved), (Ok(direct), false));
    assert_eq!(member_identity(host, branch(&document), "direct"), None);

    // Checkout of the live node rematerializes it: the proposal's selection
    // restores the name onto the record's original identity.
    navigate(
        &document,
        &request,
        Navigate::Checkout(live.id()),
        0x418_0001,
    );
    assert!(member_identity(host, branch(&document), "direct").is_some());
    restored(&world.take(), direct);

    // Checkout of the deleted node retires it again; the name ends.
    navigate(
        &document,
        &request,
        Navigate::Checkout(deleted.id()),
        0x418_0002,
    );
    let (before, reobserved, _) = gone(&world.take(), "direct");
    assert_eq!((before, reobserved), (Ok(direct), false));

    // Undo from the deletion restores it once more, and redo retires it.
    navigate(&document, &request, Navigate::Undo, 0x418_0003);
    restored(&world.take(), direct);
    navigate(
        &document,
        &request,
        Navigate::Redo(deleted.id()),
        0x418_0004,
    );
    let (before, reobserved, _) = gone(&world.take(), "direct");
    assert_eq!((before, reobserved), (Ok(direct), false));
    host.with_host_advancement(|phase| document.close(&phase).unwrap())
        .unwrap();
}

/// The selection's rematerialized `direct` carries `name` in the proposal,
/// and its birth is the name's recorded birth.
fn restored(seen: &[Seen], name: WorthQueryName<Member>) {
    let watched = evaluated(seen, "direct");
    assert_eq!(watched.proposed, Ok(name), "{seen:?}");
    match &watched.birth {
        Some(Ok(birth)) => assert_eq!(birth.name(), name),
        other => panic!("the restored name's birth reads: {other:?}"),
    }
    assert_eq!(watched.committed, None, "the record is rematerialized");
}

/// Records one `delete` edit on `document` in the watched program.
fn delete<'a>(
    host: &'a world::Host,
    document: world::DocumentOf<'a>,
    request: &WorthQueryApplicationRequest<'_, '_, '_, Schema>,
    key: &str,
) -> world::DocumentOf<'a> {
    let intent = delete::Intent { key: key.into() };
    let mut edit = document
        .edit(intent)
        .idempotency_owned(request, format!("watch-delete:{key}"))
        .unwrap()
        .retain_in_program::<WatchProgram, DeleteAction>(host)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    let ready = loop {
        edit = match edit.advance(request) {
            WorthQueryHistoryEditAdvance::Progress(next) => next,
            WorthQueryHistoryEditAdvance::Ready(next) => break next,
            WorthQueryHistoryEditAdvance::Refused { denial, .. } => panic!("{denial:?}"),
        };
    };
    let WorthQueryHistoryEditFinish { document, outcome } = ready
        .finish(request)
        .unwrap_or_else(|(_, cause)| panic!("{cause:?}"));
    if let WorthQueryHistoryEditOutcome::NotRecorded { terminal, .. } = &outcome {
        panic!("not recorded: {:?}", terminal.native_outcome());
    }
    document
}
