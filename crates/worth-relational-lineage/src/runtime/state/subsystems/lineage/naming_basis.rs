//! Authoritative branch-local persistent-name basis derived solely from
//! Relational's committed `LineageEventRecord`s, entity `LineageNode`s,
//! persisted relation lineage bindings, and fork ancestry.
//!
//! Commit finalization queries this derived basis through
//! a branch naming guard implementing `LineageBasisView` with `O(log N)`
//! point lookups and zero map clones. Every table is tiered: a reopened
//! runtime faults only the rows a lookup or an applied event touches.

use std::collections::BTreeMap;

use crate::contracts::history::data::coordinates::CommitId;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::RelationId;
use crate::contracts::lineage::data::events::{
    LineageEventKind, LineageEventRecord, LineageSubject,
};
use crate::contracts::lineage::data::graph::LineageNode;
use crate::contracts::lineage::data::naming::{PersistentNameHolder, PersistentNameIdentity};
use crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause as Cause;
use crate::lineage::naming_pages::tables;
use crate::lineage::tiered::{NamingBaseLink, NamingReadScope, RowRef, TieredMap};

/// Snapshot of live persistent-name holders, live record lineages, and
/// previously retired persistent-name lineages for one branch (test-only
/// basis-equality helper; live commits borrow via `BranchNamingBasisGuard`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthoritativeLineageBasis {
    pub live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    pub retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
}

/// Why a branch's naming basis cannot answer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NamingBasisUnavailable {
    /// A cold naming page it needed is missing or damaged.
    Storage(Cause),
    /// It cannot be derived from the resident state: the reopened checkpoint
    /// recorded it unavailable, or a fork needed a rebuild from history the
    /// cold runtime does not hold.
    Unrecoverable,
}

/// A retired name's subject, lineage and retirement commit.
pub(super) type RetiredLineage = (LineageSubject, LineageId, CommitId);

crate::lineage::clone_fences::assert_not_clone!(AuthoritativeBranchNamingState);

/// Incrementally maintained per-branch naming basis inside `LineageState`.
///
/// Deliberately not `Clone`: [`Self::fork`] shares tables copy-on-write.
#[derive(Debug)]
pub(crate) struct AuthoritativeBranchNamingState {
    pub(super) lineage_holders: TieredMap<LineageId, PersistentNameHolder>,
    pub(super) holder_names: TieredMap<PersistentNameHolder, PersistentNameIdentity>,
    pub(super) live_holders: TieredMap<PersistentNameIdentity, PersistentNameHolder>,
    pub(super) live_lineages: TieredMap<PersistentNameHolder, LineageId>,
    pub(super) retired_lineages: TieredMap<PersistentNameIdentity, RetiredLineage>,
    pub(super) earliest_event_commit: Option<CommitId>,
    pub(super) latest_event_commit: Option<CommitId>,
}

impl AuthoritativeBranchNamingState {
    pub(super) fn over(
        base: Option<NamingBaseLink>,
        earliest: Option<CommitId>,
        latest: Option<CommitId>,
    ) -> Self {
        Self {
            lineage_holders: TieredMap::over(tables::LINEAGE_HOLDER, base.clone()),
            holder_names: TieredMap::over(tables::HOLDER_NAME, base.clone()),
            live_holders: TieredMap::over(tables::LIVE_HOLDER, base.clone()),
            live_lineages: TieredMap::over(tables::LIVE_LINEAGE, base.clone()),
            retired_lineages: TieredMap::over(tables::RETIRED_LINEAGE, base),
            earliest_event_commit: earliest,
            latest_event_commit: latest,
        }
    }

    /// Complexity: O(1): tables are shared copy-on-write.
    pub(super) fn fork(&self, base: Option<NamingBaseLink>) -> Self {
        Self {
            lineage_holders: self.lineage_holders.fork(base.clone()),
            holder_names: self.holder_names.fork(base.clone()),
            live_holders: self.live_holders.fork(base.clone()),
            live_lineages: self.live_lineages.fork(base.clone()),
            retired_lineages: self.retired_lineages.fork(base),
            earliest_event_commit: self.earliest_event_commit,
            latest_event_commit: self.latest_event_commit,
        }
    }

    /// Rebinds every table cleanly onto an acknowledged cold base, dropping
    /// resident deltas.
    pub(super) fn rebind_clean(&mut self, base: NamingBaseLink) {
        self.lineage_holders.rebind_clean(base.clone());
        self.holder_names.rebind_clean(base.clone());
        self.live_holders.rebind_clean(base.clone());
        self.live_lineages.rebind_clean(base.clone());
        self.retired_lineages.rebind_clean(base);
    }

