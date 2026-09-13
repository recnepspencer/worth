use worth_runtime_bridge::facade::{
    BridgeSnapshotReadError, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    TruthSnapshotIdentity, TruthSnapshotReader,
};

use super::identities::record_ref_from_identity_parts;
use super::snapshot_values::{
    export_entity_aspect_snapshot_value, export_relation_aspect_snapshot_value,
};

/// Shares the admitted observation's bounded owner retention obligation.
/// Reader clones neither reacquire a head nor create another obligation; the
/// last observation reference releases it. Bridge registration has a separate
/// external pin and may end while an already-open reader remains alive.
#[derive(Debug, Clone)]
pub(crate) struct RuntimePublicationSnapshotReader {
    runtime: crate::visibility::runtime_authority::RelationalVisibilityRuntimeAuthority,
    snapshot_identity: TruthSnapshotIdentity,
    observation: crate::mvcc::RelationalBranchObservation,
    partition: Option<crate::identity::data::PartitionId>,
}

impl RuntimePublicationSnapshotReader {
    pub(super) fn for_observation_authority(
        runtime: crate::visibility::runtime_authority::RelationalVisibilityRuntimeAuthority,
        snapshot_identity: TruthSnapshotIdentity,
        observation: crate::mvcc::RelationalBranchObservation,
        partition: Option<crate::identity::data::PartitionId>,
    ) -> Self {
        Self {
            runtime,
            snapshot_identity,
            observation,
            partition,
        }
    }
}

impl TruthSnapshotReader for RuntimePublicationSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.snapshot_identity.clone()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.runtime
            .with_runtime(|runtime| {
                read_packet(runtime, request, &self.observation, self.partition)
            })
            .map(|records| SnapshotReadPacketResult::new(self.snapshot_identity.clone(), records))
    }
}

fn read_packet(
    runtime: &crate::runtime::RelationalRuntime,
    request: &SnapshotReadPacket,
    observation: &crate::mvcc::RelationalBranchObservation,
    partition: Option<crate::identity::data::PartitionId>,
) -> Result<Vec<SnapshotReadRecord>, BridgeSnapshotReadError> {
    let read_truth = runtime.read_truth();
    let mut records = Vec::with_capacity(request.reads().len());
    for read in request.reads() {
        let identity_parts = read.relational_record_identity_parts().ok_or_else(|| {
            BridgeSnapshotReadError::new(
                "relational bridge snapshot reader requires typed record identity parts",
            )
        })?;
        if partition.is_some_and(|bound| bound.as_u32() != identity_parts.partition_id()) {
            return Err(BridgeSnapshotReadError::new(
                "relational bridge snapshot read is outside the source partition authority",
            ));
        }
        let record_ref = record_ref_from_identity_parts(identity_parts)
            .map_err(|error| BridgeSnapshotReadError::new(error.to_string()))?;
        let aspect_value = match record_ref {
            crate::transactions::data::RecordRef::Entity(entity_id) => {
                match read_entity(&read_truth, observation, entity_id) {
                    Some(record) => {
                        require_declared_aspect(
                            observation
                                .selected_root()
                                .schema_authority()
                                .entity_aspect_plan(record.kind.kind_id),
                            read.aspect_key(),
                        )?;
                        export_entity_aspect_snapshot_value(&record, read.aspect_key())
                    }
                    None => None,
                }
            }
            crate::transactions::data::RecordRef::Relation(relation_id) => {
                match read_relation(&read_truth, observation, relation_id) {
                    Some(record) => {
                        require_declared_aspect(
                            observation
                                .selected_root()
                                .schema_authority()
                                .relation_aspect_plan(record.kind.kind_id),
                            read.aspect_key(),
                        )?;
                        export_relation_aspect_snapshot_value(&record, read.aspect_key())
                    }
                    None => None,
                }
            }
        };
        records.push(match aspect_value {
            Some(value) => SnapshotReadRecord::for_request(read, value),
            None => SnapshotReadRecord::absent_for_request(read),
        });
    }
    Ok(records)
}

fn read_entity(
    read_truth: &crate::runtime::VisibilityReadContext<'_>,
    observation: &crate::mvcc::RelationalBranchObservation,
    entity_id: crate::identity::data::EntityId,
) -> Option<crate::storage::data::EntityReadRecord> {
    read_truth.authoritative_entity_record_for_id_from_exact_state(
        observation.selected_root().as_ref(),
        observation.selected_root().schema_authority().registry(),
        entity_id,
    )
}

fn read_relation(
    read_truth: &crate::runtime::VisibilityReadContext<'_>,
    observation: &crate::mvcc::RelationalBranchObservation,
    relation_id: crate::identity::data::RelationId,
) -> Option<crate::storage::data::RelationReadRecord> {
    read_truth.authoritative_relation_record_for_id_from_exact_state(
        observation.selected_root().as_ref(),
        observation.selected_root().schema_authority().registry(),
        relation_id,
    )
}

fn require_declared_aspect(
    plan: Option<&crate::schema::data::LoweredAspectContractPlan>,
    aspect: &worth_foundational::facade::AspectKey,
) -> Result<(), BridgeSnapshotReadError> {
    if plan.and_then(|plan| plan.contract_for(aspect)).is_some()
        || matches!(aspect.as_str(), "source" | "target" | "lifecycle")
    {
        Ok(())
    } else {
        Err(BridgeSnapshotReadError::new(format!(
            "relational bridge snapshot reader could not resolve aspect `{}` in authoritative schema",
            aspect.as_str()
        )))
    }
}
