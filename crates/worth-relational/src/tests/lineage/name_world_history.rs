//! History and continuity extensions for [`LineageWorld`]: in-commit
//! regeneration and cross-commit restore (explicit or birth-classified).

use std::collections::BTreeSet;

use crate::facade::identity::LineageId;
use crate::facade::lineage::{
    LineageEventKind, LineageMergeKeeper, LineagePostureClaim, LineageSplitChild, LineageSubject,
    PersistentNameBirthRequest, PersistentNameCommittedBatch, PersistentNameHolder,
    PersistentNameKey,
};

use super::name_world::{record_target, subject_of, Keep, LineageWorld, Name, Subject};

impl LineageWorld {
    pub(super) fn retired_names(&self) -> Vec<(Name, LineageSubject)> {
        self.retired_lineages
            .iter()
            .map(|(n, (subj, _))| (*n, *subj))
            .collect()
    }

    pub(super) fn regenerate(&mut self, name: Name) -> PersistentNameHolder {
        let old = self.live_holders[&name].clone();
        self.retire_record(&old);
        let fresh = self.new_record(subject_of(&old));
        let birth = self.replay_birth(name, &fresh);
        self.postures.push(LineagePostureClaim::Retire {
            target: record_target(&old),
            name,
        });
        self.postures.push(LineagePostureClaim::Create {
            target: record_target(&fresh),
            birth,
        });
        fresh
    }

    pub(super) fn restore(&mut self, name: Name, via_birth: bool) -> PersistentNameHolder {
        let subject = match self.retired_lineages[&name].0 {
            LineageSubject::Entity => Subject::Entity,
            LineageSubject::Relation => Subject::Relation,
        };
        let fresh = self.new_record(subject);
        let target = record_target(&fresh);
        if via_birth {
            let birth = self.replay_birth(name, &fresh);
            self.postures
                .push(LineagePostureClaim::Create { target, birth });
        } else {
            self.postures
                .push(LineagePostureClaim::Restore { target, name });
        }
        fresh
    }

    pub(super) fn retired_children_of(&self, parent: Name) -> Vec<Name> {
        let parent_subj = match self.live_holders.get(&parent) {
            Some(PersistentNameHolder::Relation(_)) => LineageSubject::Relation,
            _ => LineageSubject::Entity,
        };
        self.retired_lineages
            .iter()
            .filter_map(|(&name, &(subj, _))| {
                let parts = self.birth_parts.get(&name)?;
                (subj == parent_subj
                    && parts.parent_identities() == [parent]
                    && !parts.canonical_discriminator().is_empty())
                .then_some(name)
            })
            .collect()
    }

    pub(super) fn resplit_replaying_retired(
        &mut self,
        parent: Name,
        retired_children: &[Name],
    ) -> Vec<PersistentNameHolder> {
        let parent_holder = self.live_holders[&parent].clone();
        self.retire_record(&parent_holder);
        let subject = subject_of(&parent_holder);
        let mut children = Vec::with_capacity(retired_children.len());
        let mut holders = Vec::with_capacity(retired_children.len());
        for &child_name in retired_children {
            let holder = self.new_record(subject);
            let birth = self.replay_birth(child_name, &holder);
            children.push(LineageSplitChild {
                target: record_target(&holder),
                birth,
            });
            holders.push(holder);
        }
        self.postures.push(LineagePostureClaim::Split {
            parent_target: record_target(&parent_holder),
            parent_name: parent,
            children,
        });
        holders
    }

    pub(super) fn retired_merge_of(&self, parents: &[Name]) -> Option<Name> {
        let mut sorted = parents.to_vec();
        sorted.sort_unstable();
        let subj = match self.live_holders.get(&parents[0]) {
            Some(PersistentNameHolder::Relation(_)) => LineageSubject::Relation,
            _ => LineageSubject::Entity,
        };
        self.retired_lineages.iter().find_map(|(&name, &(s, _))| {
            let parts = self.birth_parts.get(&name)?;
            (s == subj && parts.parent_identities() == sorted.as_slice()).then_some(name)
        })
    }

    pub(super) fn remerge_mint_new_replaying_retired(
        &mut self,
        parents: &[Name],
        retired_merged: Name,
    ) -> PersistentNameHolder {
        let listed: Vec<_> = parents
            .iter()
            .map(|name| (self.live_holders[name].clone(), *name))
            .collect();
        for (holder, _) in &listed {
            self.retire_record(holder);
        }
        let subject = subject_of(&listed[0].0);
        let target = self.new_record(subject);
        let birth = self.replay_birth(retired_merged, &target);
        self.postures.push(LineagePostureClaim::Merge {
            parents: listed
                .iter()
                .map(|(holder, name)| (record_target(holder), *name))
                .collect(),
            target: record_target(&target),
            keeper: LineageMergeKeeper::MintNew { birth },
        });
        target
    }

    fn replay_birth(
        &self,
        name: Name,
        holder: &PersistentNameHolder,
    ) -> PersistentNameBirthRequest {
        let parts = &self.birth_parts[&name];
        PersistentNameBirthRequest {
            scheme_id: parts.scheme_id().to_owned(),
            occurrence_key: parts.occurrence_key().to_owned(),
            role_path: parts.role_path().to_owned(),
            parent_identities: parts.parent_identities().to_vec(),
            referenced_names: Vec::new(),
            canonical_discriminator: parts.canonical_discriminator().to_vec(),
            holder: holder.clone(),
        }
    }
}

