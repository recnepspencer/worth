use super::*;

use crate::schema::data::{DescriptorSemanticsVersion, SchemaVersionId};
use worth_foundational::facade::{
    AspectFieldLocator, AspectKey, CanonicalFieldPath, FieldKey, InternedString, LocatorAuthority,
};

#[test]
fn explicit_target_helper_builds_planned_packet() {
    let planned = PlannedQueryPacket::explicit_targets(
        "targets",
        QueryPlanContextId {
            runtime_instance_id: 11,
            snapshot_id: SnapshotId(5),
            version_id: VersionId(19),
            schema_version: SchemaVersionId(2),
            descriptor_semantics_version: DescriptorSemanticsVersion(1),
            evidence_basis: QueryPlanEvidenceBasis::CanonicalCommitEnvelope {
                commit_id: CommitId(13),
            },
        },
        vec![RecordRef::Entity(EntityId::new(PartitionId(7), 3, 1))],
    );

    assert_eq!(planned.label, "targets");
    assert_eq!(
        planned.locality,
        QueryLocalityClass::CrossPartitionTraversal
    );
    assert_eq!(
        planned.ordering,
        QueryOrderingContract::CanonicalRecordRefOrder
    );
    assert_eq!(
        planned.access_contract,
        QueryAccessContract::AuthoritativeStorageOnly
    );
    assert_eq!(planned.target_count_hint, 1);
    assert_eq!(
        planned.explicit_target_refs(),
        Some([RecordRef::Entity(EntityId::new(PartitionId(7), 3, 1))].as_slice())
    );
}

#[test]
fn explicit_target_helper_can_bind_single_partition_locality() {
    let planned = PlannedQueryPacket::explicit_targets_with_locality(
        "partitioned",
        QueryPlanContextId {
            runtime_instance_id: 1,
            snapshot_id: SnapshotId(2),
            version_id: VersionId(3),
            schema_version: SchemaVersionId(4),
            descriptor_semantics_version: DescriptorSemanticsVersion(5),
            evidence_basis: QueryPlanEvidenceBasis::CanonicalCommitEnvelope {
                commit_id: CommitId(21),
            },
        },
        vec![],
        QueryLocalityClass::SinglePartition {
            partition_id: PartitionId(42),
        },
    );

    assert_eq!(
        planned.locality,
        QueryLocalityClass::SinglePartition {
            partition_id: PartitionId(42)
        }
    );
}

#[test]
fn explicit_target_helper_generates_non_zero_deterministic_plan_key() {
    let context = QueryPlanContextId {
        runtime_instance_id: 11,
        snapshot_id: SnapshotId(7),
        version_id: VersionId(9),
        schema_version: SchemaVersionId(2),
        descriptor_semantics_version: DescriptorSemanticsVersion(1),
        evidence_basis: QueryPlanEvidenceBasis::CanonicalCommitEnvelope {
            commit_id: CommitId(34),
        },
    };

    let first = PlannedQueryPacket::explicit_targets(
        "explicit-target",
        context.clone(),
        vec![RecordRef::Entity(EntityId::new(PartitionId(7), 3, 1))],
    );
    let second = PlannedQueryPacket::explicit_targets(
        "explicit-target",
        context,
        vec![RecordRef::Entity(EntityId::new(PartitionId(7), 3, 1))],
    );

    assert_ne!(first.plan_key, DeterministicQueryPlanKey(0));
    assert_eq!(first.plan_key, second.plan_key);
}

#[test]
fn field_predicate_plan_keys_preserve_aspect_value_family() {
    let context = QueryPlanContextId {
        runtime_instance_id: 11,
        snapshot_id: SnapshotId(7),
        version_id: VersionId(9),
        schema_version: SchemaVersionId(2),
        descriptor_semantics_version: DescriptorSemanticsVersion(1),
        evidence_basis: QueryPlanEvidenceBasis::GenesisRuntimeBootstrap,
    };
    let field_locator = test_field_locator("counter", "count");

    let int_plan_key =
        field_equals_plan_key(&context, field_locator.clone(), AspectValue::Int64(1));
    let string_plan_key = field_equals_plan_key(
        &context,
        field_locator,
        AspectValue::String(InternedString::Raw("1".to_string())),
    );

    assert_ne!(int_plan_key, string_plan_key);
}

fn field_equals_plan_key(
    context: &QueryPlanContextId,
    field_locator: AspectFieldLocator,
    value: AspectValue,
) -> DeterministicQueryPlanKey {
    deterministic_query_plan_key(
        context,
        "field",
        &QueryScope::EntityFieldEquals {
            field_locator,
            value,
            partition_scope: None,
        },
        &QueryLocalityClass::CrossPartitionTraversal,
        QueryOrderingContract::CanonicalEntityIdOrder,
        QueryAccessContract::AuthoritativeStorageOnly,
        QueryExecutionShape::BulkPacketized,
        ReductionDiscipline::DeterministicMerge,
        0,
    )
}

fn test_field_locator(aspect: &str, field: &str) -> AspectFieldLocator {
    AspectFieldLocator::new(
        LocatorAuthority::Planned,
        AspectKey::new(aspect).expect("valid aspect key"),
        CanonicalFieldPath::single(FieldKey::new(field).expect("valid field key")),
    )
}
