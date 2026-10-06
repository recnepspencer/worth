use crate::snapshot::{
    BridgeSnapshotReadError, SnapshotReadPacket, SnapshotReadPacketResult, TruthSnapshotIdentity,
};

pub trait TruthSnapshotReader: Send + Sync + 'static {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity;

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError>;

    /// Carries the caller's lease across Bridge correspondence. Readers with
    /// no dispatched work may use the serial implementation.
    fn read_packet_with_lease(
        &self,
        request: &SnapshotReadPacket,
        _lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.read_packet(request)
    }
}

impl<T> TruthSnapshotReader for Box<T>
where
    T: TruthSnapshotReader + ?Sized,
{
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        (**self).snapshot_identity()
    }

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        (**self).read_packet(request)
    }

    fn read_packet_with_lease(
        &self,
        request: &SnapshotReadPacket,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        (**self).read_packet_with_lease(request, lease)
    }
}

#[derive(Debug)]
pub struct BridgeSnapshotContext<R: TruthSnapshotReader> {
    snapshot: R,
    snapshot_identity: TruthSnapshotIdentity,
}

#[derive(Debug)]
pub struct AdmittedSnapshotContext<R: TruthSnapshotReader> {
    bound: BridgeSnapshotContext<R>,
}

impl<R: TruthSnapshotReader> BridgeSnapshotContext<R> {
    pub(crate) fn bind(snapshot: R) -> Self {
        let snapshot_identity = snapshot.snapshot_identity();
        Self {
            snapshot,
            snapshot_identity,
        }
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        &self.snapshot_identity
    }

    pub fn reader(&self) -> &R {
        &self.snapshot
    }

    pub fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.snapshot.read_packet(request)
    }

    pub fn read_packet_with_lease(
        &self,
        request: &SnapshotReadPacket,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        if let Some(stop) = BridgeSnapshotReadError::execution_stopped(lease) {
            return Err(stop);
        }
        let result = self.snapshot.read_packet_with_lease(request, lease)?;
        if let Some(stop) = BridgeSnapshotReadError::execution_stopped(lease) {
            return Err(stop);
        }
        Ok(result)
    }
}

impl<R: TruthSnapshotReader> AdmittedSnapshotContext<R> {
    pub(crate) fn admit_for(
        snapshot: BridgeSnapshotContext<R>,
        planned_identity: &TruthSnapshotIdentity,
    ) -> Result<Self, TruthSnapshotIdentity> {
        if snapshot.snapshot_identity() != planned_identity {
            return Err(snapshot.snapshot_identity().clone());
        }

        Ok(Self { bound: snapshot })
    }

    pub fn snapshot_identity(&self) -> &TruthSnapshotIdentity {
        self.bound.snapshot_identity()
    }

    pub fn read_packet(
        &self,
        request: &SnapshotReadPacket,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.bound.read_packet(request)
    }

    pub fn read_packet_with_lease(
        &self,
        request: &SnapshotReadPacket,
        lease: &worth_execution::ExecutionResourceLease<'_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.bound.read_packet_with_lease(request, lease)
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };
    use worth_execution::CancellationToken;

    use crate::snapshot::{
        BridgeSnapshotContext, BridgeSnapshotReadError, SnapshotReadPacket,
        SnapshotReadPacketResult, TruthSnapshotIdentity, TruthSnapshotReader,
    };

    struct StaticReader;

    impl TruthSnapshotReader for StaticReader {
        fn snapshot_identity(&self) -> TruthSnapshotIdentity {
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
        }

        fn read_packet(
            &self,
            _request: &SnapshotReadPacket,
        ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
            Ok(SnapshotReadPacketResult::new(
                crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a"),
                vec![],
            ))
        }
    }

    #[test]
    fn context_binds_snapshot_identity_from_reader() {
        let context = BridgeSnapshotContext::bind(StaticReader);

        assert_eq!(
            context.snapshot_identity().as_str(),
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a").as_str()
        );
    }

    struct LeaseTrackingReader(Arc<AtomicBool>);

    impl TruthSnapshotReader for LeaseTrackingReader {
        fn snapshot_identity(&self) -> TruthSnapshotIdentity {
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
        }

        fn read_packet(
            &self,
            _: &SnapshotReadPacket,
        ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
            panic!("leased read must reach the leased reader method")
        }

        fn read_packet_with_lease(
            &self,
            _: &SnapshotReadPacket,
            _: &worth_execution::ExecutionResourceLease<'_>,
        ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
            self.0.store(true, Ordering::SeqCst);
            Ok(SnapshotReadPacketResult::new(
                self.snapshot_identity(),
                vec![],
            ))
        }
    }

    #[test]
    fn admitted_snapshot_carries_lease_and_denies_pre_cancelled_reads() {
        let seen = Arc::new(AtomicBool::new(false));
        let identity = crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a");
        let snapshot = BridgeSnapshotContext::bind(
            Box::new(LeaseTrackingReader(Arc::clone(&seen))) as Box<dyn TruthSnapshotReader>,
        );
        let admitted = super::AdmittedSnapshotContext::admit_for(snapshot, &identity).unwrap();
        let lease = crate::snapshot::test_execution_lease(CancellationToken::new());
        admitted
            .read_packet_with_lease(&SnapshotReadPacket::new(vec![]), &lease)
            .unwrap();
        assert!(seen.load(Ordering::SeqCst));

        seen.store(false, Ordering::SeqCst);
        let cancelled = worth_execution::CancellationSource::new();
        cancelled.cancel();
        let lease = crate::snapshot::test_execution_lease(cancelled.token());
        let error = admitted
            .read_packet_with_lease(&SnapshotReadPacket::new(vec![]), &lease)
            .unwrap_err();
        assert_eq!(
            error.kind(),
            crate::snapshot::BridgeSnapshotReadErrorKind::ExecutionCancelled
        );
        assert!(!seen.load(Ordering::SeqCst));
    }
}
