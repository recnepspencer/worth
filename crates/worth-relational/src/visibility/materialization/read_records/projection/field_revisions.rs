use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};

use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::{
    RecordLifecycleState, RelationalFieldPresence, RelationalFieldRevision,
};
use crate::storage::overlay::PartitionAccess;

use super::VisibilityProjectionView;

impl VisibilityProjectionView<'_> {
    /// Native field status and revision at an exact immutable root. `None`
    /// means unavailable, including a restored root whose checkpoint grammar
    /// did not retain field revisions; it never means an absent field.
    pub fn entity_field_revision(
        &self,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> Option<RelationalFieldRevision> {
        if !self.is_exact_basis() || locator.aspect().authority() != LocatorAuthority::Authoritative
        {
            return None;
        }
        let [field] = locator.field_path().fields() else {
            return None;
        };
        let root = self.basis.root()?;
        let partition = root.get_partition(entity.partition_id)?;
        let slot = partition.entity_arena.get(&entity)?;
        if slot.lifecycle() != RecordLifecycleState::Live {
            return None;
        }
        let kind = slot.kind_id()?;
        let declared = self
            .entity_aspect_plan(kind)?
            .executable_bindings
            .iter()
            .any(|binding| {
                binding.aspect_key() == locator.aspect().aspect_key()
                    && (binding.targets_entity_scalar_field(field)
                        || binding.targets_entity_struct_field(field))
            });
        if !declared {
            return None;
        }
        let revisions = partition
            .entity_arena
            .field_revisions_at(entity.slot_index())?;
        let (aspect_symbol, field_symbol) = self.runtime.services.symbols.with_read(|symbols| {
            (
                symbols.symbol(locator.aspect().aspect_key().as_str()),
                symbols.symbol(field.as_str()),
            )
        });
        let explicit = aspect_symbol
            .zip(field_symbol)
            .and_then(|key| revisions.get(&key).copied());
        Some(
            explicit.unwrap_or(RelationalFieldRevision::new(
                partition
                    .entity_arena
                    .created_at_for_slot(entity.slot_index())?,
                RelationalFieldPresence::Absent,
            )),
        )
    }

    pub fn relation_field_revision(
        &self,
        relation: RelationId,
        locator: &AspectFieldLocator,
    ) -> Option<RelationalFieldRevision> {
        if !self.is_exact_basis() || locator.aspect().authority() != LocatorAuthority::Authoritative
        {
            return None;
        }
        let [field] = locator.field_path().fields() else {
            return None;
        };
        let root = self.basis.root()?;
        let partition = root.get_partition(relation.partition_id)?;
        let slot = partition.relation_arena.get(&relation)?;
        if slot.lifecycle() != RecordLifecycleState::Live {
            return None;
        }
        let kind = slot.kind_id()?;
        let declared = self
            .relation_aspect_plan(kind)?
            .executable_bindings
            .iter()
            .any(|binding| {
                binding.aspect_key() == locator.aspect().aspect_key()
                    && (binding.targets_relation_scalar_field(field)
                        || binding.targets_relation_struct_field(field))
            });
        if !declared {
            return None;
        }
        let revisions = partition
            .relation_arena
            .field_revisions_at(relation.slot_index())?;
        let (aspect_symbol, field_symbol) = self.runtime.services.symbols.with_read(|symbols| {
            (
                symbols.symbol(locator.aspect().aspect_key().as_str()),
                symbols.symbol(field.as_str()),
            )
        });
        let explicit = aspect_symbol
            .zip(field_symbol)
            .and_then(|key| revisions.get(&key).copied());
        Some(
            explicit.unwrap_or(RelationalFieldRevision::new(
                partition
                    .relation_arena
                    .created_at_for_slot(relation.slot_index())?,
                RelationalFieldPresence::Absent,
            )),
        )
    }
}

#[cfg(test)]
#[path = "field_revisions/restore_tests.rs"]
mod restore_tests;

#[cfg(test)]
mod tests {
    use worth_foundational::facade::{AspectFieldLocator, CanonicalFieldPath, LocatorAuthority};

