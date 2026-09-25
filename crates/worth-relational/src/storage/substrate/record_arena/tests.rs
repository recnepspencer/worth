#[cfg(test)]
mod tests {
    use worth_foundational::{
        admit_authoritative_record_aspect_state, aspects, validate_aspect_value, AspectContract,
        AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
        AuthoritativeRecordAspectState, ContractValidationInput, FieldKey, InternedString,
        ScalarAspectType, StructAspectValue,
    };
    use worth_proof::TransitionOutcome;

    use crate::identity::data::{EntityId, KindId, PartitionId, RelationId, VersionId};
    use crate::storage::data::RecordLifecycleState;
    use crate::symbols::data::Symbol;

    use super::super::{EntityArena, EntityExtra, RelationArena, RelationEndpoints, RelationExtra};

    #[test]
    fn reusing_entity_slot_clears_entity_sidecars_and_increments_generation() {
        let mut arena = EntityArena::with_capacity(1);
        let partition_id = PartitionId(7);
        let version_one = VersionId(1);
        let version_two = VersionId(2);

        let (slot, generation, _) = arena.push_slot(super::super::SlotInit {
            partition_id,
            kind_id: KindId(11),
            version_id: version_one,
            extra: EntityExtra::default(),
        });
        assert_eq!(generation, 1);
        arena.extra[slot] = EntityExtra {
            structural_fingerprint: Some(crate::identity::data::StructuralFingerprint {
                family: Symbol(9),
                value: 42,
            }),
            lineage_id: Some(crate::identity::data::LineageId(12)),
            authoritative_aspect_state: None,
        };
        arena.retire(slot, version_two);
        arena.lifecycle[slot] = RecordLifecycleState::Reusable;
        arena.reset_slot(slot);

        let (_, reused_generation, reused) = arena.push_slot(super::super::SlotInit {
            partition_id,
            kind_id: KindId(12),
            version_id: VersionId(3),
            extra: EntityExtra::default(),
        });
        assert!(reused);
        assert_eq!(reused_generation, 2);
        assert!(arena.extra[slot].structural_fingerprint.is_none());
        assert!(arena.extra[slot].lineage_id.is_none());
        assert!(arena.extra[slot].authoritative_aspect_state.is_none());
    }

    #[test]
    fn reusing_relation_slot_replaces_endpoints_and_increments_generation() {
        let mut arena = RelationArena::with_capacity(1);
        let partition_id = PartitionId(3);
        let first = RelationEndpoints {
            source: EntityId::new(partition_id, 1, 1),
            target: EntityId::new(partition_id, 2, 1),
        };
        let second = RelationEndpoints {
            source: EntityId::new(partition_id, 3, 1),
            target: EntityId::new(partition_id, 4, 1),
        };

        let (slot, generation, _) = arena.push_slot(super::super::SlotInit {
            partition_id,
            kind_id: KindId(21),
            version_id: VersionId(1),
            extra: RelationExtra {
                endpoints: Some(first),
                authoritative_aspect_state: None,
            },
        });
        assert_eq!(generation, 1);
        arena.retire(slot, VersionId(2));
        arena.lifecycle[slot] = RecordLifecycleState::Reusable;
        arena.reset_slot(slot);

        let (_, reused_generation, reused) = arena.push_slot(super::super::SlotInit {
            partition_id,
            kind_id: KindId(22),
            version_id: VersionId(3),
            extra: RelationExtra {
                endpoints: Some(second.clone()),
                authoritative_aspect_state: None,
            },
        });
        assert!(reused);
        assert_eq!(reused_generation, 2);
        assert_eq!(arena.extra[slot].endpoints, Some(second));
        assert!(arena.extra[slot].authoritative_aspect_state.is_none());
    }

    #[test]
    fn get_rejects_id_from_different_partition_even_with_same_slot_and_generation() {
        let mut arena = RelationArena::with_capacity(1);
        let partition_id = PartitionId(3);
        let other_partition_id = PartitionId(4);
        let (slot, generation, _) = arena.push_slot(super::super::SlotInit {
            partition_id,
            kind_id: KindId(21),
            version_id: VersionId(1),
            extra: RelationExtra {
                endpoints: Some(RelationEndpoints {
                    source: EntityId::new(partition_id, 1, 1),
                    target: EntityId::new(partition_id, 2, 1),
                }),
                authoritative_aspect_state: None,
            },
        });

        let wrong_partition_id = RelationId::new(other_partition_id, slot as u64, generation);
        assert!(arena.get(&wrong_partition_id).is_none());
    }

