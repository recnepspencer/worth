//! A declared partition of one group kind's changed scope.
//!
//! An operation never mints a group name. It declares which members belong
//! together, each by its persistent name and typed role, and which held
//! groups it dissolves; Relational names the next groups by the kind's
//! declared continuity rule when the commit finalizes.

use std::fmt;
use std::marker::PhantomData;

use super::same_commit::SameCommitNameReference;
use super::{NameRef, WorthQueryName, WorthQueryNameRole, WorthQuerySameCommitName};

/// A group member: a name admitted by an earlier commit, or the name an
/// output this same candidate created and named will be born with.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupMemberName {
    Committed([u8; 32]),
    InCommit(SameCommitNameReference),
}

impl<K> From<WorthQueryName<K>> for GroupMemberName {
    fn from(name: WorthQueryName<K>) -> Self {
        Self::Committed(*name.as_bytes())
    }
}

impl<K> From<&WorthQueryName<K>> for GroupMemberName {
    fn from(name: &WorthQueryName<K>) -> Self {
        Self::Committed(*name.as_bytes())
    }
}

impl<K> From<WorthQuerySameCommitName<K>> for GroupMemberName {
    fn from(name: WorthQuerySameCommitName<K>) -> Self {
        Self::InCommit(name.reference().clone())
    }
}

impl<K> From<&WorthQuerySameCommitName<K>> for GroupMemberName {
    fn from(name: &WorthQuerySameCommitName<K>) -> Self {
        Self::InCommit(name.reference().clone())
    }
}

impl<K> From<NameRef<K>> for GroupMemberName {
    fn from(name: NameRef<K>) -> Self {
        match name {
            NameRef::Committed(name) => name.into(),
            NameRef::InCommit(name) => name.into(),
        }
    }
}

impl<K> From<&NameRef<K>> for GroupMemberName {
    fn from(name: &NameRef<K>) -> Self {
        name.clone().into()
    }
}

/// One group kind `G`'s declared next partition.
///
/// Prior groups of `G` that share a member with a declared group, or lose a
/// member the same commit retires, join the changed scope; their members the
/// partition does not redeclare stay grouped together, unless the partition
/// [`dissolves`](Self::dissolve) that group, in which case they leave `G`.
pub struct WorthQueryGroupPartition<G> {
    dissolved: Vec<[u8; 32]>,
    groups: Vec<Vec<(&'static str, GroupMemberName)>>,
    _kind: PhantomData<fn() -> G>,
}

impl<G> WorthQueryGroupPartition<G> {
    pub fn new() -> Self {
        Self {
            dissolved: Vec::new(),
            groups: Vec::new(),
            _kind: PhantomData,
        }
    }

    /// Declares one next-state group: each member by name, under its typed
    /// role. Order never matters; Relational orders members by name.
    pub fn group<R, M>(mut self, members: impl IntoIterator<Item = (R, M)>) -> Self
    where
        R: WorthQueryNameRole,
        M: Into<GroupMemberName>,
    {
        let members = members
            .into_iter()
            .map(|(role, name)| (role.role_name(), name.into()))
            .collect();
        self.groups.push(members);
        self
    }

    /// Dissolves a held group of `G`: its members this partition does not
    /// redeclare leave the kind, and a group left without members retires.
    pub fn dissolve(mut self, group: &WorthQueryName<G>) -> Self {
        self.dissolved.push(*group.as_bytes());
        self
    }

    /// Whether the partition declares nothing.
    pub fn is_empty(&self) -> bool {
        self.dissolved.is_empty() && self.groups.is_empty()
    }

    /// The declared groups and the dissolved group digests.
    pub fn into_parts(self) -> (Vec<Vec<(&'static str, GroupMemberName)>>, Vec<[u8; 32]>) {
        (self.groups, self.dissolved)
    }
}

impl<G> Default for WorthQueryGroupPartition<G> {
    fn default() -> Self {
        Self::new()
    }
}

impl<G> fmt::Debug for WorthQueryGroupPartition<G> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("WorthQueryGroupPartition")
            .field("kind", &std::any::type_name::<G>())
            .field("groups", &self.groups)
            .field("dissolved", &self.dissolved.len())
            .finish()
    }
}
