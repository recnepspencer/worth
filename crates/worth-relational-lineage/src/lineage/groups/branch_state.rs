//! One branch's live groups per installed scheme, derived by applying each
//! commit's [`CommittedGroupTransition`]s in publication order.
//!
//! Application is structural verification, never naming: it refuses any
//! transition that retires, keeps or revives a group the branch does not hold
//! in that posture, overlaps two live groups' members, or carries lineage
//! events its own outcome does not imply. Keeper choice and identity minting
//! belong to the group naming authority, which recovery re-runs separately.
//!
//! Each scheme's live group headers, member rows, retired names and member
//! owners are tiered tables: a reopened runtime faults, and a commit
//! writes, only the rows the transition touches.

use std::collections::{BTreeMap, BTreeSet};

use worth_foundational::facade::AggregateIdentity;

use crate::contracts::aggregate_identity::transition::group::model::AggregateContinuityRule;
use crate::contracts::aggregate_identity::transition::group::model::CanonicalGroupMemberIdentity;
use crate::contracts::aggregate_identity::transition::group::state_view::AggregateGroupHeader;
use crate::contracts::graph_member::NativeMemberKey;
use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::tables;
use crate::lineage::tiered::{NamingBaseLink, TieredMap};

use super::group_rows::{GroupMemberKey, GroupOwnerRow};
use super::CommittedGroupTransition;

/// Why a committed group transition cannot apply to a branch's group state.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GroupStateDenial {
    /// One commit carries two transitions for one scheme.
    DuplicateScheme { scheme_id: String },
    /// The transition's continuity rule differs from the scheme's history.
    RuleChanged { scheme_id: String },
    /// The named group is not in the posture the transition requires.
    GroupPosture {
        scheme_id: String,
        group: AggregateIdentity,
        detail: &'static str,
    },
    /// A next-state group has no members.
    EmptyGroup {
        scheme_id: String,
        group: AggregateIdentity,
    },
    /// One member would belong to two live groups of the scheme.
    MemberOverlap {
        scheme_id: String,
        member: CanonicalGroupMemberIdentity,
    },
    /// The lineage events disagree with the transition's own outcome.
    EventMismatch {
        scheme_id: String,
        detail: &'static str,
    },
    /// A cold group page the state needed is missing or damaged.
    StorageUnavailable(Cause),
    /// The reopened checkpoint recorded this branch's group state unavailable.
    CheckpointedUnavailable,
}

impl From<Cause> for GroupStateDenial {
    fn from(cause: Cause) -> Self {
        Self::StorageUnavailable(cause)
    }
}

pub(crate) type SchemeKey<T> = (String, T);

crate::lineage::clone_fences::assert_not_clone!(SchemeGroups);
crate::lineage::clone_fences::assert_not_clone!(BranchGroupState);
crate::lineage::clone_fences::assert_not_clone!(
    crate::contracts::aggregate_identity::transition::group::model::AggregatePriorGroup
);

/// One installed scheme's live group headers, member rows, retired names
/// and member owners.
///
/// Not `Clone`: it holds every live group's members. Forks share tables
/// copy-on-write through [`BranchGroupState::fork`].
#[derive(Debug, Eq, PartialEq)]
pub struct SchemeGroups {
    pub(super) id: String,
    pub(super) rule: AggregateContinuityRule,
    pub(super) live: TieredMap<SchemeKey<AggregateIdentity>, AggregateGroupHeader>,
    pub(super) members: TieredMap<SchemeKey<GroupMemberKey>, NativeMemberKey>,
    pub(super) retired: TieredMap<SchemeKey<AggregateIdentity>, ()>,
    pub(super) member_owner: TieredMap<SchemeKey<CanonicalGroupMemberIdentity>, GroupOwnerRow>,
}

impl SchemeGroups {
    pub(crate) fn over(
        id: String,
        rule: AggregateContinuityRule,
        base: Option<NamingBaseLink>,
    ) -> Self {
        Self {
            id,
            rule,
            live: TieredMap::over(tables::GROUP_LIVE, base.clone()),
            members: TieredMap::over(tables::GROUP_MEMBERS, base.clone()),
            retired: TieredMap::over(tables::GROUP_RETIRED, base.clone()),
            member_owner: TieredMap::over(tables::GROUP_MEMBER_OWNER, base),
        }
    }

