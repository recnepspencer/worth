//! Tracked group reads: `Exact`, `Plural`, `Retired` and `Ambiguous` parity
//! with entity resolution, and the other two continuity rules.
use super::super::outputs::sculpt::Partition;
use super::*;
use GroupRole::{Host, Member as Plain, Producer, Wall};

/// Deleting a loop's host while re-partitioning the rest leaves no keeper:
/// the loop ends with two successors, `Exact` refuses `Ambiguous` naming
/// both, and `AllDescendants` answers both.
#[test]
fn an_ended_loop_with_two_successors_is_ambiguous_never_picked() {
    let (world, n) = seeded("loop-ambiguous", &["h", "x", "y"]);
    declare(
        &world,
        "loop-ambiguous-0",
        loops(
            vec![group(&[(Host, n[0]), (Plain, n[1]), (Plain, n[2])])],
            &[],
        ),
    );
    let whole = loop_of(&world, "loop-ambiguous-r0", n[0]).unwrap();
    world.committed(
        "loop-ambiguous-1",
        Plan {
            reads: keys(&["h"]),
            deletes: keys(&["h"]),
            steps: vec![Step::Groups(loops(
                vec![group(&[(Host, n[1])]), group(&[(Host, n[2])])],
                &[],
            ))],
            ..Plan::default()
        },
    );
    let left = loop_of(&world, "loop-ambiguous-x", n[1]).unwrap();
    let right = loop_of(&world, "loop-ambiguous-y", n[2]).unwrap();
    let mut both = vec![the_loop(left), the_loop(right)];
    both.sort();
    match resolve_loop(&world, "loop-ambiguous-exact", whole, NameSelection::Exact) {
        GroupAnswer::Ambiguous(mut candidates) => {
            candidates.sort();
            assert_eq!(candidates, both);
        }
        other => panic!("Exact must refuse Ambiguous: {other:?}"),
    }
    match resolve_loop(
        &world,
        "loop-ambiguous-all",
        whole,
        NameSelection::AllDescendants,
    ) {
        GroupAnswer::Plural(mut answered) => {
            answered.sort();
            assert_eq!(answered, both);
        }
        other => panic!("AllDescendants answers every piece: {other:?}"),
    }
    // Dissolving one piece leaves the other as the only live meaning.
    declare(&world, "loop-ambiguous-2", loops(Vec::new(), &[left]));
    assert_eq!(
        resolve_loop(&world, "loop-ambiguous-one", whole, NameSelection::Exact),
        GroupAnswer::Exact(the_loop(right))
    );
    declare(&world, "loop-ambiguous-3", loops(Vec::new(), &[right]));
    assert_eq!(
        resolve_loop(&world, "loop-ambiguous-none", whole, NameSelection::Exact),
        GroupAnswer::Retired
    );
}

/// A zone follows the strict majority of its walls; a tie keeps nothing.
#[test]
fn a_zone_follows_the_majority_of_its_walls() {
    let (world, n) = seeded("zone", &["w1", "w2", "w3", "r1", "r2"]);
    let zone = |groups, dissolve| GroupDecl::Zone(Partition { groups, dissolve });
    declare(
        &world,
        "zone-0",
        zone(
            vec![group(&[
                (Wall, n[0]),
                (Wall, n[1]),
                (Wall, n[2]),
                (Plain, n[3]),
                (Plain, n[4]),
            ])],
            Vec::new(),
        ),
    );
    let zone_of = |token: &str, member| match read(
        &world,
        token,
        vec![GroupRead::ZoneOf(Held::Name(member))],
    )
    .as_slice()
    {
        [GroupAnswer::Of(group)] => group.map(GroupName::zone_name),
        other => panic!("{token}: {other:?}"),
    };
    let born = zone_of("zone-r0", n[3]).expect("the zone is born");
    // `w3` and `r2` leave; the two walls left behind keep the zone's name.
    declare(
        &world,
        "zone-1",
        zone(vec![group(&[(Wall, n[2]), (Plain, n[4])])], Vec::new()),
    );
    assert_eq!(zone_of("zone-r1", n[0]), Some(born));
    let piece = zone_of("zone-r2", n[4]).unwrap();
    assert_ne!(piece, born);
    // Splitting the kept zone one wall each way ties: neither keeps it, so
    // its name ends with two successors and `Exact` never picks one.
    declare(
        &world,
        "zone-2",
        zone(vec![group(&[(Wall, n[1]), (Plain, n[3])])], Vec::new()),
    );
    let left = zone_of("zone-r3", n[0]).unwrap();
    let right = zone_of("zone-r4", n[1]).unwrap();
    assert!(left != born && right != born && left != right);
    let resolved = read(
        &world,
        "zone-r5",
        vec![GroupRead::Zone(born, NameSelection::Exact)],
    );
    assert!(
        matches!(resolved.as_slice(), [GroupAnswer::Ambiguous(both)] if both.len() == 2),
        "{resolved:?}"
    );
}

/// A shell follows its producer: adding and removing members keeps its name.
#[test]
fn a_shell_follows_its_producer() {
    let (world, n) = seeded("shell", &["p", "f1", "f2", "f3"]);
    let shell = |groups| {
        GroupDecl::Shell(Partition {
            groups,
            dissolve: Vec::new(),
        })
    };
    declare(
        &world,
        "shell-0",
        shell(vec![group(&[
            (Producer, n[0]),
            (Plain, n[1]),
            (Plain, n[2]),
        ])]),
    );
    let shell_of = |token: &str, member| match read(
        &world,
        token,
        vec![GroupRead::ShellOf(Held::Name(member))],
    )
    .as_slice()
    {
        [GroupAnswer::Of(group)] => group.map(GroupName::shell_name),
        other => panic!("{token}: {other:?}"),
    };
    let born = shell_of("shell-r0", n[1]).expect("the shell is born");
    // Growing the shell keeps its name: it follows its producer.
    declare(
        &world,
        "shell-1",
        shell(vec![group(&[
            (Producer, n[0]),
            (Plain, n[1]),
            (Plain, n[2]),
            (Plain, n[3]),
        ])]),
    );
    assert_eq!(shell_of("shell-r1", n[3]), Some(born));
    // Dissolving while redeclaring the producer keeps the name; members the
    // partition does not redeclare leave the kind.
    let shrink = GroupDecl::Shell(Partition {
        groups: vec![group(&[(Producer, n[0]), (Plain, n[1])])],
        dissolve: vec![born],
    });
    declare(&world, "shell-2", shrink);
    assert_eq!(shell_of("shell-r2", n[1]), Some(born));
    assert_eq!(shell_of("shell-r3", n[2]), None);
    // Without its producer a remainder cannot stand as a shell: the
    // undeclared remainder of a scoped shell must hold one.
    let outcome = world.sculpt(
        "shell-3",
        Plan {
            steps: vec![Step::Groups(shell(vec![group(&[(Producer, n[0])])]))],
            ..Plan::default()
        },
    );
    assert!(
        super::super::outputs::naming_refusal(&outcome).is_some(),
        "{outcome:?}"
    );
    assert_eq!(
        shell_of("shell-r4", n[0]),
        Some(born),
        "the refusal committed nothing"
    );
}
