//! Authoritative branch-local persistent-name basis derived solely from
//! Relational's committed `LineageEventRecord`s, entity `LineageNode`s, and
//! persisted relation lineage bindings.
//!
//! Commit finalization reads this derived basis directly from
//! `LineageSubsystem` rather than trusting caller-supplied maps, and
//! reconstruction (`LineageState::rebuild_derived_indexes`) rebuilds it from
//! canonical events and holder bindings alone.

use std::collections::BTreeMap;

use crate::history::data::BranchId;
use crate::identity::data::{LineageId, RelationId};
use crate::lineage::data::{
    LineageEventKind, LineageEventRecord, LineageNode, LineageSubject, PersistentNameHolder,
    PersistentNameIdentity,
};
use crate::runtime::state::subsystems::RuntimeOwnedState;

use super::LineageState;

/// Authoritative snapshot of live persistent-name holders, live record
/// lineages, and previously retired persistent-name lineages for one branch.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct AuthoritativeLineageBasis {
    pub(crate) live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    pub(crate) live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    pub(crate) retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
}

/// Incrementally maintained, rebuildable per-branch naming basis inside
/// `LineageState`.
#[derive(Debug, Clone, Default)]
pub(super) struct AuthoritativeBranchNamingState {
    lineage_holders: BTreeMap<LineageId, PersistentNameHolder>,
    holder_names: BTreeMap<PersistentNameHolder, PersistentNameIdentity>,
    live_holders: BTreeMap<PersistentNameIdentity, PersistentNameHolder>,
    live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    retired_lineages: BTreeMap<PersistentNameIdentity, (LineageSubject, LineageId)>,
}

impl AuthoritativeBranchNamingState {
    pub(super) fn snapshot(&self) -> AuthoritativeLineageBasis {
        AuthoritativeLineageBasis {
            live_holders: self.live_holders.clone(),
            live_lineages: self.live_lineages.clone(),
            retired_lineages: self.retired_lineages.clone(),
        }
    }

    pub(super) fn live_holders(&self) -> &BTreeMap<PersistentNameIdentity, PersistentNameHolder> {
        &self.live_holders
    }

    pub(super) fn apply_event(
        &mut self,
        event: &LineageEventRecord,
        entity_nodes: &BTreeMap<LineageId, LineageNode>,
        relation_holders: &BTreeMap<LineageId, RelationId>,
    ) {
        match event.kind() {
            LineageEventKind::Create | LineageEventKind::Restore => {
                let Some(&target_lineage) = event.targets().first() else {
                    return;
                };
                let Some(holder) =
                    resolve_holder(event.subject(), target_lineage, entity_nodes, relation_holders)
                else {
                    return;
                };
                self.lineage_holders.insert(target_lineage, holder.clone());
                self.live_lineages.insert(holder.clone(), target_lineage);
                if let Some(birth) = event.name_birth() {
                    let identity = birth.identity();
                    self.holder_names.insert(holder.clone(), identity);
                    self.live_holders.insert(identity, holder);
                    self.retired_lineages.remove(&identity);
                } else if let Some(kept) = event.kept_name() {
                    self.holder_names.insert(holder.clone(), kept);
                    self.live_holders.insert(kept, holder);
                    self.retired_lineages.remove(&kept);
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
                    let Some(holder) = self.lineage_holders.get(&source).cloned().or_else(|| {
                        resolve_holder(event.subject(), source, entity_nodes, relation_holders)
                    }) else {
                        continue;
                    };
                    self.live_lineages.remove(&holder);
                    if let Some(&name) = self.holder_names.get(&holder)
                        && self.live_holders.get(&name) == Some(&holder)
                    {
                        self.live_holders.remove(&name);
                        self.retired_lineages
                            .insert(name, (event.subject(), source));
                    }
                }
            }
        }
    }
}

fn resolve_holder(
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

/// Narrow read-only binding handed to `RelationalPreparationRuntime` so commit
/// preparation derives live/retired persistent-name sets exclusively from
/// Relational's own authoritative `LineageSubsystem`.
#[derive(Debug)]
pub(crate) struct LineageNamingBasisBinding {
    state: RuntimeOwnedState<LineageState>,
}

impl Clone for LineageNamingBasisBinding {
    fn clone(&self) -> Self {
        Self {
            state: self.state.share(),
        }
    }
}

impl LineageNamingBasisBinding {
    pub(super) fn new(state: RuntimeOwnedState<LineageState>) -> Self {
        Self { state }
    }

    pub(crate) fn authoritative_basis(&self, branch_id: &BranchId) -> AuthoritativeLineageBasis {
        self.state.read().authoritative_branch_naming_basis(branch_id)
    }
}