    use super::*;
    use crate::facade::identity::{KindId, PartitionId};
    use crate::facade::mvcc::WorkerIntentBatch;
    use crate::facade::transactions::{
        ApplyEntityAspectPatchIntent, CreateIntent, EntityMutationIntent, MutationIntent,
        UpdateEntityFieldsIntent,
    };
    use crate::tests::support::*;
    use worth_foundational::facade::{
        PortableAspectContractBasis, PortableAspectPatchOperation, PortableRecordAspectPatch,
    };

    fn locator(aspect: &str, field: &str) -> AspectFieldLocator {
        AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            aspect_key(aspect),
            CanonicalFieldPath::single(field_key(field)),
        )
    }

    #[test]
    fn exact_field_revisions_keep_unrelated_sibling_and_absence_stable() {
        let runtime = AspectSchemaFixture {
            entity_aspects: vec![entity_summary_struct_aspect(
                aspect_key("summary"),
                field_key("summary"),
            )],
            ..AspectSchemaFixture::default()
        }
        .build_runtime();
        let mut create = test_owner_begin_transaction_for_main(&runtime);
        create
            .push_batch(WorkerIntentBatch::new("field-revision-create").push(
                MutationIntent::Create(CreateIntent::Entity(
                    crate::transactions::data::EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("field-revision"),
                        fields: string_aspect_field_patch([(
                            aspect_key("summary"),
                            field_key("title"),
                            "same",
                        )]),
                    },
                )),
            ))
            .unwrap();
        let created = create.commit(&runtime).unwrap();
        let entity = changed_entities(&created)[0];
        let title = locator("summary", "title");
        let status = locator("summary", "status");
        let read = |snapshot| {
            let view = runtime.read_truth().project_snapshot(snapshot).unwrap();
            (
                view.entity_field_revision(entity, &title),
                view.entity_field_revision(entity, &status),
            )
        };
        let before = read(&created.snapshot);
        assert_eq!(
            before.0.unwrap().presence(),
            RelationalFieldPresence::Present
        );
        assert_eq!(
            before.1.unwrap().presence(),
            RelationalFieldPresence::Absent
        );

        let mut update = test_owner_begin_transaction_for_main(&runtime);
        update
            .push_batch(WorkerIntentBatch::new("field-revision-update").push(
                MutationIntent::Entity(EntityMutationIntent::UpdateFields(
                    UpdateEntityFieldsIntent {
                        entity_id: entity,
                        fields: string_aspect_field_patch([(
                            aspect_key("summary"),
                            field_key("status"),
                            "ready",
                        )]),
                    },
                )),
            ))
            .unwrap();
        let updated = update.commit(&runtime).unwrap();
        let after = read(&updated.snapshot);
        assert_eq!(before.0, after.0);
        assert_ne!(before.1, after.1);
        assert_eq!(
            after.1.unwrap().presence(),
            RelationalFieldPresence::Present
        );
        assert_eq!(read(&created.snapshot), before);
        release_test_commit_snapshot(&runtime, &created);
        release_test_commit_snapshot(&runtime, &updated);
    }

    #[test]
    fn absent_present_absent_aba_has_distinct_native_revision() {
        let binding = entity_summary_struct_aspect(aspect_key("summary"), field_key("summary"));
        let contract = binding.contract.clone();
        let runtime = AspectSchemaFixture {
            entity_aspects: vec![binding],
            ..AspectSchemaFixture::default()
        }
        .build_runtime();
        let mut create = test_owner_begin_transaction_for_main(&runtime);
        create
            .push_batch(
                WorkerIntentBatch::new("aba-create").push(MutationIntent::Create(
                    CreateIntent::Entity(crate::transactions::data::EntitySpec {
                        partition_id: PartitionId::main(),
                        kind_id: KindId(1),
                        client_key: crate::symbols::data::ClientKey::raw("aba"),
                        fields: string_aspect_field_patch([(
                            aspect_key("summary"),
                            field_key("title"),
                            "fixed",
                        )]),
                    }),
                )),
            )
            .unwrap();
        let created = create.commit(&runtime).unwrap();
        let entity = changed_entities(&created)[0];
        let status = locator("summary", "status");
        let title = locator("summary", "title");
        let revision = |snapshot| {
            runtime
                .read_truth()
                .project_snapshot(snapshot)
                .unwrap()
                .entity_field_revision(entity, &status)
                .unwrap()
        };
        let initial = revision(&created.snapshot);
        assert_eq!(initial.presence(), RelationalFieldPresence::Absent);
        let title_initial = runtime
            .read_truth()
            .project_snapshot(&created.snapshot)
            .unwrap()
            .entity_field_revision(entity, &title);

        let mut set = test_owner_begin_transaction_for_main(&runtime);
        set.push_batch(
            WorkerIntentBatch::new("aba-set").push(MutationIntent::Entity(
                EntityMutationIntent::UpdateFields(UpdateEntityFieldsIntent {
                    entity_id: entity,
                    fields: string_aspect_field_patch([(
                        aspect_key("summary"),
                        field_key("status"),
                        "intermediate",
                    )]),
                }),
            )),
        )
        .unwrap();
        let present = set.commit(&runtime).unwrap();
        assert_eq!(
            revision(&present.snapshot).presence(),
            RelationalFieldPresence::Present
        );

        let mut clear = test_owner_begin_transaction_for_main(&runtime);
        clear
            .push_batch(
                WorkerIntentBatch::new("aba-clear").push(MutationIntent::Entity(
                    EntityMutationIntent::ApplyAspectPatch(ApplyEntityAspectPatchIntent {
                        entity_id: entity,
                        aspect_patch: PortableRecordAspectPatch::new([
                            PortableAspectPatchOperation::PatchFields {
                                basis: PortableAspectContractBasis::from_contract(&contract),
                                selected_fields: vec![field_key("status")],
                                field_sets: Vec::new(),
                                field_clears: vec![field_key("status")],
                            },
                        ]),
                    }),
                )),
            )
            .unwrap();
        let absent_again = clear.commit(&runtime).unwrap();
        let final_revision = revision(&absent_again.snapshot);
        assert_eq!(final_revision.presence(), RelationalFieldPresence::Absent);
        assert_ne!(initial, final_revision);
        assert_eq!(
            title_initial,
            runtime
                .read_truth()
                .project_snapshot(&absent_again.snapshot)
                .unwrap()
                .entity_field_revision(entity, &title)
        );
        for outcome in [&created, &present, &absent_again] {
            release_test_commit_snapshot(&runtime, outcome);
        }
    }

    #[test]
    fn forked_updates_preserve_the_original_field_revision_and_recreation_changes_identity() {
        let runtime = runtime_with_test_schema();
        let created = create_entity_outcome(&runtime, "first");
        let entity = changed_entities(&created)[0];
        let name = locator("name", "name");
        let original = runtime
            .read_truth()
            .project_snapshot(&created.snapshot)
            .unwrap()
            .entity_field_revision(entity, &name)
            .unwrap();
        let fork = create_branch_from_main(&runtime, "field-revision-fork");
        let changed = update_entity_on_branch(&runtime, entity, "second", fork.clone());
        let on_fork = runtime
            .read_truth()
            .project_snapshot(&changed.snapshot)
            .unwrap()
            .entity_field_revision(entity, &name)
            .unwrap();
        assert_ne!(on_fork, original);
        assert_eq!(
            runtime
                .read_truth()
                .project_snapshot(&created.snapshot)
                .unwrap()
                .entity_field_revision(entity, &name),
            Some(original)
        );
        let main = snapshot_for_owner_branch(
            &runtime,
            &crate::facade::history::BranchId("main".to_owned()),
        );
        assert_eq!(
            runtime
                .read_truth()
                .project_snapshot(&main)
                .unwrap()
                .entity_field_revision(entity, &name),
            Some(original)
        );
        runtime.snapshots().release_snapshot(&main).unwrap();

        let deleted = delete_entity(&runtime, entity);
        assert_eq!(
            runtime
                .read_truth()
                .project_snapshot(&deleted.snapshot)
                .unwrap()
                .entity_field_revision(entity, &name),
            None
        );
        let recreated = create_entity_outcome(&runtime, "first");
        let new_entity = changed_entities(&recreated)[0];
        assert_ne!(new_entity, entity);
        assert_ne!(
            runtime
                .read_truth()
                .project_snapshot(&recreated.snapshot)
                .unwrap()
                .entity_field_revision(new_entity, &name),
            Some(original)
        );
        for outcome in [&created, &changed, &deleted, &recreated] {
            release_test_commit_snapshot(&runtime, outcome);
        }
    }
}