    fn fork(&self, base: Option<NamingBaseLink>) -> Self {
        Self {
            id: self.id.clone(),
            rule: self.rule,
            live: self.live.fork(base.clone()),
            members: self.members.fork(base.clone()),
            retired: self.retired.fork(base.clone()),
            member_owner: self.member_owner.fork(base),
        }
    }

    pub const fn rule(&self) -> AggregateContinuityRule {
        self.rule
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub(super) fn key<T>(&self, value: T) -> SchemeKey<T> {
        (self.id.clone(), value)
    }

    fn rebind_clean(&mut self, base: NamingBaseLink) {
        self.live.rebind_clean(base.clone());
        self.members.rebind_clean(base.clone());
        self.retired.rebind_clean(base.clone());
        self.member_owner.rebind_clean(base);
    }

    fn rebind_source(
        &mut self,
        source: &std::sync::Arc<dyn crate::lineage::tiered::ColdNamingSource>,
        root: Option<&crate::contracts::history::data::coordinates::BranchId>,
    ) {
        self.live.rebind_source(source, root);
        self.members.rebind_source(source, root);
        self.retired.rebind_source(source, root);
        self.member_owner.rebind_source(source, root);
    }

    pub(crate) fn base(&self) -> Option<&NamingBaseLink> {
        self.live.base()
    }

    /// Every table of this scheme, for capture.
    pub(crate) fn encoded_changes(&self) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
        let mut changes = self.live.encoded_changes();
        changes.extend(self.members.encoded_changes());
        changes.extend(self.retired.encoded_changes());
        changes.extend(self.member_owner.encoded_changes());
        changes
    }

    /// Every live row of this scheme merged against `store`, for mixed-base capture.
    pub(crate) fn collect_encoded_in(
        &self,
        store: Option<&crate::lineage::naming_pages::store::ColdNamingStore>,
    ) -> Result<Vec<(Vec<u8>, Option<Vec<u8>>)>, Cause> {
        let mut rows = self.live.collect_encoded_in(store)?;
        rows.extend(self.members.collect_encoded_in(store)?);
        rows.extend(self.retired.collect_encoded_in(store)?);
        rows.extend(self.member_owner.collect_encoded_in(store)?);
        Ok(rows)
    }
}

/// Every installed scheme's groups on one branch.
///
/// Deliberately not `Clone`: head reads borrow the maintained state and forks
/// share tables copy-on-write through [`Self::fork`].
#[derive(Debug, Default, Eq, PartialEq)]
pub struct BranchGroupState {
    pub(super) schemes: BTreeMap<String, SchemeGroups>,
    pub(super) earliest_commit: Option<CommitId>,
    pub(super) latest_commit: Option<CommitId>,
    pub(super) base: Option<NamingBaseLink>,
}

impl BranchGroupState {
    /// The state a checkpoint recorded: scheme headers resident, tables cold.
    pub(crate) fn over(
        schemes: impl IntoIterator<Item = (String, AggregateContinuityRule)>,
        earliest_commit: Option<CommitId>,
        latest_commit: Option<CommitId>,
        base: Option<NamingBaseLink>,
    ) -> Self {
        Self {
            schemes: schemes
                .into_iter()
                .map(|(id, rule)| (id.clone(), SchemeGroups::over(id, rule, base.clone())))
                .collect(),
            earliest_commit,
            latest_commit,
            base,
        }
    }

    /// An empty state whose tables read through `base`.
    pub(crate) fn empty_over(base: Option<NamingBaseLink>) -> Self {
        Self::over(std::iter::empty(), None, None, base)
    }

    pub fn scheme(&self, scheme_id: &str) -> Option<&SchemeGroups> {
        self.schemes.get(scheme_id)
    }

    /// This state for a fork, sharing every table copy-on-write and preserving
    /// whether each individual scheme is backed by `base` or newly resident.
    ///
    /// Complexity: O(schemes).
    pub(crate) fn fork(&self, base: Option<NamingBaseLink>) -> Self {
        Self {
            schemes: self
                .schemes
                .iter()
                .map(|(id, groups)| {
                    let scheme_base = if groups.base().is_some() {
                        base.clone()
                    } else {
                        None
                    };
                    (id.clone(), groups.fork(scheme_base))
                })
                .collect(),
            earliest_commit: self.earliest_commit,
            latest_commit: self.latest_commit,
            base,
        }
    }

