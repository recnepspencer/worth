//! `#[worth_query(group_name)]` projects the groups a row's own name belongs
//! to in the same read, matching what a decision's tracked read answers.
use super::super::named_read::{self, GroupedMember};
use super::super::outputs::sculpt::Partition;
use super::*;
use GroupRole::{Host, Member as Plain, Producer};

#[test]
fn a_projected_row_carries_its_group_names() {
    let (world, n) = seeded("projected", &["ph", "px", "pl"]);
    declare(
        &world,
        "projected-loop",
        loops(vec![group(&[(Host, n[0]), (Plain, n[1])])], &[]),
    );
    let shells = GroupDecl::Shell(Partition {
        groups: vec![group(&[(Producer, n[0])])],
        dissolve: Vec::new(),
    });
    declare(&world, "projected-shell", shells);
    let in_loop = loop_of(&world, "projected-loop-of", n[1]).unwrap();
    let shell = match read(
        &world,
        "projected-shell-of",
        vec![GroupRead::ShellOf(Held::Name(n[0]))],
    )
    .as_slice()
    {
        [GroupAnswer::Of(Some(shell))] => shell.shell_name(),
        other => panic!("{other:?}"),
    };
    let request = world.request();
    let on_world = request.on_branch(world.host.current_world());
    let row = |key: &str| {
        let read = on_world
            .query(named_read::grouped_member::Read(key.into()))
            .execute()
            .unwrap_or_else(|cause| panic!("{key}: {cause:?}"));
        read.rows().to_vec()
    };
    assert_eq!(
        row("ph"),
        [GroupedMember {
            key: "ph".into(),
            name: n[0],
            in_loop: Some(in_loop),
            in_shell: Some(shell)
        }]
    );
    assert_eq!(
        row("px"),
        [GroupedMember {
            key: "px".into(),
            name: n[1],
            in_loop: Some(in_loop),
            in_shell: None
        }]
    );
    assert_eq!(
        row("pl"),
        [GroupedMember {
            key: "pl".into(),
            name: n[2],
            in_loop: None,
            in_shell: None
        }]
    );
    // A later split is what the next read projects.
    declare(
        &world,
        "projected-split",
        loops(vec![group(&[(Host, n[1])])], &[]),
    );
    let piece = loop_of(&world, "projected-piece", n[1]).unwrap();
    assert_ne!(piece, in_loop);
    let request = world.request();
    let on_world = request.on_branch(world.host.current_world());
    let read = on_world
        .query(named_read::grouped_member::Read("px".into()))
        .execute()
        .unwrap();
    assert_eq!(read.rows()[0].in_loop, Some(piece));
}
