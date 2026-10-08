//! Two root outputs feed one shared dependent through two consumed edges.

use super::*;
use worth_query_consumer_values::PositiveLength;
use worth_query_host::facade::application_entry::WorthQueryApplicationPerformedMutationOutcome;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntityKey, WorthQueryApplicationEntitySeed,
    WorthQueryApplicationRelationSeed, WorthQueryPrimaryGraphBootstrap,
};

/// Three separate rings: each root's source reads only its own ring, and the
/// shared dependent's source reads only its own.
fn seed(graph: &mut WorthQueryPrimaryGraphBootstrap<CheckpointSchema>) {
    for (ring, offset) in [("left", 1), ("right", 50), ("join", 100)] {
        let keys = [
            format!("diamond-{ring}"),
            format!("diamond-{ring}-b"),
            format!("diamond-{ring}-c"),
        ];
        for (key, (x, y)) in keys.iter().zip([(0, 0), (9, 0), (0, 9)]) {
            graph
                .bind_entity(
                    WorthQueryApplicationEntitySeed::new(
                        Body::reference::<CheckpointSchema>(),
                        entity_key(key),
                    )
                    .field(BodyKey::reference::<CheckpointSchema>(), key.clone())
                    .field(
                        Length::reference::<CheckpointSchema>(),
                        length(offset + y + 1),
                    )
                    .field(
                        PositionX::reference::<CheckpointSchema>(),
                        length(offset + x),
                    )
                    .field(
                        PositionY::reference::<CheckpointSchema>(),
                        length(offset + y),
                    ),
                )
                .unwrap();
        }
        for (from, to) in [(0, 1), (1, 2), (2, 0)] {
            graph
                .bind_relation(WorthQueryApplicationRelationSeed::new(
                    PlanarSuccessor::reference::<CheckpointSchema>(),
                    format!("{}-to-{}", keys[from], keys[to]),
                    entity_key(&keys[from]),
                    entity_key(&keys[to]),
                ))
                .unwrap();
        }
    }
}

fn entity_key(key: &str) -> WorthQueryApplicationEntityKey<CheckpointSchema, Body> {
    WorthQueryApplicationEntityKey::new(key.to_owned()).unwrap()
}

macro_rules! settle {
    ($demand:expr, $request:expr) => {
        (0..256)
            .find_map(|_| match $demand.advance(&$request).unwrap() {
                WorthQueryApplicationOutputDemandProgress::Pending => None,
                WorthQueryApplicationOutputDemandProgress::Settled(settled) => Some(settled),
            })
            .expect("the diamond demand initially settles")
    };
}

macro_rules! settled_in_one_advance {
    ($demand:expr, $request:expr, $what:literal) => {
        match $demand.advance(&$request).unwrap() {
            WorthQueryApplicationOutputDemandProgress::Settled(settled) => settled,
            WorthQueryApplicationOutputDemandProgress::Pending => {
                panic!(concat!($what, " needs another advance"))
            }
        }
    };
}

/// Moves a root's source Y, which changes that root producer's input. The
/// new Y must keep the ring's turn positive, or the commit is denied.
macro_rules! change_root {
    ($request:expr, $application:expr, $root:literal, $y:expr, $idempotency:expr) => {{
        let selected = $request
            .query(PlanarRead {
                body_key: $root.to_owned(),
            })
            .execute()
            .unwrap();
        let outcome = $request
            .mutate(PlanarSourceAdjustment {
                scope_key: $root.to_owned(),
                replacement_y: length($y),
            })
            .expect_source(selected.observed_sources()[0].clone())
            .idempotency(&$idempotency)
            .execute_performed::<program::ChainProgram, program::ChainRoot>(&$application)
            .unwrap();
        assert!(
            matches!(
                outcome,
                WorthQueryApplicationPerformedMutationOutcome::Performed(_)
            ),
            "the {} source change commits",
            $root
        );
    }};
}

/// The Length each named output carries now.
macro_rules! output_lengths {
    ($request:expr, [$($key:literal),+]) => {
        [$(PositiveLength::get(
            &$request
                .query(PlanarOutputRead {
                    body_key: $key.to_owned(),
                })
                .execute()
                .unwrap()
                .rows()[0]
                .value,
        )),+]
    };
}

