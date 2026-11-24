//! Invalid group declarations refuse typed at commit, naming the scheme and
//! the offending member or group, and commit nothing.
use super::super::outputs::sculpt::Partition;
use super::*;
use worth_query_host::facade::application_naming::{AggregateGroupNamingDenial, LineageNameDenial};
use GroupRole::{Host, Member as Plain, Producer};

fn group_refusal(
    world: &World,
    token: &str,
    plan: Plan,
    scheme: &str,
) -> AggregateGroupNamingDenial {
    let outcome = world.sculpt(token, plan);
    match super::super::outputs::naming_refusal(&outcome) {
        Some(LineageNameDenial::GroupNaming { scheme_id, denial }) => {
            assert_eq!(scheme_id, scheme);
            denial
        }
        other => panic!("{token}: expected a group refusal, got {other:?} from {outcome:?}"),
    }
}

fn groups_only(decl: GroupDecl) -> Plan {
    Plan {
        steps: vec![Step::Groups(decl)],
        ..Plan::default()
    }
}

fn shells(groups: Vec<Vec<(GroupRole, GroupMember)>>) -> GroupDecl {
    GroupDecl::Shell(Partition {
        groups,
        dissolve: Vec::new(),
    })
}

/// Each invalid loop declaration refuses typed and leaves the world unchanged.
#[test]
fn invalid_loop_declarations_refuse_typed_with_zero_effects() {
    let (world, n) = seeded("loop-refuse", &["h", "x", "y", "z"]);
    declare(
        &world,
        "loop-refuse-0",
        loops(vec![group(&[(Host, n[0]), (Plain, n[1])])], &[]),
    );
    let held = loop_of(&world, "loop-refuse-r0", n[0]).unwrap();
    let refused = |token: &str, plan| group_refusal(&world, token, plan, LOOP_SCHEME);

    // One member cannot be in two declared groups.
    let overlap = loops(
        vec![
            group(&[(Host, n[2]), (Plain, n[3])]),
            group(&[(Host, n[3])]),
        ],
        &[],
    );
    let denial = refused("loop-refuse-overlap", groups_only(overlap));
    assert!(
        matches!(denial, AggregateGroupNamingDenial::OverlappingMember { .. }),
        "{denial:?}"
    );
    // A member listed twice in one group.
    let twice = loops(vec![group(&[(Host, n[2]), (Plain, n[2])])], &[]);
    let denial = refused("loop-refuse-twice", groups_only(twice));
    assert!(
        matches!(
            denial,
            AggregateGroupNamingDenial::DuplicateMemberName { .. }
        ),
        "{denial:?}"
    );
    // A member the same commit deletes is not live.
    let deleted = Plan {
        reads: keys(&["y"]),
        deletes: keys(&["y"]),
        steps: vec![Step::Groups(loops(vec![group(&[(Host, n[2])])], &[]))],
        ..Plan::default()
    };
    let denial = refused("loop-refuse-dead", deleted);
    assert!(
        matches!(denial, AggregateGroupNamingDenial::MemberNotLive { .. }),
        "{denial:?}"
    );
    assert!(world.live("y"), "the refused delete committed nothing");
    // Dissolving a loop twice: the second names no live loop.
    declare(&world, "loop-refuse-dissolve", loops(Vec::new(), &[held]));
    let denial = refused("loop-refuse-again", groups_only(loops(Vec::new(), &[held])));
    assert!(
        matches!(denial, AggregateGroupNamingDenial::UnknownGroup { .. }),
        "{denial:?}"
    );
    // Nothing refused changed any group.
    assert_eq!(loop_of(&world, "loop-refuse-x", n[1]), None);
    assert_eq!(loop_of(&world, "loop-refuse-y", n[2]), None);
    assert_eq!(loop_of(&world, "loop-refuse-z", n[3]), None);
}

/// A shell's rule requires a producer: a shell declared without one, or a
/// producer deleted out of its shell, refuses typed and commits nothing.
#[test]
fn a_shell_without_its_producer_refuses_typed() {
    let (world, n) = seeded("shell-refuse", &["p", "f"]);
    let refused = |token: &str, plan| group_refusal(&world, token, plan, SHELL_SCHEME);
    let denial = refused(
        "shell-refuse-bare",
        groups_only(shells(vec![group(&[(Plain, n[1])])])),
    );
    assert!(
        matches!(
            denial,
            AggregateGroupNamingDenial::MissingAnchorMember { .. }
        ),
        "{denial:?}"
    );
    declare(
        &world,
        "shell-refuse-0",
        shells(vec![group(&[(Producer, n[0]), (Plain, n[1])])]),
    );
    let producer = Plan {
        reads: keys(&["p"]),
        deletes: keys(&["p"]),
        ..Plan::default()
    };
    let denial = refused("shell-refuse-producer", producer);
    assert!(
        matches!(
            denial,
            AggregateGroupNamingDenial::MissingAnchorMember { .. }
        ),
        "{denial:?}"
    );
    assert!(world.live("p"), "the refused delete committed nothing");
}

/// One loop declared twice overlaps itself: it refuses typed even when the
/// declared groups number exactly the loops a same-commit delete or dissolve
/// would retire, and the loop that would have retired stays live.
#[test]
fn a_loop_declared_twice_refuses_beside_a_retiring_loop() {
    let (world, n) = seeded("loop-twice", &["a", "b"]);
    declare(
        &world,
        "loop-twice-0",
        loops(vec![group(&[(Host, n[0])]), group(&[(Host, n[1])])], &[]),
    );
    let first = loop_of(&world, "loop-twice-a", n[0]).unwrap();
    let second = loop_of(&world, "loop-twice-b", n[1]).unwrap();
    let refused = |token: &str, plan| group_refusal(&world, token, plan, LOOP_SCHEME);
    let twice = || vec![group(&[(Host, n[0])]), group(&[(Host, n[0])])];
    let overlapping =
        |denial| matches!(denial, AggregateGroupNamingDenial::OverlappingMember { .. });
    // Deleting `b` would retire the second loop.
    let delete = Plan {
        reads: keys(&["b"]),
        deletes: keys(&["b"]),
        steps: vec![Step::Groups(loops(twice(), &[]))],
        ..Plan::default()
    };
    let denial = refused("loop-twice-delete", delete);
    assert!(overlapping(denial.clone()), "{denial:?}");
    assert!(world.live("b"), "the refused delete committed nothing");
    // Dissolving the second loop beside the same duplicate.
    let dissolve = groups_only(loops(twice(), &[second]));
    let denial = refused("loop-twice-dissolve", dissolve);
    assert!(overlapping(denial.clone()), "{denial:?}");
    assert_eq!(loop_of(&world, "loop-twice-a2", n[0]), Some(first));
    assert_eq!(loop_of(&world, "loop-twice-b2", n[1]), Some(second));
}
