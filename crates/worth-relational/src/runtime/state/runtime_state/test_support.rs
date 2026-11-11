use super::RelationalRuntime;

impl RelationalRuntime {
    /// Edit the configuration in force, the way an owner would.
    ///
    /// Tests that need a non-default policy go through the same single
    /// installation route as production reconfiguration, so nothing in the test
    /// tree can leave this handle reading a configuration it has replaced.
    pub(crate) fn configure_for_test(
        &mut self,
        edit: impl FnOnce(&mut crate::runtime::RelationalRuntimeConfig),
    ) {
        self.reconfigure(|configuration| configuration.update(edit));
    }

    pub(crate) fn configure_diagnostics_for_test(
        &mut self,
        configure: impl FnOnce(&mut crate::diagnostics::data::RelationalDiagnosticsProfile),
    ) {
        self.reconfigure(|configuration| {
            configuration.update(|config| configure(&mut config.diagnostics.profile));
        });
    }

    pub(crate) fn set_schema_registry_for_test(
        &mut self,
        registry: crate::schema::data::RelationalSchemaRegistry,
    ) {
        self.reconfigure(|configuration| {
            configuration.update(|config| config.schema.registry = registry);
        });
    }

    pub(crate) fn set_entity_structural_identity_for_test(
        &mut self,
        entity_id: crate::identity::data::EntityId,
        structural_fingerprint: Option<crate::identity::data::StructuralFingerprint>,
        lineage_id: Option<crate::identity::data::LineageId>,
    ) -> bool {
        let mut writer = self.edit_partitions();
        let Some(partition) = writer.partition_mut(entity_id.partition_id) else {
            return false;
        };
        if partition.entity_arena.get(&entity_id).is_none() {
            return false;
        }
        partition.entity_arena.extra[entity_id.local_slot.0 as usize] =
            crate::storage::substrate::EntityExtra::from_unbudgeted_parts(
                structural_fingerprint,
                lineage_id,
                None,
            );
        true
    }

    pub(crate) fn simulate_entity_slot_reuse_for_test(
        &mut self,
        entity_id: crate::identity::data::EntityId,
        structural_fingerprint: Option<crate::identity::data::StructuralFingerprint>,
        lineage_id: Option<crate::identity::data::LineageId>,
    ) -> Option<crate::identity::data::EntityId> {
        let replacement_version = crate::identity::data::VersionId(self.current_version_id().0 + 1);
        let mut writer = self.edit_partitions();
        let partition = writer.partition_mut(entity_id.partition_id)?;
        let arena = &mut partition.entity_arena;
        arena.get(&entity_id)?;
        let slot = entity_id.local_slot.0 as usize;
        let next_generation = arena.generations[slot] + 1;

        arena.generations[slot] = next_generation;
        arena.lifecycle[slot] = crate::storage::data::RecordLifecycleState::Live;
        arena.apply_extra_update(
            slot,
            crate::storage::substrate::EntityExtra::from_unbudgeted_parts(
                structural_fingerprint,
                lineage_id,
                None,
            ),
            replacement_version,
        )?;
        arena.created_at[slot] = replacement_version;
        arena.retired_at[slot] = None;
        arena.live_bitset.set(slot, true);
        arena.reclaimable_bitset.set(slot, false);
        self.history
            .with_ledger_mut(|ledger| ledger.next_version_id = replacement_version.0 + 1);

        Some(crate::identity::data::EntityId::new(
            entity_id.partition_id,
            entity_id.local_slot.0,
            next_generation,
        ))
    }

    pub(crate) fn entity_history_len_for_test(
        &self,
        entity_id: crate::identity::data::EntityId,
    ) -> usize {
        self.partitions
            .partition(entity_id.partition_id)
            .and_then(|partition| {
                partition
                    .entity_arena
                    .metadata_history_at(entity_id.local_slot.0 as usize)
                    .map(|history| history.len())
            })
            .unwrap_or(0)
    }

    pub(crate) fn relation_history_len_for_test(
        &self,
        relation_id: crate::identity::data::RelationId,
    ) -> usize {
        self.partitions
            .partition(relation_id.partition_id)
            .and_then(|partition| {
                partition
                    .relation_arena
                    .metadata_history_at(relation_id.local_slot.0 as usize)
                    .map(|history| history.len())
            })
            .unwrap_or(0)
    }
}