#[test]
fn a_diamond_output_settles_in_one_advance_after_both_roots_change() {
    let _guard = checkpoint_recovery_test_guard();
    let profile =
        worth_query_host::facade::runtime::WorthQueryOutputDemandResourceProfile::standard();
    let application = support::install_program_with_seed::<program::ChainProgram>(
        None,
        profile,
        4_096,
        128 * 1_024 * 1_024,
        u64::try_from(profile.limits().source_currentness_work()).unwrap(),
        seed,
    );
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let mut left = request
        .demand(PlanarOutputDemand::new("diamond-left"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut right = request
        .demand(PlanarOutputDemand::new("diamond-right"))
        .start_in_program::<program::ChainProgram, program::ChainRoot>(&application)
        .unwrap();
    let mut join = request
        .demand(ChainDemand("diamond-join".to_owned()))
        .start_dependent_in_program::<program::ChainProgram, program::ChainConnection>(&application)
        .unwrap();
    settle!(left, request);
    settle!(right, request);
    settle!(join, request);
    assert_eq!(
        take_decisions("diamond-join"),
        [[2, 51]],
        "the shared dependent first reads both seeded root outputs"
    );

    // Both roots change; one caller advance of the shared dependent follows
    // both consumed edges.
    change_root!(request, application, "diamond-left", 5, 0x9176_3500_u64);
    change_root!(request, application, "diamond-right", 54, 0x9176_3501_u64);
    crate::producer::reset_provider_contacts();
    let settled = settled_in_one_advance!(join, request, "the shared dependent");
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        2,
        "initial execution plus 1 join refreshes"
    );
    let both_roots = crate::producer::provider_contacts();
    assert_eq!(
        output_lengths!(request, ["diamond-left", "diamond-right"]),
        [6, 55],
        "both roots published their new outputs inside that advance"
    );
    assert_eq!(
        take_decisions("diamond-join"),
        [[6, 55]],
        "the shared dependent decided once, reading both new root outputs"
    );
    let basis = request.retain_read().unwrap();
    for (name, root) in [("left", &mut left), ("right", &mut right)] {
        let settled = settled_in_one_advance!(root, request, "a refreshed root");
        assert_eq!(
            settled.producer_contacts_in_this_demand(),
            1,
            "the {name} root retains its initial execution; the upstream refresh is counted by no demand handle"
        );
    }
    assert_eq!(
        request.retain_read().unwrap().selected_commit(),
        basis.selected_commit(),
        "the refreshed roots were already current"
    );

    // Only the left root changes: the right branch is reused clean.
    change_root!(request, application, "diamond-left", 7, 0x9176_3502_u64);
    crate::producer::reset_provider_contacts();
    let settled = settled_in_one_advance!(join, request, "the shared dependent");
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        3,
        "initial execution plus 2 join refreshes"
    );
    // The shared planar provider counts each refresh contact the same way.
    let left_root = crate::producer::provider_contacts();
    assert!(left_root > 0);
    assert_eq!(
        both_roots,
        2 * left_root,
        "each changed root producer was contacted once, the unchanged one never"
    );
    assert_eq!(
        take_decisions("diamond-join"),
        [[8, 55]],
        "the shared dependent reads the new left output and the unchanged right one"
    );
    let settled = settled_in_one_advance!(right, request, "the clean right root");
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        1,
        "the unchanged right root retains its initial execution"
    );

    // The left root returns to an earlier value, so the shared dependent's
    // inputs equal those of its first refreshed decision. Its consumed edge
    // still names the newer left output; one advance settles it again.
    change_root!(request, application, "diamond-left", 5, 0x9176_3503_u64);
    let settled = settled_in_one_advance!(join, request, "the shared dependent");
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        4,
        "initial execution plus 3 join refreshes"
    );
    assert_eq!(
        output_lengths!(request, ["diamond-left", "diamond-right"]),
        [6, 55],
        "the left root published its earlier output again"
    );
    assert_eq!(
        take_decisions("diamond-join"),
        [[6, 55]],
        "the shared dependent reads the left output it returned to"
    );
    let settled = settled_in_one_advance!(left, request, "the returned left root");
    assert_eq!(
        settled.producer_contacts_in_this_demand(),
        1,
        "the left root retains its initial execution; the upstream refresh is counted by no demand handle"
    );
    drop((left, right, join));
}
