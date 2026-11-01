//! Genuine recorded correction exercises the same owning Program and canonical witnesses.
use super::super::{
    correction::{self, mover, note, recorded, state},
    schema::Catalog,
};
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryApplicationCommitReceipt;
mod delivery;
use delivery::*;

#[test]
fn managed_correction_requires_genuine_cancellation_for_populated_and_empty_inputs() {
    for (prefix, empty, moved) in [("sh", false, "renamed"), ("needle", true, "needle")] {
        let calls = Arc::new(prefix::Calls::default());
        let rules = Arc::new(rule::Calls::new(calls.clone()));
        let world = host::install(calls, rules);
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
        let source = note_source(&world, &request, prefix, empty, 0x9194_1001);
        state(&request, "recorded note", "shared");
        let changed = move_source(&world, &request, moved, 0x9194_1002);
        state(&request, "recorded note", moved);
        reverse_mover(&world, &request, &changed, "2442391555");
        state(&request, "recorded note", "shared");
        reverse_note(&world, &request, &source, "2442391556");
        state(&request, "", "shared");
    }
}

#[test]
fn managed_correction_refuses_raw_restoration_and_an_uncancelled_transient_entrant() {
    for (prefix, empty, moved) in [("sh", false, "renamed"), ("needle", true, "needle")] {
        let calls = Arc::new(prefix::Calls::default());
        let rules = Arc::new(rule::Calls::new(calls.clone()));
        let world = host::install(calls, rules);
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
        let source = note_source(&world, &request, prefix, empty, 0x9194_1101);
        move_source(&world, &request, moved, 0x9194_1102);
        move_source(&world, &request, "shared", 0x9194_1103);
        state(&request, "recorded note", "shared");
        refuse_note(&request, &source, "2442391812");
        state(&request, "recorded note", "shared");
    }
}

#[test]
fn managed_correction_keeps_authored_noops_even_when_the_empty_result_stays_equal() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
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
    let source = note_source(&world, &request, "needle", true, 0x9194_1201);
    let noop = move_source(&world, &request, "shared", 0x9194_1202);
    state(&request, "recorded note", "shared");
    refuse_note(&request, &source, "2442392067");
    reverse_mover(&world, &request, &noop, "2442392068");
    reverse_note(&world, &request, &source, "2442392069");
    state(&request, "", "shared");
}

#[test]
fn managed_correction_scope_candidate_posture_conservatively_requires_irrelevant_edits_to_cancel() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
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
    let source = note_source(&world, &request, "needle", true, 0x9194_1301);
    // Neither endpoint enters the real probe. Exact same-scope relevance is
    // deliberately unfinished; this scope-candidate posture requires a pair.
    let unrelated = move_source(&world, &request, "elsewhere", 0x9194_1302);
    refuse_note(&request, &source, "2442392323");
    reverse_mover(&world, &request, &unrelated, "2442392324");
    reverse_note(&world, &request, &source, "2442392325");
    state(&request, "", "shared");
}

#[test]
fn ordinary_recorded_correction_preserves_an_intervening_disjoint_note() {
    let calls = Arc::new(prefix::Calls::default());
    let rules = Arc::new(rule::Calls::new(calls.clone()));
    let world = host::install(calls, rules);
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
    let moved = move_source(&world, &request, "renamed", 0x9194_1401);
    note_source(&world, &request, "needle", true, 0x9194_1402);
    state(&request, "recorded note", "renamed");
    reverse_mover(&world, &request, &moved, "2442392580");
    state(&request, "recorded note", "shared");
}