    /// Rebinds existing cold bases onto `source` (and optional `checkpointed_root`).
    pub(super) fn rebind_source(
        &mut self,
        source: &std::sync::Arc<dyn crate::lineage::tiered::ColdNamingSource>,
        root: Option<&crate::contracts::history::data::coordinates::BranchId>,
    ) {
        self.lineage_holders.rebind_source(source, root);
        self.holder_names.rebind_source(source, root);
        self.live_holders.rebind_source(source, root);
        self.live_lineages.rebind_source(source, root);
        self.retired_lineages.rebind_source(source, root);
    }

    /// Probes every cold row `apply_event` will read for `events`, latching
    /// any storage failure into `scope` before durable publication.
    pub(super) fn preflight_events(&self, events: &[LineageEventRecord], scope: &NamingReadScope) {
        for event in events {
            if matches!(
                event.kind(),
                LineageEventKind::Retire
                    | LineageEventKind::Replace
                    | LineageEventKind::Split
                    | LineageEventKind::Merge
            ) {
                for &source in event.sources() {
                    if event.targets().contains(&source) {
                        continue;
                    }
                    if let Some(holder) = self.lineage_holders.get_in(&source, scope)
                        && let Some(&name) = self.holder_names.get_in(holder, scope)
                    {
                        let _ = self.live_holders.get_in(&name, scope);
                    }
                }
            }
        }
    }

    /// The actual cold base link backing this naming basis, if any.
    pub(super) fn base(&self) -> Option<&NamingBaseLink> {
        self.live_holders.base()
    }

    /// Every table's resident changes, for capture.
    pub(super) fn encoded_changes(&self) -> Vec<(Vec<u8>, Option<Vec<u8>>)> {
        let mut changes = self.lineage_holders.encoded_changes();
        changes.extend(self.holder_names.encoded_changes());
        changes.extend(self.live_holders.encoded_changes());
        changes.extend(self.live_lineages.encoded_changes());
        changes.extend(self.retired_lineages.encoded_changes());
        changes
    }

    pub(super) fn delta_len(&self) -> usize {
        self.lineage_holders.delta_len()
            + self.holder_names.delta_len()
            + self.live_holders.delta_len()
            + self.live_lineages.delta_len()
            + self.retired_lineages.delta_len()
    }

