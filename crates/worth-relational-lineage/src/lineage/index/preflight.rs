//! Read-only preflight of every cold name-index row a commit's settlement will
//! touch, run during preparation BEFORE durable append so isolated cold page
//! damage refuses the commit cleanly instead of poisoning settlement.

use crate::contracts::aggregate_identity::group_lineage::AggregateGroupLineageEvent as GroupEvent;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::identity::data::record_ids::{EntityId, RelationId};
use crate::contracts::lineage::data::events::{LineageEventRecord, LineageSubject};
use crate::contracts::lineage::data::name_resolution::PersistentNameKey;
use crate::lineage::groups::CommittedGroupTransition;
use crate::lineage::tiered::NamingReadScope;

use super::{LineageHolder, NameEntry, PersistentNameBranchIndex};

fn subject_key(
    subject: LineageSubject,
    name: crate::contracts::lineage::data::naming::PersistentNameIdentity,
) -> PersistentNameKey {
    match subject {
        LineageSubject::Entity => PersistentNameKey::Entity(name),
        LineageSubject::Relation => PersistentNameKey::Relation(name),
    }
}

impl PersistentNameBranchIndex {
    /// Probes every cold row `PersistentNameBranchIndex::apply` would touch
    /// for `events`, `entity_bindings`, `relation_bindings` and
    /// `group_transitions`, latching any storage failure into `scope`.
    ///
    /// Complexity: O(touched names, lineages, groups and touched members,
    /// plus each retired group's members).
    pub(crate) fn preflight_commit(
        &self,
        events: &[LineageEventRecord],
        entity_bindings: &[(EntityId, LineageId)],
        relation_bindings: &[(RelationId, LineageId)],
        group_transitions: &[CommittedGroupTransition],
        scope: &NamingReadScope,
    ) {
        for &(entity, lineage) in entity_bindings {
            let _ = self.lineage_holders.get_in(&lineage, scope);
            let _ = self
                .holder_lineages
                .get_in(&LineageHolder::Entity(entity), scope);
        }
        for &(relation, lineage) in relation_bindings {
            let _ = self.lineage_holders.get_in(&lineage, scope);
            let _ = self
                .holder_lineages
                .get_in(&LineageHolder::Relation(relation), scope);
        }
        for event in events {
            for &lineage in event.sources().iter().chain(event.targets().iter()) {
                if let Some(&holder) = self.lineage_holders.get_in(&lineage, scope) {
                    let _ = self.holder_lineages.get_in(&holder, scope);
                }
                if let Some(&key) = self.lineage_names.get_in(&lineage, scope) {
                    let _ = self.names.get_in(&key, scope);
                }
            }
            if let Some(birth) = event.name_birth() {
                let key = subject_key(event.subject(), birth.identity());
                let _ = self.names.get_in(&key, scope);
            }
            if let Some(kept) = event.kept_name() {
                let key = subject_key(event.subject(), kept);
                let _ = self.names.get_in(&key, scope);
                let opposite = match event.subject() {
                    LineageSubject::Entity => PersistentNameKey::Relation(kept),
                    LineageSubject::Relation => PersistentNameKey::Entity(kept),
                };
                let _ = self.names.get_in(&opposite, scope);
            }
        }
        for committed in group_transitions {
            let transition = &committed.transition;
            // Ingestion reads every event's group entry; only a retirement
            // ends every member's tenure, and an assignment changes only its
            // touched members' tenures.
            for event in &transition.lineage {
                let (first, second) = match *event {
                    GroupEvent::Born { group }
                    | GroupEvent::Revived { group }
                    | GroupEvent::Retired { group } => (group, None),
                    GroupEvent::SplitFrom { fragment, source } => (fragment, Some(source)),
                    GroupEvent::MergedInto { absorbed, keeper } => (absorbed, Some(keeper)),
                };
                for group in std::iter::once(first).chain(second) {
                    let _ = self.names.get_in(&PersistentNameKey::Group(group), scope);
                }
            }
            for &retired in &transition.retired {
                self.preflight_group_and_members(retired, scope);
            }
            for assignment in &transition.assignments {
                let _ = self
                    .names
                    .get_in(&PersistentNameKey::Group(assignment.identity), scope);
                for member in assignment
                    .touched_members()
                    .filter_map(|name| name.entity_name())
                {
                    let _ = self.member_groups.get_in(member, scope);
                }
            }
        }
    }

    fn preflight_group_and_members(
        &self,
        group: worth_foundational::facade::AggregateIdentity,
        scope: &NamingReadScope,
    ) {
        if let Some(entry) = self.names.get_in(&PersistentNameKey::Group(group), scope)
            && let Some(live) = NameEntry::current_live(entry)
        {
            for member in &live.carriers {
                let _ = self.member_groups.get_in(member, scope);
            }
        }
    }

    pub(crate) fn delta_len(&self) -> usize {
        self.names.delta_len()
            + self.lineage_names.delta_len()
            + self.lineage_holders.delta_len()
            + self.holder_lineages.delta_len()
            + self.member_groups.delta_len()
    }

    pub(crate) fn count_diff_against_bounded(
        &self,
        base: &Self,
        has_cold_base: bool,
        budget: usize,
        visited: &mut usize,
    ) -> Option<usize> {
        let mut total = 0usize;
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(total);
                total += self.$field.count_diff_against_bounded(
                    &base.$field,
                    has_cold_base,
                    rem,
                    visited,
                )?;
                if total > budget {
                    return None;
                }
            };
        }
        step!(names);
        step!(lineage_names);
        step!(lineage_holders);
        step!(holder_lineages);
        step!(member_groups);
        Some(total)
    }

    pub(crate) fn diff_encoded_against_bounded(
        &self,
        base: &Self,
        cold_base: Option<(
            &crate::lineage::naming_pages::store::ColdNamingStore,
            &crate::lineage::naming_pages::BranchNamingRoot,
        )>,
        budget: usize,
    ) -> Result<
        Option<Vec<(Vec<u8>, crate::lineage::naming_pages::builder::PageChange)>>,
        crate::contracts::storage::data::storage_unavailable::RelationalStorageUnavailableCause,
    > {
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
        step!(names);
        step!(lineage_names);
        step!(lineage_holders);
        step!(holder_lineages);
        step!(member_groups);
        Ok(Some(out))
    }

    pub(crate) fn encoded_puts_bounded(
        &self,
        budget: usize,
    ) -> Option<Vec<(Vec<u8>, crate::lineage::naming_pages::builder::PageChange)>> {
        let mut out = Vec::new();
        macro_rules! step {
            ($field:ident) => {
                let rem = budget.saturating_sub(out.len());
                out.extend(self.$field.encoded_puts_bounded(rem)?);
            };
        }
        step!(names);
        step!(lineage_names);
        step!(lineage_holders);
        step!(holder_lineages);
        step!(member_groups);
        Some(out)
    }
}
