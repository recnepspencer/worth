//! Real Phase 2 naming world: every commit runs Relational posture
//! finalization over a staged working state and hands the resulting canonical
//! batch to the derived name index courts.

use std::collections::{BTreeMap, BTreeSet};

use crate::facade::history::CommitId;
use crate::facade::identity::{EntityId, KindId, LineageId, PartitionId, VersionId};
use crate::facade::lineage::{
    InstalledPersistentNameSchemeRegistry, LineageCommitPostureInput, LineageEventKind,
    LineageMergeKeeper, LineagePostureClaim, LineageRecordTarget, LineageSplitChild,
    PersistentNameBirthRequest, PersistentNameCommittedBatch, PersistentNameHolder,
    PersistentNameIdentity, PersistentNameKey,
};
use crate::facade::transactions::{
    AspectFieldPatch, EntityMutationIntent, EntitySpec, MutationIntent, RecordRef,
    ReplaceEntityIntent,
};
use crate::lineage::authority::finalize_postures_in_working_state;
use crate::runtime::LineageIdentityAllocator;
use crate::storage::overlay::WorkingState;
use crate::symbols::data::ClientKey;

use super::posture_fixtures::{
    cad_scheme_registry, empty_working_state, face_birth, push_entity, push_relation, receipt_at,
    retire_entity, retire_relation, seam_birth,
};

