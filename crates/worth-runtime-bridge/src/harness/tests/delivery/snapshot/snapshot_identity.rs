use crate::facade::{
    BridgeDeliveryErrorKind, BridgeFailureClass, BridgeRouteRequest, BridgeSnapshotReadError,
    RelationalBridgeSourceError, SnapshotReadPacket, SnapshotReadPacketResult, SnapshotReadSource,
    TruthSnapshotIdentity, TruthSnapshotReader,
};
use crate::harness::fixtures::{NamedTruth, RecordingSignalBridgeSink};
use worth_execution::ExecutionRequest;

struct SubstitutesSnapshotIdentity<S> {
    source: S,
    wrong_identity: TruthSnapshotIdentity,
}

impl<S: SnapshotReadSource> SnapshotReadSource for SubstitutesSnapshotIdentity<S> {
    fn open_snapshot(
        &self,
        identity: &TruthSnapshotIdentity,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(IdentitySubstitutingReader {
            reader: self.source.open_snapshot(identity, execution)?,
            wrong_identity: self.wrong_identity.clone(),
        }))
    }
}

struct IdentitySubstitutingReader {
    reader: Box<dyn TruthSnapshotReader>,
    wrong_identity: TruthSnapshotIdentity,
}

impl TruthSnapshotReader for IdentitySubstitutingReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        self.reader.snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        execution: ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        let result = self.reader.read_packet(request, execution)?;
        Ok(SnapshotReadPacketResult::new(
            self.wrong_identity.clone(),
            result.records().to_vec(),
        ))
    }
}

#[test]
fn bridge_snapshot_identity_mismatch_fails_explicitly() {
    let serial = crate::snapshot::test_serial_request();
    let execution = ExecutionRequest::serial(&serial);
    let truth = NamedTruth::new();
    let (commit, original) = truth.commit_name(execution, "alice");
    let (_, other) = truth.commit_name(execution, "bob");
    let expected = original.snapshot_identity();
    assert_ne!(expected, other.snapshot_identity());

    let sink = RecordingSignalBridgeSink::default();
    let runtime = super::super::super::support::build_runtime_with_snapshot_source(
        truth.source.clone(),
        SubstitutesSnapshotIdentity {
            source: truth.source,
            wrong_identity: other.snapshot_identity().clone(),
        },
        sink.clone(),
        NamedTruth::mapping(),
        None,
    );
    let route = runtime
        .plan_committed_patch(BridgeRouteRequest::for_commit(commit), execution)
        .unwrap();

    let denial = runtime.deliver_invalidation(route, execution).unwrap_err();
    assert_eq!(
        denial.kind(),
        BridgeDeliveryErrorKind::SnapshotIdentityMismatch
    );
    let failure = runtime.diagnostics().last_failure_record().unwrap();
    assert_eq!(
        failure.failure_class(),
        &BridgeFailureClass::Delivery(BridgeDeliveryErrorKind::SnapshotIdentityMismatch)
    );
    assert_eq!(failure.context().snapshot_identity(), Some(expected));
    assert_eq!(failure.counters().snapshot_identity_mismatch_count(), 1);
    assert!(sink.deliveries().is_empty());
}
