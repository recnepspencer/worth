//! Runtime group persistent names through an installed Query world.
//!
//! Every group here is declared by an installed operation's candidate writer
//! (`declare_group_partition`), named by Relational's group naming authority
//! at commit finalization under its kind's declared continuity rule, carried
//! in the commit's canonical group lineage, and read back through the
//! operation decision reader's tracked `group_name_of` / `resolve_group`.
use super::outputs::sculpt::{
    GroupAnswer, GroupDecl, GroupMember, GroupName, GroupRead, Held, Partition, Plan, Step,
};
use super::outputs::{birth, keys, named, World};
use super::*;
use worth_query_host::facade::application_naming::{NameSelection, WorthQueryName};

mod history;
mod invalidation;
mod lifecycle;
mod projection;
mod reads;
mod refusals;
pub(super) mod source_currentness;

/// A group member held by name.
pub fn held(name: WorthQueryName<Member>) -> GroupMember {
    GroupMember::Held(Held::Name(name))
}

/// One declared group: `(role, member)` pairs.
pub fn group(members: &[(GroupRole, WorthQueryName<Member>)]) -> Vec<(GroupRole, GroupMember)> {
    members
        .iter()
        .map(|(role, name)| (*role, held(*name)))
        .collect()
}

/// A world with `keys` created and named under authored occurrences, and
/// their names in `keys` order.
pub fn seeded(token: &str, members: &[&str]) -> (World, Vec<WorthQueryName<Member>>) {
    let world = World::install();
    world.committed(
        &format!("{token}-seed"),
        Plan {
            creates: keys(members),
            steps: members
                .iter()
                .map(|key| Step::Birth((*key).to_owned(), birth(key, "authored")))
                .collect(),
            ..Plan::default()
        },
    );
    let names = names(&world, token, members, 0);
    (world, names)
}

/// The current names of `members`, read by a probe.
pub fn names(
    world: &World,
    token: &str,
    members: &[&str],
    ordinal: u64,
) -> Vec<WorthQueryName<Member>> {
    let observed = world.probe(&format!("{token}-names-{ordinal}"), members, ordinal);
    members.iter().map(|key| named(&observed, key)).collect()
}

/// Commits one plan declaring `decl`, and nothing else.
pub fn declare(world: &World, token: &str, decl: GroupDecl) {
    world.committed(
        token,
        Plan {
            steps: vec![Step::Groups(decl)],
            ..Plan::default()
        },
    );
}

/// Runs `reads` in one decision and returns their answers.
pub fn read(world: &World, token: &str, reads: Vec<GroupRead>) -> Vec<GroupAnswer> {
    let _ = world.try_sculpt(
        token,
        Plan {
            group_reads: reads,
            ..Plan::default()
        },
    );
    super::outputs::sculpt::group_answers(token)
}

/// The `Loop` holding `member`, read by a fresh decision.
pub fn loop_of(
    world: &World,
    token: &str,
    member: WorthQueryName<Member>,
) -> Option<WorthQueryName<Loop>> {
    match read(world, token, vec![GroupRead::LoopOf(Held::Name(member))]).as_slice() {
        [GroupAnswer::Of(group)] => group.map(GroupName::loop_name),
        other => panic!("{token}: {other:?}"),
    }
}

/// Resolves a held `Loop` under `selection` in a fresh decision.
pub fn resolve_loop(
    world: &World,
    token: &str,
    held: WorthQueryName<Loop>,
    selection: NameSelection,
) -> GroupAnswer {
    let mut answers = read(world, token, vec![GroupRead::Loop(held, selection)]);
    answers.pop().expect("one read answers once")
}

/// The loop partition declaring `groups`, dissolving `dissolve`.
pub fn loops(
    groups: Vec<Vec<(GroupRole, GroupMember)>>,
    dissolve: &[WorthQueryName<Loop>],
) -> GroupDecl {
    GroupDecl::Loop(Partition {
        groups,
        dissolve: dissolve.to_vec(),
    })
}

/// `name` as a loop answer.
pub fn the_loop(name: WorthQueryName<Loop>) -> GroupName {
    GroupName::Loop(name)
}
