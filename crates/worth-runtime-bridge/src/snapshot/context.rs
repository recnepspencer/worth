use crate::snapshot::{
    BridgeSnapshotReadError, SnapshotReadPacket, SnapshotReadPacketResult, TruthSnapshotIdentity,
};

pub trait TruthSnapshotReader: Send + Sync + 'static {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity;

    fn read_packet(
        &self,
        request: &SnapshotReadPacket,
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError>;
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
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        (**self).read_packet(request, execution)
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
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        execution
            .in_scope(|_| {
                crate::execution_contact::admit(execution)
                    .map_err(BridgeSnapshotReadError::execution_denied)?;
                let result = self.snapshot.read_packet(request, execution)?;
                BridgeSnapshotReadError::checkpoint(execution)?;
                Ok(result)
            })
            .map_err(BridgeSnapshotReadError::execution_scope_denied)?
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
        execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        self.bound.read_packet(request, execution)
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
            _execution: worth_execution::ExecutionRequest<'_, '_>,
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
            execution: worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
            execution
                .in_scope(|lease| {
                    assert!(lease.is_some(), "reader must receive the caller's lease");
                    self.0.store(true, Ordering::SeqCst);
                })
                .map_err(BridgeSnapshotReadError::execution_scope_denied)?;
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
            .read_packet(
                &SnapshotReadPacket::new(vec![]),
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap();
        assert!(seen.load(Ordering::SeqCst));

        seen.store(false, Ordering::SeqCst);
        let cancelled = worth_execution::CancellationSource::new();
        cancelled.cancel();
        let lease = crate::snapshot::test_execution_lease(cancelled.token());
        let error = admitted
            .read_packet(
                &SnapshotReadPacket::new(vec![]),
                worth_execution::ExecutionRequest::leased(&lease),
            )
            .unwrap_err();
        assert_eq!(
            error.kind(),
            crate::snapshot::BridgeSnapshotReadErrorKind::ExecutionDenied(
                crate::error::BridgeExecutionDenial::Cancelled
            )
        );
        assert!(!seen.load(Ordering::SeqCst));
    }
    struct CancellingReader(worth_execution::CancellationSource);

    impl TruthSnapshotReader for CancellingReader {
        fn snapshot_identity(&self) -> TruthSnapshotIdentity {
            crate::truth_identity_fixtures::truth_snapshot_fixture("snapshot-a")
        }

        fn read_packet(
            &self,
            _: &SnapshotReadPacket,
            _: worth_execution::ExecutionRequest<'_, '_>,
        ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
            self.0.cancel();
            Ok(SnapshotReadPacketResult::new(
                self.snapshot_identity(),
                vec![],
            ))
        }
    }

    #[test]
    fn serial_snapshot_checks_cancellation_after_the_reader() {
        let cancellation = worth_execution::CancellationSource::new();
        let serial = crate::snapshot::test_serial_request().with_cancellation(cancellation.token());
        let context = BridgeSnapshotContext::bind(CancellingReader(cancellation));
        let error = context
            .read_packet(
                &SnapshotReadPacket::new(vec![]),
                worth_execution::ExecutionRequest::serial(&serial),
            )
            .unwrap_err();
        assert_eq!(
            error.kind(),
            crate::snapshot::BridgeSnapshotReadErrorKind::ExecutionDenied(
                crate::error::BridgeExecutionDenial::Cancelled,
            )
        );
    }
}
