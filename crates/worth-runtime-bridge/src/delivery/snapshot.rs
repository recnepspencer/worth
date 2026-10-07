use crate::error::{
    BridgeDeliveryError, BridgeDeliveryErrorKind, BridgeErrorContext, BridgeSnapshotReadCoordinate,
};
use crate::facade::RuntimeBridge;
use crate::snapshot::{
    AdmittedSnapshotContext, BridgeSnapshotContext, SnapshotReadPacket, TruthSnapshotReader,
};

pub(crate) fn open_planned_snapshot(
    runtime: &RuntimeBridge,
    snapshot_identity: &crate::snapshot::TruthSnapshotIdentity,
) -> Result<AdmittedSnapshotContext<Box<dyn TruthSnapshotReader>>, BridgeDeliveryError> {
    let snapshot_reader = open_snapshot_reader(runtime, snapshot_identity).map_err(|error| {
        BridgeDeliveryError::new(
            BridgeDeliveryErrorKind::SnapshotAcquisitionFailure,
            format!(
                "Bridge failed to open snapshot `{}`: {error}",
                snapshot_identity.as_str()
            ),
        )
        .with_context(BridgeErrorContext::snapshot(snapshot_identity.clone()))
    })?;
    let snapshot = BridgeSnapshotContext::bind(snapshot_reader);
    let admitted = AdmittedSnapshotContext::admit_for(snapshot, snapshot_identity).map_err(
        |bound_snapshot_identity| {
            BridgeDeliveryError::new(
                BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
                format!(
                    "Snapshot reader bound `{}` but planned route required `{}`.",
                    bound_snapshot_identity.as_str(),
                    snapshot_identity.as_str()
                ),
            )
            .with_context(BridgeErrorContext::snapshot(snapshot_identity.clone()))
        },
    )?;

    if admitted.snapshot_identity() != snapshot_identity {
        return Err(BridgeDeliveryError::new(
            BridgeDeliveryErrorKind::SnapshotIdentityMismatch,
            format!(
                "Snapshot reader bound `{}` but planned route required `{}`.",
                admitted.snapshot_identity().as_str(),
                snapshot_identity.as_str()
            ),
        )
        .with_context(BridgeErrorContext::snapshot(snapshot_identity.clone())));
    }

    Ok(admitted)
}

pub(super) fn open_snapshot_reader(
    runtime: &RuntimeBridge,
    snapshot_identity: &crate::snapshot::TruthSnapshotIdentity,
) -> Result<Box<dyn TruthSnapshotReader>, crate::adapter::RelationalBridgeSourceError> {
    if let Some(pool) = runtime.snapshot_reader_pool.as_ref() {
        let pool = std::sync::Arc::clone(pool);
        let reader = pool.acquire(snapshot_identity)?;
        return Ok(Box::new(PooledTruthSnapshotReader::new(pool, reader)));
    }

    runtime
        .snapshot_read_source
        .open_snapshot(snapshot_identity)
}

struct PooledTruthSnapshotReader {
    pool: std::sync::Arc<dyn crate::adapter::SnapshotReaderPool>,
    reader: Option<Box<dyn TruthSnapshotReader>>,
}

impl PooledTruthSnapshotReader {
    fn new(
        pool: std::sync::Arc<dyn crate::adapter::SnapshotReaderPool>,
        reader: Box<dyn TruthSnapshotReader>,
    ) -> Self {
        Self {
            pool,
            reader: Some(reader),
        }
    }

    fn reader(&self) -> &dyn TruthSnapshotReader {
        self.reader
            .as_deref()
            .expect("pooled snapshot reader should be present while borrowed")
    }
}

impl TruthSnapshotReader for PooledTruthSnapshotReader {
    fn snapshot_identity(&self) -> crate::snapshot::TruthSnapshotIdentity {
        self.reader().snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<crate::snapshot::SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError>
    {
        self.reader().read_packet(request)
    }

    fn read_packet_with_lease(
        &self,
        request: &SnapshotReadPacket,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<crate::snapshot::SnapshotReadPacketResult, crate::snapshot::BridgeSnapshotReadError>
    {
        self.reader().read_packet_with_lease(request, lease)
    }
}

impl Drop for PooledTruthSnapshotReader {
    fn drop(&mut self) {
        if let Some(reader) = self.reader.take() {
            self.pool.release(reader);
        }
    }
}

pub(super) fn first_snapshot_read_coordinate(
    packet: &SnapshotReadPacket,
) -> BridgeSnapshotReadCoordinate {
    let read = &packet.reads()[0];
    match read.slice_kind() {
        Some(slice_kind) => BridgeSnapshotReadCoordinate::new_subscription_slice(
            read.correlation_id().clone(),
            read.entity_identity(),
            read.aspect_key().clone(),
            read.target_identity().clone(),
            slice_kind.clone(),
        ),
        None => BridgeSnapshotReadCoordinate::new_coarse(
            read.correlation_id().clone(),
            read.entity_identity(),
            read.aspect_key().clone(),
        ),
    }
}