pub(super) type Name = PersistentNameIdentity;
pub(super) use super::posture_fixtures::canonical_label_discriminator as side_of;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Subject {
    Entity,
    Relation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Keep {
    InPlace,
    Recreated,
    MintNew,
}

/// One branch of real entity and relation naming history.
pub(super) struct LineageWorld {
    allocator: LineageIdentityAllocator,
    registry: InstalledPersistentNameSchemeRegistry,
    staged: WorkingState,
    commit: u64,
    anchors: (EntityId, EntityId),
    holders: BTreeMap<LineageId, PersistentNameHolder>,
    births: BTreeMap<PersistentNameHolder, Name>,
    live_holders: BTreeMap<Name, PersistentNameHolder>,
    live_lineages: BTreeMap<PersistentNameHolder, LineageId>,
    postures: Vec<LineagePostureClaim>,
    changed: Vec<RecordRef>,
    plan: Vec<MutationIntent>,
    carriers: Vec<(PersistentNameHolder, BTreeSet<Name>)>,
    ordinal: usize,
    log: Vec<PersistentNameCommittedBatch>,
}

impl LineageWorld {
    /// Commit 1 creates two unnamed anchor faces that every seam spans.
    pub(super) fn new() -> Self {
        let mut staged = empty_working_state();
        let anchors = (
            push_entity(&mut staged, VersionId(1)),
            push_entity(&mut staged, VersionId(1)),
        );
        let mut world = Self {
            allocator: LineageIdentityAllocator::new(),
            registry: cad_scheme_registry(),
            staged,
            commit: 0,
            anchors,
            holders: BTreeMap::new(),
            births: BTreeMap::new(),
            live_holders: BTreeMap::new(),
            live_lineages: BTreeMap::new(),
            postures: Vec::new(),
            changed: vec![RecordRef::Entity(anchors.0), RecordRef::Entity(anchors.1)],
            plan: Vec::new(),
            carriers: Vec::new(),
            ordinal: 0,
            log: Vec::new(),
        };
        world.commit();
        world
    }

    pub(super) fn log(&self) -> &[PersistentNameCommittedBatch] {
        &self.log
    }

    pub(super) fn head(&self) -> CommitId {
        CommitId(self.commit)
    }

    pub(super) fn name_of(&self, holder: &PersistentNameHolder) -> Name {
        self.births[holder]
    }

    pub(super) fn key_of(&self, holder: &PersistentNameHolder) -> PersistentNameKey {
        key(holder, self.name_of(holder))
    }

    pub(super) fn live_names(&self) -> Vec<(Name, PersistentNameHolder)> {
        self.live_holders
            .iter()
            .map(|(n, h)| (*n, h.clone()))
            .collect()
    }

    fn version(&self) -> VersionId {
        VersionId(self.commit + 1)
    }

    fn occurrence(&mut self) -> String {
        self.ordinal += 1;
        format!("occ.{}.{}", self.commit + 1, self.ordinal)
    }

    fn new_record(&mut self, subject: Subject) -> PersistentNameHolder {
        let version = self.version();
        let holder = match subject {
            Subject::Entity => PersistentNameHolder::Entity(push_entity(&mut self.staged, version)),
            Subject::Relation => {
                let (a, b) = self.anchors;
                PersistentNameHolder::Relation(push_relation(&mut self.staged, version, a, b))
            }
        };
        self.changed.push(record_ref(&holder));
        holder
    }

    fn retire_record(&mut self, holder: &PersistentNameHolder) {
        let version = self.version();
        match holder {
            PersistentNameHolder::Entity(e) => retire_entity(&mut self.staged, *e, version),
            PersistentNameHolder::Relation(r) => retire_relation(&mut self.staged, *r, version),
            _ => unreachable!("worlds hold only entity and relation records"),
        }
        self.changed.push(record_ref(holder));
    }

    fn birth(&mut self, holder: &PersistentNameHolder, role: &str, label: &str) -> BirthRequest {
        let occurrence = self.occurrence();
        match holder {
            PersistentNameHolder::Entity(e) => face_birth(&occurrence, role, vec![], label, *e),
            PersistentNameHolder::Relation(r) => {
                let role = role
                    .replace("cut_piece", "seam_piece")
                    .replace("face", "seam");
                let role = if role == "authored" { "seam" } else { &role };
                seam_birth(&occurrence, role, vec![], label, *r)
            }
            _ => unreachable!("worlds hold only entity and relation records"),
        }
    }

    pub(super) fn create(&mut self, subject: Subject, label: &str) -> PersistentNameHolder {
        let holder = self.new_record(subject);
        let birth = self.birth(&holder, "authored", label);
        let target = record_target(&holder);
        self.postures
            .push(LineagePostureClaim::Create { target, birth });
        holder
    }

    /// Declares `carriers` for the name `holder` carries after this commit.
    pub(super) fn bind_carriers(&mut self, holder: &PersistentNameHolder, carriers: &[Name]) {
        let carriers = carriers.iter().copied().collect();
        self.carriers.push((holder.clone(), carriers));
    }

    pub(super) fn split(&mut self, parent: Name, labels: &[&str]) -> Vec<PersistentNameHolder> {
        let parent_holder = self.live_holders[&parent].clone();
        self.retire_record(&parent_holder);
        let subject = subject_of(&parent_holder);
        let mut children = Vec::new();
        let mut holders = Vec::new();
        for label in labels {
            let holder = self.new_record(subject);
            let mut birth = self.birth(&holder, "cut_piece", label);
            birth.parent_identities = vec![parent];
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

    /// Merges `parents`; the keeper (for `Keep`ing modes) is `parents[0]`.
    pub(super) fn merge(&mut self, parents: &[Name], keep: Keep) -> PersistentNameHolder {
        let listed: Vec<_> = parents
            .iter()
            .map(|name| (self.live_holders[name].clone(), *name))
            .collect();
        for (index, (holder, _)) in listed.iter().enumerate() {
            if index > 0 || keep != Keep::InPlace {
                self.retire_record(holder);
            }
        }
        let subject = subject_of(&listed[0].0);
        let (target, keeper) = match keep {
            Keep::InPlace => (
                listed[0].0.clone(),
                LineageMergeKeeper::KeepParent(parents[0]),
            ),
            Keep::Recreated => (
                self.new_record(subject),
                LineageMergeKeeper::KeepParent(parents[0]),
            ),
            Keep::MintNew => {
                let target = self.new_record(subject);
                let birth = self.birth(&target, "merged_face", "merged");
                (target, LineageMergeKeeper::MintNew { birth })
            }
        };
        self.postures.push(LineagePostureClaim::Merge {
            parents: listed
                .iter()
                .map(|(holder, name)| (record_target(holder), *name))
                .collect(),
            target: record_target(&target),
            keeper,
        });
        target
    }

    pub(super) fn retire(&mut self, name: Name) {
        let holder = self.live_holders[&name].clone();
        self.retire_record(&holder);
        let target = record_target(&holder);
        self.postures
            .push(LineagePostureClaim::Retire { target, name });
    }

    /// Unpostured default `Replace` of a named face by an unnamed successor.
    pub(super) fn replace(&mut self, name: Name) {
        let PersistentNameHolder::Entity(old) = self.live_holders[&name] else {
            panic!("only entities replace");
        };
        self.retire_record(&PersistentNameHolder::Entity(old));
        self.new_record(Subject::Entity);
        let client_key = ClientKey::raw(format!("replace-{}", self.occurrence()));
        self.plan
            .push(MutationIntent::Entity(EntityMutationIntent::Replace(
                ReplaceEntityIntent {
                    entity_id: old,
                    replacement: EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key,
                        fields: AspectFieldPatch::default(),
                    },
                },
            )));
    }

    /// Finalizes every staged posture as one real Relational commit.
    pub(super) fn commit(&mut self) -> PersistentNameCommittedBatch {
        self.commit += 1;
        let input = LineageCommitPostureInput {
            scheme_registry: self.registry.clone(),
            live_holders: self.live_holders.clone(),
            live_lineages: self.live_lineages.clone(),
            postures: std::mem::take(&mut self.postures),
        };
        let prepared = finalize_postures_in_working_state(
            &self.allocator,
            &mut self.staged,
            &receipt_at(self.commit, self.commit),
            &std::mem::take(&mut self.plan),
            &std::mem::take(&mut self.changed),
            &input,
        )
        .expect("staged postures finalize as one canonical commit");
        for node in prepared.new_nodes() {
            let holder = PersistentNameHolder::Entity(node.entity_id());
            self.holders.insert(node.lineage_id(), holder);
        }
        for (relation, lineage) in prepared.relation_lineages() {
            self.holders
                .insert(*lineage, PersistentNameHolder::Relation(*relation));
        }
        let events = prepared.artifact().event_batch().events();
        for event in events
            .iter()
            .filter(|e| e.kind() == LineageEventKind::Create)
        {
            let lineage = event.targets()[0];
            let holder = self.holders[&lineage].clone();
            if let Some(birth) = event.name_birth() {
                self.births.insert(holder.clone(), birth.identity());
                self.live_holders.insert(birth.identity(), holder.clone());
                self.live_lineages.insert(holder.clone(), lineage);
            }
            if let Some(kept) = event.kept_name() {
                self.births.insert(holder.clone(), kept);
                self.live_holders.insert(kept, holder.clone());
                self.live_lineages.insert(holder, lineage);
            }
        }
        for event in events
            .iter()
            .filter(|e| e.kind() != LineageEventKind::Create)
        {
            for source in event.sources() {
                if event.targets().contains(source) {
                    continue;
                }
                let holder = self.holders[source].clone();
                self.live_lineages.remove(&holder);
                self.live_holders.retain(|_, live| *live != holder);
            }
        }
        let mut batch =
            PersistentNameCommittedBatch::new(self.head()).with_prepared_lineage(&prepared);
        for (holder, carriers) in std::mem::take(&mut self.carriers) {
            batch = batch.with_carriers(self.key_of(&holder), carriers);
        }
        self.log.push(batch.clone());
        batch
    }
}

type BirthRequest = PersistentNameBirthRequest;

pub(super) fn key(holder: &PersistentNameHolder, name: Name) -> PersistentNameKey {
    match holder {
        PersistentNameHolder::Relation(_) => PersistentNameKey::Relation(name),
        _ => PersistentNameKey::Entity(name),
    }
}

fn subject_of(holder: &PersistentNameHolder) -> Subject {
    match holder {
        PersistentNameHolder::Relation(_) => Subject::Relation,
        _ => Subject::Entity,
    }
}

fn record_target(holder: &PersistentNameHolder) -> LineageRecordTarget {
    match holder {
        PersistentNameHolder::Entity(e) => LineageRecordTarget::Entity(*e),
        PersistentNameHolder::Relation(r) => LineageRecordTarget::Relation(*r),
        _ => unreachable!("worlds hold only entity and relation records"),
    }
}

fn record_ref(holder: &PersistentNameHolder) -> RecordRef {
    record_target(holder).to_record_ref()
}
