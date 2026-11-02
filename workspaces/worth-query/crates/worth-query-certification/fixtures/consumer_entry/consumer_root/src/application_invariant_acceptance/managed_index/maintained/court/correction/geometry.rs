//! The alternate provider consumes actual indirect history through recorded correction.
use super::super::super::{
    bounds::Coordinates,
    geometry::{self as model, authoring::Edit},
};
use super::*;

fn point(x: i64, y: i64) -> Coordinates {
    Coordinates::new(x, y, x, y)
}
fn seed(request: &Request<'_, '_, '_>, world: &host::Host, end: (i64, i64)) {
    for (offset, edit) in [
        Edit::Vertex {
            directory: "catalog".into(),
            identity: "correction-a".into(),
            x: 0,
            y: 0,
        },
        Edit::Vertex {
            directory: "catalog".into(),
            identity: "correction-b".into(),
            x: end.0,
            y: end.1,
        },
        Edit::Edge {
            directory: "A".into(),
            identity: "correction-edge".into(),
            start: "correction-a".into(),
            end: "correction-b".into(),
        },
    ]
    .into_iter()
    .enumerate()
    {
        raw(request, world, edit, 0x919c_0000 + offset as u64);
    }
}
fn raw(request: &Request<'_, '_, '_>, world: &host::Host, edit: Edit, key: u64) {
    let result = request
        .advanced()
        .mutate(edit)
        .idempotency(&key)
        .execute_in_program(world, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    assert!(
        matches!(
            result,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{result:?}"
    );
}
fn moved(end: (i64, i64)) -> Edit {
    Edit::Move {
        directory: "catalog".into(),
        vertex: "correction-b".into(),
        x: end.0,
        y: end.1,
    }
}
fn note(
    world: &host::Host,
    request: &Request<'_, '_, '_>,
    parent: note::Parent,
    empty: bool,
    key: u64,
) -> WorthQueryApplicationCommitReceipt {
    note_input(
        world,
        request,
        note::Input {
            parent,
            directory: "A".into(),
            prefix: String::new(),
            expect_empty: empty,
            replacement: "recorded note".into(),
        },
        key,
    )
}
fn check(request: &Request<'_, '_, '_>, note: &str, end: (i64, i64)) {
    let actual = request
        .query(state::Read {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(actual.rows().len(), 1);
    assert_eq!(actual.rows()[0].note, note);
    // Ordinary full Source path is independent of managed candidates/refinement.
    let source = request
        .query(model::query::ReadSource {
            directory: "A".into(),
        })
        .execute()
        .unwrap();
    assert_eq!(source.rows().len(), 1);
    let edge = &source.rows()[0];
    assert_eq!(edge.identity, "correction-edge");
    assert_eq!(edge.start, point(0, 0));
    assert_eq!(edge.end, point(end.0, end.1));
}

#[test]
fn box_correction_excludes_uncancelled_nonmatching_indirect_motion() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    seed(&request, &world, (10, 0));
    let source = note(
        &world,
        &request,
        note::Parent::Boxes {
            probe: point(25, 0),
        },
        true,
        0x919c_1001,
    );
    motion_source(&world, &request, "correction-b", point(11, 0), 0x919c_1002);
    check(&request, "recorded note", (11, 0));
    reverse_note(&world, &request, &source, "2442915843");
    check(&request, "", (11, 0));
}

#[test]
fn box_correction_keeps_transient_entrants_and_raw_restoration_without_performed_pairs() {
    for (probe, empty, end) in [(point(5, 0), false, (-1, 0)), (point(25, 0), true, (30, 0))] {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        seed(&request, &world, (10, 0));
        let source = note(
            &world,
            &request,
            note::Parent::Boxes { probe },
            empty,
            0x919c_1101,
        );
        raw(&request, &world, moved(end), 0x919c_1102);
        raw(&request, &world, moved((10, 0)), 0x919c_1103);
        check(&request, "recorded note", (10, 0));
        refuse_note(&request, &source, "2442916099");
        check(&request, "recorded note", (10, 0));
    }
}

#[test]
fn box_correction_accepts_genuine_recorded_motion_cancellation_for_populated_and_empty_inputs() {
    for (probe, empty, end) in [(point(5, 0), false, (-1, 0)), (point(25, 0), true, (30, 0))] {
        let calls = Arc::new(prefix::Calls::default());
        let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
        let scope = authentication::request_scope();
        let principal = authentication::block_on(
            authentication::admit(world.installed_schema()).authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &scope,
            ),
        )
        .unwrap();
        let request = world
            .runtime()
            .request(&principal, &scope)
            .on_branch(world.current_world());
        seed(&request, &world, (10, 0));
        let source = note(
            &world,
            &request,
            note::Parent::Boxes { probe },
            empty,
            0x919c_1201,
        );
        let motion = motion_source(
            &world,
            &request,
            "correction-b",
            point(end.0, end.1),
            0x919c_1202,
        );
        check(&request, "recorded note", end);
        reverse_motion(&world, &request, &motion, "2442916355");
        check(&request, "recorded note", (10, 0));
        reverse_note(&world, &request, &source, "2442916356");
        check(&request, "", (10, 0));
    }
}

#[test]
fn box_correction_keeps_matching_authored_noops_rejected_by_exact_contact() {
    let calls = Arc::new(prefix::Calls::default());
    let world = host::install(calls.clone(), Arc::new(rule::Calls::new(calls)));
    let scope = authentication::request_scope();
    let principal = authentication::block_on(
        authentication::admit(world.installed_schema()).authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &scope,
        ),
    )
    .unwrap();
    let request = world
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.current_world());
    seed(&request, &world, (10, 10));
    let probe = point(4, 5);
    assert_eq!(
        request
            .query(model::query::FindBoxes {
                directory: "A".into(),
                probe
            })
            .execute()
            .unwrap()
            .rows()
            .len(),
        1
    );
    assert!(request
        .query(model::refinement::FindContact {
            directory: "A".into(),
            probe
        })
        .execute()
        .unwrap()
        .rows()
        .is_empty());
    let source = note(
        &world,
        &request,
        note::Parent::Contact { probe },
        true,
        0x919c_1301,
    );
    let noop = motion_source(&world, &request, "correction-b", point(10, 10), 0x919c_1302);
    check(&request, "recorded note", (10, 10));
    refuse_note(&request, &source, "2442916611");
    reverse_motion(&world, &request, &noop, "2442916612");
    reverse_note(&world, &request, &source, "2442916613");
    check(&request, "", (10, 10));
}
