use crate::facade::{
    BridgeSnapshotReadError, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadRecord,
    TruthSnapshotIdentity, TruthSnapshotReader,
};

use super::identities::record_ref_from_identity_parts;
use super::snapshot_values::{
    export_entity_aspect_snapshot_value, export_relation_aspect_snapshot_value,
};
use worth_relational::facade::change_source::{
    RelationalObservationReadDenial, RelationalRuntimeHandle,
};
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::mvcc::RelationalBranchObservation;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::transactions::RecordRef;

/// Shares the admitted observation's bounded owner retention obligation.
/// Reader clones neither reacquire a head nor create another obligation; the
/// last observation reference releases it. Bridge registration has a separate
/// external pin and may end while an already-open reader remains alive.
#[derive(Debug, Clone)]
pub(crate) struct RuntimePublicationSnapshotReader {
    runtime: RelationalRuntimeHandle,
    snapshot_identity: TruthSnapshotIdentity,
    observation: RelationalBranchObservation,
    partition: Option<PartitionId>,
}

impl RuntimePublicationSnapshotReader {
    pub(super) fn for_observation_authority(
        runtime: RelationalRuntimeHandle,
        snapshot_identity: TruthSnapshotIdentity,
        observation: RelationalBranchObservation,
        partition: Option<PartitionId>,
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
    runtime: &RelationalRuntime,
    request: &SnapshotReadPacket,
    observation: &RelationalBranchObservation,
    partition: Option<PartitionId>,
) -> Result<Vec<SnapshotReadRecord>, BridgeSnapshotReadError> {
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
        let aspect = read.aspect_key();
        let aspect_value = match record_ref {
            RecordRef::Entity(entity_id) => {
                match runtime
                    .entity_record_at_observation(observation, entity_id)
                    .map_err(foreign_snapshot_observation)?
                {
                    Some(record) => {
                        require_declared_aspect(
                            observation.entity_kind_declares_aspect(record.kind.kind_id, aspect),
                            aspect,
                        )?;
                        export_entity_aspect_snapshot_value(&record, aspect)
                    }
                    None => None,
                }
            }
            RecordRef::Relation(relation_id) => {
                match runtime
                    .relation_record_at_observation(observation, relation_id)
                    .map_err(foreign_snapshot_observation)?
                {
                    Some(record) => {
                        require_declared_aspect(
                            observation.relation_kind_declares_aspect(record.kind.kind_id, aspect),
                            aspect,
                        )?;
                        export_relation_aspect_snapshot_value(&record, aspect)
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

fn foreign_snapshot_observation(
    denial: RelationalObservationReadDenial,
) -> BridgeSnapshotReadError {
    BridgeSnapshotReadError::new(format!(
        "relational bridge snapshot reader read an observation from another runtime: {denial:?}"
    ))
}

/// Relation endpoints and lifecycle are readable on every record; any other
/// aspect must be declared by the retained schema.
fn require_declared_aspect(
    declared: bool,
    aspect: &worth_foundational::facade::AspectKey,
) -> Result<(), BridgeSnapshotReadError> {
    if declared || matches!(aspect.as_str(), "source" | "target" | "lifecycle") {
        Ok(())
    } else {
        Err(BridgeSnapshotReadError::new(format!(
            "relational bridge snapshot reader could not resolve aspect `{}` in authoritative schema",
            aspect.as_str()
        )))
    }
}
