//! Group partitions a sculpt plan declares, and the tracked group reads its
//! decision makes, through the public candidate writer and decision reader.
use std::collections::BTreeMap;

use super::super::super::*;
use super::{Binding, Held};
use worth_query_host::facade::application_naming::{
    ApplicationGroupKind, ExecutionNameResolveDenial, GroupMemberName, NameResolutionDenial,
    NameSelection, ResolvedName, WorthQueryDecisionGroupResolution, WorthQueryGroupPartition,
    WorthQueryName,
};

/// A declared group member: a held name, or a member this run created and
/// named, referenced by its same-commit name.
#[derive(Clone, Debug)]
pub enum GroupMember {
    Held(Held),
    Created(String),
}

/// One group kind's declared partition.
#[derive(Clone, Debug)]
pub struct Partition<G> {
    pub groups: Vec<Vec<(GroupRole, GroupMember)>>,
    pub dissolve: Vec<WorthQueryName<G>>,
}

impl<G> Default for Partition<G> {
    fn default() -> Self {
        Self {
            groups: Vec::new(),
            dissolve: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum GroupDecl {
    Loop(Partition<Loop>),
    Zone(Partition<Zone>),
    Shell(Partition<Shell>),
}

/// One tracked group read the decision makes.
#[derive(Clone, Debug)]
pub enum GroupRead {
    LoopOf(Held),
    ShellOf(Held),
    ZoneOf(Held),
    Loop(WorthQueryName<Loop>, NameSelection),
    Zone(WorthQueryName<Zone>, NameSelection),
}

/// A group name of one of the test kinds, exactly as a read returned it.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum GroupName {
    Loop(WorthQueryName<Loop>),
    Zone(WorthQueryName<Zone>),
    Shell(WorthQueryName<Shell>),
}

impl GroupName {
    pub fn loop_name(self) -> WorthQueryName<Loop> {
        match self {
            Self::Loop(name) => name,
            other => panic!("not a loop: {other:?}"),
        }
    }

    pub fn zone_name(self) -> WorthQueryName<Zone> {
        match self {
            Self::Zone(name) => name,
            other => panic!("not a zone: {other:?}"),
        }
    }

    pub fn shell_name(self) -> WorthQueryName<Shell> {
        match self {
            Self::Shell(name) => name,
            other => panic!("not a shell: {other:?}"),
        }
    }
}

trait Typed: ApplicationGroupKind + Sized {
    fn wrap(name: WorthQueryName<Self>) -> GroupName;
}
impl Typed for Loop {
    fn wrap(name: WorthQueryName<Self>) -> GroupName {
        GroupName::Loop(name)
    }
}
impl Typed for Zone {
    fn wrap(name: WorthQueryName<Self>) -> GroupName {
        GroupName::Zone(name)
    }
}
impl Typed for Shell {
    fn wrap(name: WorthQueryName<Self>) -> GroupName {
        GroupName::Shell(name)
    }
}

/// What one group read answered.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupAnswer {
    Of(Option<GroupName>),
    Exact(GroupName),
    Plural(Vec<GroupName>),
    Retired,
    Ambiguous(Vec<GroupName>),
    Refused(String),
}

pub(super) fn declare(
    decl: &GroupDecl,
    held: &impl Fn(&Held) -> WorthQueryName<Member>,
    members: &BTreeMap<String, WorthQueryApplicationEffectEntity<Schema, Member>>,
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    match decl {
        GroupDecl::Loop(partition) => declare_kind(partition, held, members, writer),
        GroupDecl::Zone(partition) => declare_kind(partition, held, members, writer),
        GroupDecl::Shell(partition) => declare_kind(partition, held, members, writer),
    }
}

fn declare_kind<G: ApplicationGroupKind>(
    partition: &Partition<G>,
    held: &impl Fn(&Held) -> WorthQueryName<Member>,
    members: &BTreeMap<String, WorthQueryApplicationEffectEntity<Schema, Member>>,
    writer: &mut CandidateWriter<'_, Schema, Binding>,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let mut declared = WorthQueryGroupPartition::<G>::new();
    for group in &partition.groups {
        let mut named = Vec::with_capacity(group.len());
        for (role, member) in group {
            let name: GroupMemberName = match member {
                GroupMember::Held(member) => held(member).into(),
                GroupMember::Created(key) => writer.same_commit_entity_name(&members[key])?.into(),
            };
            named.push((*role, name));
        }
        declared = declared.group(named);
    }
    for group in &partition.dissolve {
        declared = declared.dissolve(group);
    }
    writer.declare_group_partition(declared)
}

pub(super) fn read(
    read: &GroupRead,
    names: &BTreeMap<String, WorthQueryName<Member>>,
    reader: &mut DecisionReader<'_, '_, '_, Schema, Binding>,
) -> GroupAnswer {
    let member = |held: &Held| match held {
        Held::Read(key) => names[key],
        Held::Name(name) => *name,
    };
    let reader = reader.reader();
    match read {
        GroupRead::LoopOf(held) => of(reader.group_name_of::<Loop, _>(&member(held))),
        GroupRead::ZoneOf(held) => of(reader.group_name_of::<Zone, _>(&member(held))),
        GroupRead::ShellOf(held) => of(reader.group_name_of::<Shell, _>(&member(held))),
        GroupRead::Loop(held, selection) => {
            resolved(reader.resolve_group::<Loop, Member>(held, selection))
        }
        GroupRead::Zone(held, selection) => {
            resolved(reader.resolve_group::<Zone, Member>(held, selection))
        }
    }
}

fn of<G: Typed, E: std::fmt::Debug>(answer: Result<Option<WorthQueryName<G>>, E>) -> GroupAnswer {
    match answer {
        Ok(group) => GroupAnswer::Of(group.map(G::wrap)),
        Err(denial) => GroupAnswer::Refused(format!("{denial:?}")),
    }
}

fn resolved<G: Typed, E: std::fmt::Debug>(
    resolution: Result<WorthQueryDecisionGroupResolution<G>, E>,
) -> GroupAnswer {
    match resolution {
        Err(denial) => GroupAnswer::Refused(format!("{denial:?}")),
        Ok(Ok(ResolvedName::Exact(group))) => GroupAnswer::Exact(G::wrap(group)),
        Ok(Ok(ResolvedName::Plural(groups))) => {
            GroupAnswer::Plural(groups.into_iter().map(G::wrap).collect())
        }
        Ok(Ok(ResolvedName::Retired { .. })) => GroupAnswer::Retired,
        Ok(Err(ExecutionNameResolveDenial::Resolution(NameResolutionDenial::Ambiguous {
            candidates,
        }))) => GroupAnswer::Ambiguous(
            candidates
                .into_iter()
                .map(|candidate| G::wrap(candidate.into_parts().0))
                .collect(),
        ),
        Ok(Err(denial)) => GroupAnswer::Refused(format!("{denial:?}")),
    }
}
