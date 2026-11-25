//! A real Relational world of named faces and seams committed through
//! `ExecutionNamingCommitAuthority`, for the tracked resolution courts.

use worth_execution::ExecutionRequest;
use worth_query_declaration::facade::application_naming::WorthQueryName;
use worth_query_execution_naming::facade::{
    AdmittedCommitNames, ExecutionNameBirthPlanBuilder, ExecutionNameReader,
    ExecutionNamedMergeKeeper, ExecutionNamingCommitAuthority, ExecutionPendingEntityBirth,
    ExecutionPendingRelationBirth, ExecutionSplitChildSpec, LoweredExecutionNameSchemeCatalog,
};
use worth_relational::facade::identity::{EntityId, RelationId};
use worth_relational::facade::lineage::{
    LineageRecordTarget, PendingBirthTarget, PendingMergeTarget, PersistentNameIdentity,
};
use worth_relational::facade::mvcc::RelationalBranchObservation;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::{
    CommitResult, DeleteEntityIntent, EntityMutationIntent, EntityReference, MutationIntent,
    RelationMutationIntent,
};

use crate::naming_execution_fixture::{
    begin_staged_tx, create_face_intent, create_seam_intent, face_ref, fresh_cad_runtime,
    lowered_cad_catalog, release_commit_snapshot, seam_ref, CadNamingSchema, Face, FaceSplitDisc,
    Seam, SeamSplitDisc, FACE_SCHEME_ID, SEAM_SCHEME_ID,
};

/// One named face: its live record, its admitted name and its identity.
#[derive(Clone, Copy, Debug)]
pub struct NamedFace {
    pub id: EntityId,
    pub name: WorthQueryName<Face>,
    pub identity: PersistentNameIdentity,
}

/// How [`TrackedWorld::rebirth`] re-mints a name.
#[derive(Clone, Copy, Debug)]
pub enum Rebirth {
    Regenerate,
    Restore,
}

/// One named seam relation.
#[derive(Clone, Copy, Debug)]
pub struct NamedSeam {
    pub id: RelationId,
    pub name: WorthQueryName<Seam>,
    pub identity: PersistentNameIdentity,
}

pub struct TrackedWorld {
    pub runtime: std::sync::Arc<RelationalRuntime>,
    catalog: LoweredExecutionNameSchemeCatalog,
}

impl TrackedWorld {
    pub fn new() -> Self {
        Self::with_runtime(fresh_cad_runtime())
    }

    /// A world over `runtime`, which must install the CAD name schemes.
    pub fn with_runtime(runtime: RelationalRuntime) -> Self {
        Self {
            runtime: std::sync::Arc::new(runtime),
            catalog: lowered_cad_catalog(),
        }
    }

    /// The main branch head as an admitted observation.
    pub fn head(&self) -> RelationalBranchObservation {
        self.runtime
            .admit_branch_basis(&self.runtime.main_branch_identity())
            .expect("head basis")
            .observation()
    }