    /// Rebinds every scheme table cleanly onto an acknowledged cold base.
    pub(crate) fn rebind_clean(&mut self, base: NamingBaseLink) {
        for groups in self.schemes.values_mut() {
            groups.rebind_clean(base.clone());
        }
        self.base = Some(base);
    }

    /// Rebinds all existing cold bases onto `source` (and optional `checkpointed_root`).
    pub(crate) fn rebind_source(
        &mut self,
        source: &std::sync::Arc<dyn crate::lineage::tiered::ColdNamingSource>,
        root: Option<&crate::contracts::history::data::coordinates::BranchId>,
    ) {
        if let Some(base) = &mut self.base {
            *base = base.rebind_source(std::sync::Arc::clone(source), root);
        }
        for groups in self.schemes.values_mut() {
            groups.rebind_source(source, root);
        }
    }

    /// The actual cold base link backing this branch group state, if any.
    pub(crate) fn base(&self) -> Option<&NamingBaseLink> {
        self.base.as_ref()
    }

    /// The earliest commit whose transitions this state applied.
    pub const fn earliest_commit(&self) -> Option<CommitId> {
        self.earliest_commit
    }

    /// The newest commit whose transitions this state applied.
    pub const fn latest_commit(&self) -> Option<CommitId> {
        self.latest_commit
    }

    /// Installed scheme headers in id order.
    pub(crate) fn scheme_headers(&self) -> impl Iterator<Item = (&str, AggregateContinuityRule)> {
        self.schemes
            .iter()
            .map(|(id, groups)| (id.as_str(), groups.rule))
    }

    pub(crate) fn schemes_raw(&self) -> impl Iterator<Item = &SchemeGroups> {
        self.schemes.values()
    }

    /// Verifies one commit's transitions against this state without applying
    /// them.
    ///
    /// Complexity: as [`Self::apply`].
    pub fn verify(&self, transitions: &[CommittedGroupTransition]) -> Result<(), GroupStateDenial> {
        let mut seen = BTreeSet::new();
        for committed in transitions {
            let scheme_id = committed.scheme_id();
            if !seen.insert(scheme_id) {
                return Err(GroupStateDenial::DuplicateScheme {
                    scheme_id: scheme_id.to_owned(),
                });
            }
            let empty;
            let groups = match self.schemes.get(scheme_id) {
                Some(groups) => groups,
                None => {
                    empty = SchemeGroups::over(scheme_id.to_owned(), committed.scheme.rule(), None);
                    &empty
                }
            };
            super::group_verify::verify(groups, committed)?;
        }
        Ok(())
    }

    /// Applies one commit's transitions atomically: every transition is
    /// verified against the state before any of them changes it.
    ///
    /// Complexity: O(T log G) for T touched groups and members and G live
    /// groups and members of the touched schemes.
    pub fn apply(
        &mut self,
        commit: CommitId,
        transitions: &[CommittedGroupTransition],
    ) -> Result<(), GroupStateDenial> {
        self.verify(transitions)?;
        for committed in transitions {
            self.schemes
                .entry(committed.scheme_id().to_owned())
                .or_insert_with(|| {
                    SchemeGroups::over(
                        committed.scheme_id().to_owned(),
                        committed.scheme.rule(),
                        None,
                    )
                })
                .install(committed)?;
        }
        if !transitions.is_empty() {
            self.earliest_commit = Some(self.earliest_commit.map_or(commit, |at| at.min(commit)));
            self.latest_commit = Some(self.latest_commit.map_or(commit, |at| at.max(commit)));
        }
        Ok(())
    }
}

/// A branch's group state as of one commit: the maintained state itself
/// when no later group lineage applies to it, else one rebuilt for that
/// historical commit.
#[derive(Debug)]
pub enum GroupStateAt<'state> {
    Head(&'state BranchGroupState),
    Rebuilt(BranchGroupState),
}

impl std::ops::Deref for GroupStateAt<'_> {
    type Target = BranchGroupState;

    fn deref(&self) -> &BranchGroupState {
        match self {
            Self::Head(state) => state,
            Self::Rebuilt(state) => state,
        }
    }
}
