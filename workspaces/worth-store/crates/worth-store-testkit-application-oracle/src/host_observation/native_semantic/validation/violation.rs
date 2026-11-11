//! Only typed identity fields change coordinates; every other verdict fact stays exact.
use super::*;
use worth_relational::facade::transactions::EntityReference;
use InvariantViolationFields as F;

fn endpoint(
    map: &mut DurableIdentityCorrespondence,
    h: &mut EntityReference,
    r: &EntityReference,
) -> Result<(), HostDivergence> {
    match (h, r) {
        (EntityReference::Existing(h), EntityReference::Existing(r)) => entity(map, h, *r),
        (EntityReference::Created(h), EntityReference::Created(r)) => {
            map.admit(
                Id::NativePartition(h.partition_id),
                Id::NativePartition(r.partition_id),
            )?;
            h.partition_id = r.partition_id;
            if h != r {
                return Err(bad());
            }
            Ok(())
        }
        (EntityReference::Created(_), EntityReference::Existing(_))
        | (EntityReference::Existing(_), EntityReference::Created(_)) => Err(bad()),
    }
}
fn endpoints(value: &F) -> Option<(&EntityReference, &EntityReference)> {
    match value {
        F::RelationEndpointKindMismatch { source, target, .. }
        | F::RelationEndpointKindSelfEdge { source, target, .. }
        | F::RelationCardinalityPair { source, target, .. }
        | F::RelationUniqueness { source, target, .. }
        | F::RelationSymmetry { source, target, .. }
        | F::PartitionIsolation { source, target, .. }
        | F::Acyclicity { source, target, .. } => Some((source, target)),
        F::None
        | F::MergedIntentLimit { .. }
        | F::SnapshotEntityLimit { .. }
        | F::UniqueEntityField { .. }
        | F::SidecarConsistency { .. }
        | F::RelationEndpointKindCrossContext { .. }
        | F::RelationCardinalityEndpoint { .. }
        | F::RelationEndpointDeletionIntegrity { .. }
        | F::StorageInconsistency { .. }
        | F::CustomInvariantFailure { .. }
        | F::CustomInvariantViolation { .. }
        | F::ConnectivityMinimum { .. }
        | F::DerivedRelationGraph { .. } => None,
    }
}
pub(super) fn correspond(
    map: &mut DurableIdentityCorrespondence,
    h: &mut F,
    r: &F,
) -> Result<(), HostDivergence> {
    if std::mem::discriminant(h) != std::mem::discriminant(r) {
        return Err(bad());
    }
    match h {
        F::SnapshotEntityLimit { version_id, .. } => {
            let F::SnapshotEntityLimit { version_id: r, .. } = r else {
                return Err(bad());
            };
            version(map, version_id, *r)?;
        }
        F::RelationEndpointKindMismatch { source, target, .. }
        | F::RelationEndpointKindSelfEdge { source, target, .. }
        | F::RelationCardinalityPair { source, target, .. }
        | F::RelationUniqueness { source, target, .. }
        | F::RelationSymmetry { source, target, .. }
        | F::PartitionIsolation { source, target, .. }
        | F::Acyclicity { source, target, .. } => {
            let Some((rs, rt)) = endpoints(r) else {
                return Err(bad());
            };
            endpoint(map, source, rs)?;
            endpoint(map, target, rt)?;
        }
        F::RelationCardinalityEndpoint { entity_id, .. } => {
            let F::RelationCardinalityEndpoint { entity_id: r, .. } = r else {
                return Err(bad());
            };
            endpoint(map, entity_id, r)?;
        }
        F::RelationEndpointDeletionIntegrity { entity_id, .. } => {
            let F::RelationEndpointDeletionIntegrity { entity_id: r, .. } = r else {
                return Err(bad());
            };
            entity(map, entity_id, *r)?;
        }
        F::StorageInconsistency { entity_id, .. } => {
            let F::StorageInconsistency { entity_id: r, .. } = r else {
                return Err(bad());
            };
            match (entity_id, r) {
                (None, None) => (),
                (Some(h), Some(r)) => entity(map, h, *r)?,
                (None, Some(_)) | (Some(_), None) => return Err(bad()),
            }
        }
        F::SidecarConsistency { partition_id, .. } => {
            let F::SidecarConsistency {
                partition_id: r, ..
            } = r
            else {
                return Err(bad());
            };
            map.admit(Id::NativePartition(*partition_id), Id::NativePartition(*r))?;
            *partition_id = *r;
        }
        F::RelationEndpointKindCrossContext {
            source_partition_id,
            target_partition_id,
            ..
        } => {
            let F::RelationEndpointKindCrossContext {
                source_partition_id: rs,
                target_partition_id: rt,
                ..
            } = r
            else {
                return Err(bad());
            };
            map.admit(
                Id::NativePartition(*source_partition_id),
                Id::NativePartition(*rs),
            )?;
            map.admit(
                Id::NativePartition(*target_partition_id),
                Id::NativePartition(*rt),
            )?;
            *source_partition_id = *rs;
            *target_partition_id = *rt;
        }
        F::ConnectivityMinimum { source, .. } => {
            let F::ConnectivityMinimum { source: r, .. } = r else {
                return Err(bad());
            };
            endpoint(map, source, r)?;
        }
        F::None
        | F::MergedIntentLimit { .. }
        | F::UniqueEntityField { .. }
        | F::CustomInvariantFailure { .. }
        | F::CustomInvariantViolation { .. }
        | F::DerivedRelationGraph { .. } => (),
    }
    if let F::PartitionIsolation { relation_id, .. } = h {
        let F::PartitionIsolation { relation_id: r, .. } = r else {
            return Err(bad());
        };
        match (relation_id, r) {
            (None, None) => (),
            (Some(h), Some(r)) => relation(map, h, *r)?,
            (None, Some(_)) | (Some(_), None) => return Err(bad()),
        }
    }
    if let F::PartitionIsolation {
        source_partition_id,
        target_partition_id,
        ..
    } = h
    {
        let F::PartitionIsolation {
            source_partition_id: rs,
            target_partition_id: rt,
            ..
        } = r
        else {
            return Err(bad());
        };
        map.admit(
            Id::NativePartition(*source_partition_id),
            Id::NativePartition(*rs),
        )?;
        map.admit(
            Id::NativePartition(*target_partition_id),
            Id::NativePartition(*rt),
        )?;
        *source_partition_id = *rs;
        *target_partition_id = *rt;
    }
    if let F::StorageInconsistency { partition_id, .. } = h {
        let F::StorageInconsistency {
            partition_id: r, ..
        } = r
        else {
            return Err(bad());
        };
        match (partition_id, r) {
            (None, None) => (),
            (Some(h), Some(r)) => {
                map.admit(Id::NativePartition(*h), Id::NativePartition(*r))?;
                *h = *r;
            }
            (None, Some(_)) | (Some(_), None) => return Err(bad()),
        }
    }
    Ok(())
}