pub(super) struct RestoreTraversalCase {
    pub(super) label: &'static str,
    pub(super) log: Vec<PersistentNameCommittedBatch>,
    pub(super) name_key: PersistentNameKey,
    pub(super) start_lineage: LineageId,
    pub(super) restored_lineage: LineageId,
    pub(super) restore_event_id: u64,
    pub(super) retired_lineages: BTreeSet<LineageId>,
}

impl RestoreTraversalCase {
    pub(super) fn assert_agrees_with_resolution(
        &self,
        resolution: &crate::facade::lineage::HistoricalLineageResolution,
    ) {
        use super::name_index_fixtures::{exact, rebuild};
        use crate::facade::lineage::{PersistentNameResolution, PersistentNameTarget};

        assert_eq!(resolution.start, self.start_lineage, "{}", self.label);
        assert!(
            resolution.resolved.contains(&self.restored_lineage),
            "{}: resolved {:?} missing restored {:?}",
            self.label,
            resolution.resolved,
            self.restored_lineage
        );
        assert!(
            resolution
                .traversed_event_ids
                .contains(&self.restore_event_id),
            "{}: traversed {:?} missing restore event {}",
            self.label,
            resolution.traversed_event_ids,
            self.restore_event_id
        );
        let live_frontier: Vec<_> = resolution
            .resolved
            .iter()
            .copied()
            .filter(|lin| !self.retired_lineages.contains(lin))
            .collect();
        let index = rebuild(&self.log);
        let head = self.log.last().unwrap().commit();
        let index_lin = match index.resolve(self.name_key, head, &exact()).result {
            Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity {
                lineage, ..
            })) => lineage,
            other => panic!("{}: expected exact index target, got {other:?}", self.label),
        };
        assert_eq!(
            live_frontier,
            vec![index_lin],
            "{}: live lineage frontier must agree with PersistentNameBranchIndex head",
            self.label
        );
    }
}

fn summarize_restore_case(
    label: &'static str,
    world: &LineageWorld,
    name: Name,
) -> RestoreTraversalCase {
    let start_lineage = world.log()[1]
        .lineage_events
        .iter()
        .find(|e| e.name_birth().is_some_and(|b| b.identity() == name))
        .expect("initial birth event")
        .targets()[0];
    let restore_event = world
        .log()
        .last()
        .expect("restore commit")
        .lineage_events
        .iter()
        .find(|e| e.kind() == LineageEventKind::Restore && e.kept_name() == Some(name))
        .expect("restore event emitted");
    let mut retired_lineages = BTreeSet::new();
    for batch in world.log() {
        for ev in &batch.lineage_events {
            if ev.kind() == LineageEventKind::Retire {
                retired_lineages.extend(ev.sources().iter().copied());
            }
        }
    }
    RestoreTraversalCase {
        label,
        log: world.log().to_vec(),
        name_key: PersistentNameKey::Entity(name),
        start_lineage,
        restored_lineage: restore_event.targets()[0],
        restore_event_id: restore_event.event_id(),
        retired_lineages,
    }
}

pub(super) fn restore_traversal_cases() -> Vec<RestoreTraversalCase> {
    // 1. Direct Retire -> Restore
    let mut direct = LineageWorld::new();
    let d1 = direct.create(Subject::Entity, "traced");
    direct.commit();
    let d_name = direct.name_of(&d1);
    direct.retire(d_name);
    direct.commit();
    direct.restore(d_name, false);
    direct.commit();

    // 2. Split -> Retire children -> Restore parent
    let mut split_case = LineageWorld::new();
    let p1 = split_case.create(Subject::Entity, "parent");
    split_case.commit();
    let p_name = split_case.name_of(&p1);
    let kids = split_case.split(p_name, &["left", "right"]);
    split_case.commit();
    let (l_name, r_name) = (split_case.name_of(&kids[0]), split_case.name_of(&kids[1]));
    split_case.retire(l_name);
    split_case.retire(r_name);
    split_case.commit();
    split_case.restore(p_name, false);
    split_case.commit();

    // 3. Merge(MintNew) -> Retire merged -> Restore parent
    let mut merge_case = LineageWorld::new();
    let a1 = merge_case.create(Subject::Entity, "a");
    let b1 = merge_case.create(Subject::Entity, "b");
    merge_case.commit();
    let (a_name, b_name) = (merge_case.name_of(&a1), merge_case.name_of(&b1));
    let m1 = merge_case.merge(&[a_name, b_name], Keep::MintNew);
    merge_case.commit();
    let m_name = merge_case.name_of(&m1);
    merge_case.retire(m_name);
    merge_case.commit();
    merge_case.restore(a_name, false);
    merge_case.commit();

    vec![
        summarize_restore_case("retire-then-restore", &direct, d_name),
        summarize_restore_case("split-retire-then-restore-parent", &split_case, p_name),
        summarize_restore_case(
            "merge-mintnew-retire-then-restore-parent",
            &merge_case,
            a_name,
        ),
    ]
}
