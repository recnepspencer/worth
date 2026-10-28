//! Public entry routing retains logical identity and original physical dependencies.
use super::super::{
    installation::{install, World},
    outcome, read,
};
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use std::{collections::BTreeSet, marker::PhantomData};
use worth_query_host::facade::{
    application_entry::{
        WorthQueryApplicationMutationOutcome as Outcome,
        WorthQueryApplicationProgramMutationPreparation as Preparation,
        WorthQueryApplicationRequestExt,
    },
    primary_graph::WorthQueryApplicationUncommitted,
    runtime::ExecutionAllocationPolicy as Allocation,
};
pub(super) fn checked<L: LawPolicy>(
    world: &World<L>,
    command: u64,
    root: u64,
) -> operation::Checked {
    let scope = request_scope();
    let principal = authenticate_operator(world.application.installed_schema(), &scope);
    let outcome = world
        .application
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.application.current_world())
        .advanced()
        .mutate(operation::Intent::<L> {
            input: operation::Input { root },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command)
        .execute_in_program(&world.application, Allocation::SystemAllocation)
        .unwrap();
    outcome
        .result()
        .unwrap_or_else(|| panic!("logical decision must complete: {outcome:?}"))
        .clone()
}
fn count(world: &World<Logical>, command: u64) {
    let scope = request_scope();
    let principal = authenticate_operator(world.application.installed_schema(), &scope);
    let outcome = world
        .application
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.application.current_world())
        .advanced()
        .mutate(derived::CountIntent::<Logical> {
            input: derived::CountInput {
                root: 0,
                bad_reference: false,
            },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command)
        .execute_in_program(&world.application, Allocation::SystemAllocation)
        .unwrap();
    assert_eq!(
        outcome.result().map(|value| value.count),
        Some(2),
        "{outcome:?}"
    );
}
fn assert_pair(world: &World<Logical>, root: u64, successor: u64, answer: &operation::Checked) {
    let graph = read::committed(world);
    assert!(graph.edges.contains(&(root, successor)));
    assert_eq!(
        answer.members.iter().cloned().collect::<BTreeSet<_>>(),
        BTreeSet::from([
            (graph.identities[&root], "Negative".into()),
            (graph.identities[&successor], "Positive".into())
        ])
    );
    assert_eq!(answer.distances, [0, 1]);
}
#[test]
fn logical_rules_decisions_orbits_and_entity_root_count_share_actual_source_owners() {
    use WorthQueryRuleReadableCollectionLifecycleForTest as State;
    let world = install::<Logical>(6, &[(0, 1), (1, 0), (2, 3), (3, 2), (4, 5), (5, 4)]);
    count(&world, 500);
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Warm]
    );
    let first = checked(&world, 501, 0);
    assert_pair(&world, 0, 1, &first);
    committed::assert_projection(&world, &first);
    outcome::committed(world.replace_edges(502, &[(0, 3), (2, 1)], &[(0, 1), (2, 3)]));
    // The complete physical permutation remains lawful, and count stays two
    // while the exact positive/empty dependency roster changes.
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Dirty]
    );
    count(&world, 503);
    let second = checked(&world, 504, 0);
    assert_pair(&world, 0, 3, &second);
    world
        .application
        .runtime()
        .discard_rule_readable_collections();
    assert!(world
        .application
        .runtime()
        .rule_readable_collection_lifecycle_for_test()
        .is_empty());
    count(&world, 505);
    assert_eq!(
        world
            .application
            .runtime()
            .rule_readable_collection_lifecycle_for_test(),
        [State::Warm]
    );
    let mut successors = vec![3, 0, 1, 2, 5, 4];
    let mut seed = 0x9181_0062_u64;
    for step in 0..12_u64 {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        let partner = 1 + seed as usize % 5;
        let removed = [(0, successors[0]), (partner as u64, successors[partner])];
        successors.swap(0, partner);
        let added = [(0, successors[0]), (partner as u64, successors[partner])];
        let command = 520 + step * 4;
        outcome::committed(world.replace_edges(command, &added, &removed));
        let expected = successors
            .iter()
            .enumerate()
            .map(|(from, to)| (from as u64, *to))
            .collect::<BTreeSet<_>>();
        assert_eq!(read::committed(&world).edges, expected);
        assert_eq!(
            world
                .application
                .runtime()
                .rule_readable_collection_lifecycle_for_test(),
            [State::Dirty]
        );
        count(&world, command + 1);
        assert_eq!(
            world
                .application
                .runtime()
                .rule_readable_collection_lifecycle_for_test(),
            [State::Warm]
        );
        if step % 3 == 0 {
            world
                .application
                .runtime()
                .discard_rule_readable_collections();
            assert!(world
                .application
                .runtime()
                .rule_readable_collection_lifecycle_for_test()
                .is_empty());
            count(&world, command + 2);
            assert_eq!(
                world
                    .application
                    .runtime()
                    .rule_readable_collection_lifecycle_for_test(),
                [State::Warm]
            );
        }
        let answer = checked(&world, command + 3, 0);
        assert_pair(&world, 0, successors[0], &answer);
    }
    world.close();
}
#[test]
fn equal_logical_answer_does_not_lend_a_stale_physical_source_to_a_candidate() {
    let world = install::<Logical>(6, &[(0, 1), (1, 0), (2, 3), (3, 2), (4, 5), (5, 4)]);
    let ids = read::committed(&world).identities;
    // The least full member is Negative on the lesser actual carrier. Its
    // surviving-key heir retains the old name across these two splits/merges.
    let root = if ids[&0] < ids[&1] { 0 } else { 1 };
    let successor = 1 - root;
    let original = checked(&world, 510, root);
    assert_pair(&world, root, successor, &original);
    let scope = request_scope();
    let principal = authenticate_operator(world.application.installed_schema(), &scope);
    let command = 511_u64;
    let mut request = world
        .application
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.application.current_world())
        .advanced()
        .mutate(operation::Intent::<Logical> {
            input: operation::Input { root },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command);
    let candidate = match request
        .prepare_in_program(&world.application, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => {
            panic!("fresh decision must retain a real unpublished candidate")
        }
    };
    outcome::committed(world.replace_edges(512, &[(0, 0), (1, 1)], &[(0, 1), (1, 0)]));
    outcome::committed(world.replace_edges(513, &[(0, 1), (1, 0)], &[(0, 0), (1, 1)]));
    let restored = checked(&world, 514, root);
    assert_eq!(
        restored, original,
        "same full cohort, name and result after real support rewire"
    );
    let before = read::committed(&world).edges;
    match candidate.commit(Allocation::SystemAllocation) {
        Outcome::Commit(WorthQueryApplicationUncommitted::Stale(stale)) => {
            assert!(stale.stale_fact_count() > 0)
        }
        other => {
            panic!("actual original source facts must stale the equal-answer candidate: {other:?}")
        }
    }
    assert_eq!(read::committed(&world).edges, before);
    drop(request);
    let command = 515_u64;
    let mut request = world
        .application
        .runtime()
        .request(&principal, &scope)
        .on_branch(world.application.current_world())
        .advanced()
        .mutate(operation::Intent::<Logical> {
            input: operation::Input { root },
            marker: PhantomData,
        })
        .without_source()
        .idempotency(&command);
    let candidate = match request
        .prepare_in_program(&world.application, Allocation::SystemAllocation)
        .unwrap()
    {
        Preparation::Prepared(candidate) => candidate,
        Preparation::Settled(_) => panic!("disjoint twin needs a real candidate"),
    };
    outcome::committed(world.replace_edges(516, &[(4, 4), (5, 5)], &[(4, 5), (5, 4)]));
    assert!(
        matches!(
            candidate.commit(Allocation::SystemAllocation),
            Outcome::Committed { .. }
        ),
        "an unrelated physical cohort must not stale this source"
    );
    drop(request);
    drop(principal);
    world.close();
}