    #[test]
    fn high_global_slot_materializes_one_row_and_leaves_gaps_absent() {
        let mut arena = EntityArena::with_capacity(0);
        let high_slot = 50_000;

        let reused = arena
            .write_reserved_slot(
                super::super::SlotInit {
                    partition_id: PartitionId(7),
                    kind_id: KindId(11),
                    version_id: VersionId(1),
                    extra: EntityExtra::default(),
                },
                high_slot,
                1,
            )
            .unwrap();

        assert!(!reused);
        assert_eq!(arena.slot_count(), 1);
        assert_eq!(arena.occupied_slots(), vec![high_slot]);
        assert!(arena.get_slot(0).is_none());
        assert!(arena.get_slot(high_slot - 1).is_none());
        assert!(arena.get_slot(high_slot).is_some());
        assert_eq!(arena.live_bitset.count_ones(), 1);
    }

    #[test]
    fn partition_payload_bytes_are_sensitive_to_nested_struct_aspect_capacity() {
        let short = entity_arena_with_authoritative_text("x");
        let long_text = "x".repeat(8_192);
        let long = entity_arena_with_authoritative_text(&long_text);

        let short_bytes = short.authoritative_allocation_bytes();
        let long_bytes = long.authoritative_allocation_bytes();
        let minimum_nested_growth = (long_text.len().saturating_sub(1) as u64) * 2;
        assert!(
            long_bytes >= short_bytes.saturating_add(minimum_nested_growth),
            "extra and version metadata own distinct nested aspect-state payloads"
        );
    }

    #[test]
    fn authoritative_arena_bytes_ignore_diagnostics_and_retention_perturbations() {
        let mut arena = entity_arena_with_authoritative_text("canonical");
        let baseline = arena.allocation_inventory();
        arena.diagnostics_enrichment[0].insert(Symbol(77), "diagnostic".repeat(512));
        arena.branch_pins.set(0, 128);
        arena.replay_pins.set(0, 64);
        arena.snapshot_pins.set(0, 32);
        let perturbed = arena.allocation_inventory();

        assert_eq!(perturbed.authoritative_bytes, baseline.authoritative_bytes);
        assert!(perturbed.diagnostic_bytes > baseline.diagnostic_bytes);
        assert!(perturbed.retention_metadata_bytes > baseline.retention_metadata_bytes);
        assert_eq!(perturbed.allocator_bookkeeping_bytes, 0);
    }

    fn entity_arena_with_authoritative_text(value: &str) -> EntityArena {
        let state = authoritative_text_state(value);
        let mut arena = EntityArena::with_capacity(1);
        arena.push_slot(super::super::SlotInit {
            partition_id: PartitionId::main(),
            kind_id: KindId(1),
            version_id: VersionId(1),
            extra: EntityExtra {
                structural_fingerprint: None,
                lineage_id: None,
                authoritative_aspect_state: Some(state),
            },
        });
        arena
    }

    fn authoritative_text_state(value: &str) -> AuthoritativeRecordAspectState {
        let contract = struct_string_contract();
        let nested = StructAspectValue::new([
            (
                FieldKey::new("title").expect("canonical field"),
                AspectValue::String(InternedString::Raw(value.to_owned())),
            ),
            (
                FieldKey::new("status").expect("canonical field"),
                AspectValue::String(InternedString::Raw(value.repeat(2))),
            ),
        ])
        .expect("canonical struct value");
        let TransitionOutcome::Success(validated) =
            validate_aspect_value(&contract, ContractValidationInput::Struct(nested))
        else {
            panic!("the nested payload validates");
        };
        let TransitionOutcome::Success(state) =
            admit_authoritative_record_aspect_state([validated])
        else {
            panic!("the nested payload admits as authoritative state");
        };
        let (state, _proofs, _basis) = state.into_parts().into_parts();
        state
    }

    fn struct_string_contract() -> AspectContract {
        let shape = aspects()
            .struct_fields()
            .required("title", ScalarAspectType::String)
            .optional("status", ScalarAspectType::String)
            .finish()
            .expect("canonical struct shape");
        aspects()
            .contract()
            .for_key(AspectKey::new("payload").expect("canonical key"))
            .identified_by(AspectIdentity(1))
            .at_revision(AspectContractRevision(1))
            .struct_aspect(shape)
    }
}
