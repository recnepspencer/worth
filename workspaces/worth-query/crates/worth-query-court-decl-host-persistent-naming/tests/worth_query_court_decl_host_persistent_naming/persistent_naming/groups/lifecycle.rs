//! Group continuity through committed partitions: birth, preserve, split,
//! merge, retire and revive, each read back by tracked decision reads.
use super::*;
use GroupRole::{Host, Member as Plain};

/// A loop is born around its host, keeps its exact name through unrelated
/// commits, and every member reads it.
#[test]
fn a_declared_loop_is_born_and_preserved() {
    let (world, n) = seeded("loop-birth", &["h", "x", "y", "other"]);
    declare(
        &world,
        "loop-birth-declare",
        loops(vec![group(&[(Host, n[0]), (Plain, n[1])])], &[]),
    );
    let born = loop_of(&world, "loop-birth-h", n[0]).expect("the host is grouped");
    assert_eq!(loop_of(&world, "loop-birth-x", n[1]), Some(born));
    assert_eq!(loop_of(&world, "loop-birth-y", n[2]), None);
    // An unrelated partition of another loop preserves this one.
    declare(
        &world,
        "loop-birth-other",
        loops(vec![group(&[(Host, n[3])])], &[]),
    );
    assert_eq!(loop_of(&world, "loop-birth-again", n[0]), Some(born));
    assert_eq!(
        resolve_loop(&world, "loop-birth-resolve", born, NameSelection::Exact),
        GroupAnswer::Exact(the_loop(born))
    );
}

/// The same loop declared in two fresh worlds, members listed in opposite
/// orders, is the same name: allocation and order never reach it.
#[test]
fn loop_names_are_independent_of_world_and_member_order() {
    let (first, a) = seeded("loop-det", &["h", "x", "y"]);
    declare(
        &first,
        "loop-det-a",
        loops(
            vec![group(&[(Host, a[0]), (Plain, a[1]), (Plain, a[2])])],
            &[],
        ),
    );
    let (second, b) = seeded("loop-det", &["h", "x", "y"]);
    declare(
        &second,
        "loop-det-b",
        loops(
            vec![group(&[(Plain, b[2]), (Plain, b[1]), (Host, b[0])])],
            &[],
        ),
    );
    assert_eq!(a, b, "member names are allocation independent");
    assert_eq!(
        loop_of(&first, "loop-det-ra", a[0]),
        loop_of(&second, "loop-det-rb", b[0])
    );
}

/// Splitting a loop keeps its name with the piece holding the host; the
/// other piece is a new loop, and the held name resolves to the keeper.
#[test]
fn a_split_keeps_the_name_with_the_host() {
    let (world, n) = seeded("loop-split", &["h", "x", "y", "z"]);
    declare(
        &world,
        "loop-split-0",
        loops(
            vec![group(&[
                (Host, n[0]),
                (Plain, n[1]),
                (Plain, n[2]),
                (Plain, n[3]),
            ])],
            &[],
        ),
    );
    let whole = loop_of(&world, "loop-split-r0", n[0]).unwrap();
    // `y` and `z` move into their own loop; the residual keeps `h`, `x`.
    declare(
        &world,
        "loop-split-1",
        loops(vec![group(&[(Host, n[2]), (Plain, n[3])])], &[]),
    );
    assert_eq!(loop_of(&world, "loop-split-h", n[0]), Some(whole));
    assert_eq!(loop_of(&world, "loop-split-x", n[1]), Some(whole));
    let piece = loop_of(&world, "loop-split-y", n[2]).expect("the piece is grouped");
    assert_ne!(piece, whole);
    assert_eq!(loop_of(&world, "loop-split-z", n[3]), Some(piece));
    assert_eq!(
        resolve_loop(&world, "loop-split-resolve", whole, NameSelection::Exact),
        GroupAnswer::Exact(the_loop(whole))
    );
}

/// Merging two loops keeps one of their names by the continuity rule and
/// retires the other into it; the absorbed name resolves to the keeper.
#[test]
fn a_merge_keeps_one_name_and_absorbs_the_other() {
    let (world, n) = seeded("loop-merge", &["h1", "x", "h2", "y"]);
    declare(
        &world,
        "loop-merge-0",
        loops(
            vec![
                group(&[(Host, n[0]), (Plain, n[1])]),
                group(&[(Host, n[2]), (Plain, n[3])]),
            ],
            &[],
        ),
    );
    let first = loop_of(&world, "loop-merge-r1", n[0]).unwrap();
    let second = loop_of(&world, "loop-merge-r2", n[2]).unwrap();
    declare(
        &world,
        "loop-merge-1",
        loops(
            vec![group(&[
                (Host, n[0]),
                (Plain, n[1]),
                (Plain, n[2]),
                (Plain, n[3]),
            ])],
            &[],
        ),
    );
    let merged = loop_of(&world, "loop-merge-after", n[3]).unwrap();
    assert!(
        merged == first || merged == second,
        "a merge keeps a prior name"
    );
    let absorbed = if merged == first { second } else { first };
    assert_eq!(
        resolve_loop(
            &world,
            "loop-merge-absorbed",
            absorbed,
            NameSelection::Exact
        ),
        GroupAnswer::Exact(the_loop(merged))
    );
}

/// Dissolving a loop retires its name; re-forming a loop around the same
/// host revives that exact name.
#[test]
fn a_dissolved_loop_retires_and_revives_around_its_host() {
    let (world, n) = seeded("loop-revive", &["h", "x"]);
    declare(
        &world,
        "loop-revive-0",
        loops(vec![group(&[(Host, n[0]), (Plain, n[1])])], &[]),
    );
    let born = loop_of(&world, "loop-revive-r0", n[0]).unwrap();
    declare(&world, "loop-revive-1", loops(Vec::new(), &[born]));
    assert_eq!(loop_of(&world, "loop-revive-gone", n[0]), None);
    assert_eq!(
        resolve_loop(&world, "loop-revive-retired", born, NameSelection::Exact),
        GroupAnswer::Retired
    );
    declare(
        &world,
        "loop-revive-2",
        loops(vec![group(&[(Host, n[0])])], &[]),
    );
    assert_eq!(loop_of(&world, "loop-revive-back", n[0]), Some(born));
}

/// Deleting a grouped member shrinks its loop without any declaration; the
/// loop keeps its name while its host lives.
#[test]
fn a_member_deletion_shrinks_its_loop() {
    let (world, n) = seeded("loop-shrink", &["h", "x", "y"]);
    declare(
        &world,
        "loop-shrink-0",
        loops(
            vec![group(&[(Host, n[0]), (Plain, n[1]), (Plain, n[2])])],
            &[],
        ),
    );
    let born = loop_of(&world, "loop-shrink-r0", n[0]).unwrap();
    world.committed(
        "loop-shrink-delete",
        Plan {
            reads: keys(&["y"]),
            deletes: keys(&["y"]),
            ..Plan::default()
        },
    );
    assert_eq!(loop_of(&world, "loop-shrink-h", n[0]), Some(born));
    assert_eq!(loop_of(&world, "loop-shrink-x", n[1]), Some(born));
}