    pub fn reader<'read>(
        &'read self,
        execution: &'read ExecutionRequest<'read, 'read>,
        observation: &'read RelationalBranchObservation,
    ) -> ExecutionNameReader<'read> {
        ExecutionNameReader::at_observation(&self.runtime, execution, observation)
    }

    /// Seeds one named face per key, all in one commit; each face's authored
    /// occurrence is `{occurrence}.{key}`, so its name follows from both.
    pub fn seed(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        occurrence: &str,
        keys: &[&str],
    ) -> (Vec<NamedFace>, CommitResult) {
        let intents = keys.iter().map(|key| create_face_intent(key)).collect();
        let tx = begin_staged_tx(&self.runtime, occurrence, intents);
        let mut plan = ExecutionNameBirthPlanBuilder::new(&self.catalog, std::iter::empty());
        for key in keys {
            let occurrence_key = format!("{occurrence}.{key}");
            let birth = ExecutionPendingEntityBirth::new(
                face_ref(key),
                FACE_SCHEME_ID,
                occurrence_key,
                "authored",
            )
            .with_discriminator(&FaceSplitDisc::Side { side: 0 });
            plan.add_entity_birth(birth).expect("seed birth");
        }
        let (committed, names) = self.commit(execution, tx, plan);
        let faces = keys
            .iter()
            .map(|key| face(&names, &committed, key))
            .collect();
        (faces, committed)
    }

    /// Splits `parent` into one face per `(key, side)`.
    pub fn split(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        parent: NamedFace,
        occurrence: &str,
        children: &[(&str, u8)],
    ) -> (Vec<NamedFace>, CommitResult) {
        let mut intents = vec![delete_face(parent.id)];
        intents.extend(children.iter().map(|(key, _)| create_face_intent(key)));
        let tx = begin_staged_tx(&self.runtime, occurrence, intents);
        let mut plan = ExecutionNameBirthPlanBuilder::new(&self.catalog, [parent.identity]);
        let specs = children
            .iter()
            .map(|(key, side)| {
                ExecutionSplitChildSpec::entity(
                    face_ref(key),
                    FACE_SCHEME_ID,
                    occurrence,
                    "cut_piece",
                )
                .with_discriminator(&FaceSplitDisc::Side { side: *side })
            })
            .collect();
        plan.add_split_posture(
            LineageRecordTarget::Entity(parent.id),
            parent.identity,
            specs,
        )
        .expect("split posture");
        let (committed, names) = self.commit(execution, tx, plan);
        let faces = children
            .iter()
            .map(|(key, _)| face(&names, &committed, key))
            .collect();
        (faces, committed)
    }

    /// Merges `absorbed` into `keeper`, which keeps its record and name.
    pub fn merge_keeping(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        keeper: NamedFace,
        absorbed: NamedFace,
    ) -> CommitResult {
        let tx = begin_staged_tx(&self.runtime, "merge", vec![delete_face(absorbed.id)]);
        let mut plan =
            ExecutionNameBirthPlanBuilder::new(&self.catalog, [keeper.identity, absorbed.identity]);
        plan.add_merge_posture(
            vec![
                (LineageRecordTarget::Entity(keeper.id), keeper.identity),
                (LineageRecordTarget::Entity(absorbed.id), absorbed.identity),
            ],
            PendingMergeTarget::Existing(LineageRecordTarget::Entity(keeper.id)),
            ExecutionNamedMergeKeeper::KeepParent(keeper.identity),
        )
        .expect("merge posture");
        self.commit(execution, tx, plan).0
    }

    /// Merges `a` and `b` into a new face `key` that mints a new name.
    pub fn merge_minting(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        a: NamedFace,
        b: NamedFace,
        key: &str,
    ) -> (NamedFace, CommitResult) {
        let intents = vec![
            delete_face(a.id),
            delete_face(b.id),
            create_face_intent(key),
        ];
        let tx = begin_staged_tx(&self.runtime, "merge-mint", intents);
        let mut plan = ExecutionNameBirthPlanBuilder::new(&self.catalog, [a.identity, b.identity]);
        plan.add_merge_posture(
            vec![
                (LineageRecordTarget::Entity(a.id), a.identity),
                (LineageRecordTarget::Entity(b.id), b.identity),
            ],
            PendingMergeTarget::Created(PendingBirthTarget::CreatedEntity(face_ref(key))),
            ExecutionNamedMergeKeeper::mint_new(
                FACE_SCHEME_ID,
                "occ.merge.mint",
                "cut_piece",
                &FaceSplitDisc::Side { side: 9 },
            ),
        )
        .expect("mint-new merge posture");
        let (committed, names) = self.commit(execution, tx, plan);
        (face(&names, &committed, key), committed)
    }

    /// Re-mints the name `original`, seeded as `key` under `occurrence`, was
    /// born with onto a new record `new_key`: `Rebirth::Regenerate` retires
    /// `original` in the same commit; `Rebirth::Restore` revives a name that
    /// retired earlier. The new record carries the kept name, not a birth, so
    /// it is read back through the tracked reader.
    pub fn rebirth(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        original: NamedFace,
        (occurrence, key, new_key): (&str, &str, &str),
        mode: Rebirth,
    ) -> (NamedFace, CommitResult) {
        let regenerating = matches!(mode, Rebirth::Regenerate);
        let mut intents = Vec::new();
        if regenerating {
            intents.push(delete_face(original.id));
        }
        intents.push(create_face_intent(new_key));
        let tx = begin_staged_tx(&self.runtime, new_key, intents);
        let basis = regenerating.then_some(original.identity);
        let mut plan = ExecutionNameBirthPlanBuilder::new(&self.catalog, basis);
        if regenerating {
            plan.add_retire_entity(original.id, original.identity);
        }
        let occurrence_key = format!("{occurrence}.{key}");
        let birth = ExecutionPendingEntityBirth::new(
            face_ref(new_key),
            FACE_SCHEME_ID,
            occurrence_key,
            "authored",
        )
        .with_discriminator(&FaceSplitDisc::Side { side: 0 });
        plan.add_entity_birth(birth).expect("re-minted birth");
        let (committed, _) = self.commit(execution, tx, plan);
        let id = committed
            .created_entity(&face_ref(new_key))
            .expect("re-minted record");
        let head = self.head();
        let name = self
            .reader(execution, &head)
            .entity_name_of::<CadNamingSchema, Face>(id, &mut Default::default())
            .expect("the new record carries the re-minted name");
        let identity = original.identity;
        (NamedFace { id, name, identity }, committed)
    }

    pub fn retire(&self, execution: &ExecutionRequest<'_, '_>, face: NamedFace) -> CommitResult {
        let tx = begin_staged_tx(&self.runtime, "retire", vec![delete_face(face.id)]);
        let mut plan = ExecutionNameBirthPlanBuilder::new(&self.catalog, [face.identity]);
        plan.add_retire_entity(face.id, face.identity);
        self.commit(execution, tx, plan).0
    }

    /// Seeds a named seam from `from` to `to`.
    pub fn seam(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        key: &str,
        from: NamedFace,
        to: NamedFace,
    ) -> (NamedSeam, CommitResult) {
        let (created, committed, names) = self.commit_seam(execution, key, from, to);
        let (id, name) = names
            .created_relation_name::<Seam>(&committed, &created)
            .expect("seam name admitted");
        let identity = names
            .relation_relational_identity(id)
            .expect("seam identity");
        (NamedSeam { id, name, identity }, committed)
    }

    /// Mints `key`'s seam again once its name retired. The new relation
    /// carries the restored name, not a birth, so only its id is returned.
    pub fn remint_seam(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        key: &str,
        from: NamedFace,
        to: NamedFace,
    ) -> RelationId {
        let (created, committed, _) = self.commit_seam(execution, key, from, to);
        committed
            .created_relation(&created)
            .expect("re-minted seam record")
    }

    fn commit_seam(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        key: &str,
        from: NamedFace,
        to: NamedFace,
    ) -> (
        worth_relational::facade::transactions::CreatedRelationRef,
        CommitResult,
        AdmittedCommitNames,
    ) {
        let (source, target) = (
            EntityReference::Existing(from.id),
            EntityReference::Existing(to.id),
        );
        let created = seam_ref(key, source.clone(), target.clone());
        let tx = begin_staged_tx(
            &self.runtime,
            key,
            vec![create_seam_intent(key, source, target)],
        );
        let mut plan = ExecutionNameBirthPlanBuilder::new(&self.catalog, [from.identity]);
        plan.add_relation_birth(
            ExecutionPendingRelationBirth::new(created.clone(), SEAM_SCHEME_ID, key, "seam")
                .with_discriminator(&SeamSplitDisc::ByFace {
                    face: from.name,
                    index: 7,
                }),
        )
        .expect("seam birth");
        let (committed, names) = self.commit(execution, tx, plan);
        (created, committed, names)
    }

    pub fn retire_seam(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        seam: NamedSeam,
    ) -> CommitResult {
        let delete = MutationIntent::Relation(RelationMutationIntent::Delete(
            worth_relational::facade::transactions::DeleteRelationIntent {
                relation_id: seam.id,
            },
        ));
        let tx = begin_staged_tx(&self.runtime, "retire-seam", vec![delete]);
        let mut plan = ExecutionNameBirthPlanBuilder::new(&self.catalog, [seam.identity]);
        plan.add_retire_relation(seam.id, seam.identity);
        self.commit(execution, tx, plan).0
    }

    fn commit(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        tx: worth_relational::facade::mvcc::BranchBoundRelationalTransaction,
        plan: ExecutionNameBirthPlanBuilder<'_>,
    ) -> (CommitResult, AdmittedCommitNames) {
        let (result, names) = ExecutionNamingCommitAuthority::commit_with_naming_plan(
            &self.runtime,
            execution,
            tx,
            plan.build(),
        )
        .expect("named commit publishes")
        .into_parts();
        release_commit_snapshot(&self.runtime, &result);
        (result, names)
    }
}

fn face(names: &AdmittedCommitNames, committed: &CommitResult, key: &str) -> NamedFace {
    let (id, name) = names
        .created_entity_name::<CadNamingSchema, Face>(committed, &face_ref(key))
        .expect("face name admitted");
    let identity = names.entity_relational_identity(id).expect("face identity");
    NamedFace { id, name, identity }
}

fn delete_face(entity_id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id,
    }))
}
