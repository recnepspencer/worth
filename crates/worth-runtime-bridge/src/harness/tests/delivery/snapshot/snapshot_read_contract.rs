use crate::facade::{
    BridgeDeliveryErrorKind, BridgeRouteRequest, BridgeSnapshotReadError,
    RelationalBridgeSourceError, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadSource,
    TruthSnapshotIdentity, TruthSnapshotReader,
};
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};
use worth_execution::ExecutionRequest;

struct OmitsSnapshotRecords<S>(S);

impl<S: SnapshotReadSource> SnapshotReadSource for OmitsSnapshotRecords<S> {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        let reader = self.0.open_snapshot(identity, execution)?;
        Ok(Box::new(OmittingReader(reader)))
    }
}

struct OmittingReader(Box<dyn TruthSnapshotReader>);

impl TruthSnapshotReader for OmittingReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.0.snapshot_identity()
    }

    fn read_packet(
        &self,
        packet: &SnapshotReadPacket,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        let observed = self.0.read_packet(packet, execution)?;
        assert!(
            !observed.records().is_empty(),
            "fault injection drops real records"
        );
        Ok(SnapshotReadPacketResult::new(
            observed.snapshot_identity().clone(),
            vec![],
        ))
    }
}

#[test]
fn bridge_snapshot_contract_rejects_missing_required_reads() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (commit, observation) = truth.commit_name(execution, "alice");
    let sink = RecordingSignalBridgeSink::default();
    let runtime = super::super::super::support::build_runtime_with_snapshot_source(
        truth.source.clone(),
        OmitsSnapshotRecords(truth.source),
        sink.clone(),
        NamedTruth::mapping(),
        Some(NamedTruth::aspect_mapping()),
    );
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
        .unwrap();
    let required = &route.read_packet().reads()[0];
    let target = required.target_identity().clone();
    let entity = required.entity_identity().into_owned();

    let error = runtime.deliver_invalidation(route, execution).unwrap_err();

    assert!(matches!(
        error.kind(), BridgeDeliveryErrorKind::SnapshotReadContractViolation(cause)
            if cause.kind() == crate::snapshot::BridgeSnapshotReadErrorKind::RecordCountMismatch
    ));
    assert_eq!(
        error.context().snapshot_identity(),
        Some(observation.snapshot_identity())
    );
    let coordinate = error.context().snapshot_read().unwrap();
    assert_eq!(coordinate.entity_identity(), entity);
    assert_eq!(coordinate.aspect_key().as_str(), "name");
    assert_eq!(coordinate.target_identity(), Some(&target));
    assert!(sink.deliveries().is_empty());
}

#[test]
fn leased_route_surfaces_cancellation_before_snapshot_reads() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (commit, _observation) = truth.commit_name(execution, "alice");
    let sink = RecordingSignalBridgeSink::default();
    let runtime =
        super::super::build_runtime(truth.source, sink.clone(), vec![NamedTruth::mapping()]);
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
        .unwrap();
    let cancellation = worth_execution::CancellationSource::new();
    cancellation.cancel();
    let lease = crate::snapshot::test_execution_lease(cancellation.token());

    let error = runtime
        .deliver_invalidation(route, ExecutionRequest::leased(&lease))
        .unwrap_err();

    assert_eq!(
        error.kind(),
        BridgeDeliveryErrorKind::ExecutionDenied(crate::error::BridgeExecutionDenial::Cancelled)
    );
    assert!(sink.deliveries().is_empty());
}