    pub(super) fn diff_encoded_against_bounded(
        &self,
        base: &Self,
        cold_base: Option<(
            &crate::lineage::naming_pages::store::ColdNamingStore,
            &crate::lineage::naming_pages::BranchNamingRoot,
        )>,
        budget: usize,
    ) -> Result<Option<Vec<(Vec<u8>, crate::lineage::naming_pages::builder::PageChange)>>, Cause>
    {
        let mut out = Vec::new();
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(out.len());
                let Some(part) =
                    self.$field
                        .diff_encoded_against_bounded(&base.$field, cold_base, rem)?
                else {
                    return Ok(None);
                };
                out.extend(part);
            };
        }
        step!(lineage_holders);
        step!(holder_names);
        step!(live_holders);
        step!(live_lineages);
        step!(retired_lineages);
        Ok(Some(out))
    }

    /// Every live row across all naming-basis tables merged against `store`,
    /// for mixed-base branch capture.
    pub(super) fn collect_encoded_in(
        &self,
        store: Option<&crate::lineage::naming_pages::store::ColdNamingStore>,
    ) -> Result<Vec<(Vec<u8>, Option<Vec<u8>>)>, Cause> {
        let mut rows = self.lineage_holders.collect_encoded_in(store)?;
        rows.extend(self.holder_names.collect_encoded_in(store)?);
        rows.extend(self.live_holders.collect_encoded_in(store)?);
        rows.extend(self.live_lineages.collect_encoded_in(store)?);
        rows.extend(self.retired_lineages.collect_encoded_in(store)?);
        Ok(rows)
    }

    /// Complexity: O(N) full scan; audits and courts only.
    pub(crate) fn snapshot(&self) -> Result<AuthoritativeLineageBasis, Cause> {
        Ok(AuthoritativeLineageBasis {
            live_holders: self.live_holders.collect_merged()?.into_iter().collect(),
            live_lineages: self.live_lineages.collect_merged()?.into_iter().collect(),
            retired_lineages: self
                .retired_lineages
                .collect_merged()?
                .into_iter()
                .map(|(name, (subject, lineage, _))| (name, (subject, lineage)))
                .collect(),
        })
    }

    pub(super) fn live_holders(
        &self,
    ) -> Result<BTreeMap<PersistentNameIdentity, PersistentNameHolder>, Cause> {
        Ok(self.live_holders.collect_merged()?.into_iter().collect())
    }

    pub(super) fn earliest_event_commit(&self) -> Option<CommitId> {
        self.earliest_event_commit
    }

    pub(super) fn latest_event_commit(&self) -> Option<CommitId> {
        self.latest_event_commit
    }

    pub(super) fn live_holder_in<'s>(
        &'s self,
        id: &PersistentNameIdentity,
        scope: &'s NamingReadScope,
    ) -> Option<&'s PersistentNameHolder> {
        self.live_holders.get_in(id, scope)
    }

    pub(super) fn live_lineage_in(
        &self,
        holder: &PersistentNameHolder,
        scope: &NamingReadScope,
    ) -> Option<LineageId> {
        self.live_lineages.get_in(holder, scope).copied()
    }

    pub(super) fn retired_lineage_in(
        &self,
        id: &PersistentNameIdentity,
        scope: &NamingReadScope,
    ) -> Option<(LineageSubject, LineageId)> {
        self.retired_lineages
            .get_in(id, scope)
            .map(|(subject, lineage, _)| (*subject, *lineage))
    }

    pub(super) fn holder_name_in(
        &self,
        holder: &PersistentNameHolder,
        scope: &NamingReadScope,
    ) -> Option<PersistentNameIdentity> {
        self.holder_names.get_in(holder, scope).copied()
    }

    pub(super) fn holder_and_name_for_lineage(
        &self,
        lineage_id: LineageId,
    ) -> Result<Option<(PersistentNameHolder, PersistentNameIdentity)>, Cause> {
        let Some(holder) = self
            .lineage_holders
            .get(&lineage_id)?
            .map(RowRef::into_owned)
        else {
            return Ok(None);
        };
        let name = self.holder_names.get(&holder)?.map(|name| *name);
        Ok(name.map(|name| (holder, name)))
    }

    /// Applies one event, faulting only the rows it names.
    ///
    /// Complexity: O(sources + targets) point lookups.
    pub(super) fn apply_event(
        &mut self,
        event: &LineageEventRecord,
        publication_commit_id: CommitId,
        entity_nodes: &BTreeMap<LineageId, LineageNode>,
        relation_holders: &BTreeMap<LineageId, RelationId>,
    ) -> Result<(), Cause> {
        self.earliest_event_commit = Some(
            self.earliest_event_commit
                .map_or(publication_commit_id, |prev| {
                    prev.min(publication_commit_id)
                }),
        );
        self.latest_event_commit = Some(
            self.latest_event_commit
                .map_or(publication_commit_id, |prev| {
                    prev.max(publication_commit_id)
                }),
        );
        match event.kind() {
            LineageEventKind::Create | LineageEventKind::Restore => {
                let Some(&target_lineage) = event.targets().first() else {
                    return Ok(());
                };
                let Some(holder) = resolve_holder(
                    event.subject(),
                    target_lineage,
                    entity_nodes,
                    relation_holders,
                ) else {
                    return Ok(());
                };
                self.lineage_holders.insert(target_lineage, holder.clone());
                self.live_lineages.insert(holder.clone(), target_lineage);
                let named = event
                    .name_birth()
                    .map(|birth| birth.identity())
                    .or_else(|| event.kept_name());
                if let Some(identity) = named {
                    self.holder_names.insert(holder.clone(), identity);
                    self.live_holders.insert(identity, holder);
                    self.retired_lineages.remove_blind(&identity);
                }
            }
            LineageEventKind::Retire
            | LineageEventKind::Replace
            | LineageEventKind::Split
            | LineageEventKind::Merge => {
                for &source in event.sources() {
                    if event.targets().contains(&source) {
                        continue;
                    }
                    let holder = match self.lineage_holders.get(&source)? {
                        Some(holder) => holder.into_owned(),
                        None => match resolve_holder(
                            event.subject(),
                            source,
                            entity_nodes,
                            relation_holders,
                        ) {
                            Some(holder) => holder,
                            None => continue,
                        },
                    };
                    self.live_lineages.remove_blind(&holder);
                    let name = self.holder_names.get(&holder)?.map(|name| *name);
                    if let Some(name) = name
                        && self.live_holders.get(&name)?.as_deref() == Some(&holder)
                    {
                        self.live_holders.remove_blind(&name);
                        self.retired_lineages
                            .insert(name, (event.subject(), source, publication_commit_id));
                    }
                }
            }
        }
        Ok(())
    }
}

pub(super) fn resolve_holder(
    subject: LineageSubject,
    lineage_id: LineageId,
    entity_nodes: &BTreeMap<LineageId, LineageNode>,
    relation_holders: &BTreeMap<LineageId, RelationId>,
) -> Option<PersistentNameHolder> {
    match subject {
        LineageSubject::Entity => entity_nodes
            .get(&lineage_id)
            .map(|node| PersistentNameHolder::Entity(node.entity_id())),
        LineageSubject::Relation => relation_holders
            .get(&lineage_id)
            .copied()
            .map(PersistentNameHolder::Relation),
    }
}
