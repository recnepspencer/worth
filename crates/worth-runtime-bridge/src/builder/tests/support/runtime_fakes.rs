use crate::adapter::{
    BridgeHistoricalLineageAuthority, BridgeHistoricalLineageRequest,
    BridgeHistoricalResolvedLineageIdentity, BridgeHistoricalResolvedRecordIdentity,
    BridgeSourceAdapter, CommittedPatchSource, ContinuityLineageSource, InvalidationSink,
    RelationalBridgeSourceError, SignalBridgeSinkError, SnapshotReadSource, TruthBranchHeadSource,
    TruthWritebackAuthority, TruthWritebackAuthorityError, TruthWritebackReceipt,
    TruthWritebackRequest,
};
use crate::delivery::BridgeDeliveryReceipt;
use crate::error::{BridgeLineageSourceError, BridgeLineageSourceErrorKind};
use crate::input::envelope::BridgeCommittedPatchEnvelope;
use crate::snapshot::{
    BridgeSnapshotReadError, SnapshotReadPacket, SnapshotReadPacketResult, TruthSnapshotIdentity,
    TruthSnapshotReader,
};
use crate::source::BridgeSourceCapabilitySet;
use crate::truth_identity_fixtures::{truth_commit, truth_patch, truth_snapshot};
use worth_foundational::facade::AspectKey;

pub(in crate::builder::tests) struct TestSource;

impl CommittedPatchSource for TestSource {
    fn authoritative_source_profile(
        &self,
    ) -> Option<crate::input::envelope::BridgeAuthoritativeSourceProfile> {
        Some(
            crate::input::envelope::BridgeAuthoritativeSourceProfile::new(
                99,
                "relational-adapter:99",
            )
            .expect("valid test source profile"),
        )
    }

    fn load_committed_patch(
        &self,
        _request: crate::adapter::RelationalCommittedPatchRequest,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::input::envelope::BridgeCommittedPatchEnvelope, RelationalBridgeSourceError>
    {
        unreachable!("builder tests do not load committed patch parts")
    }
}

impl SnapshotReadSource for TestSource {
    fn open_snapshot(
        &self,
        _identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        Ok(Box::new(TestSnapshotReader))
    }
}

impl TruthBranchHeadSource for TestSource {
    fn load_branch_head_patch(
        &self,
        branch_identity: &crate::input::envelope::TruthBranchIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<crate::input::envelope::BridgeCommittedPatchEnvelope, RelationalBridgeSourceError>
    {
        BridgeCommittedPatchEnvelope::new(
            crate::input::envelope::BridgeCommittedPatchEnvelopeIdentity::new(
                truth_commit(100),
                truth_patch(100),
                truth_snapshot(1, 1),
                branch_identity.clone(),
            ),
            vec![
                crate::input::envelope::BridgeCommittedPatchItem::with_target(
                    "entity-1",
                    crate::facade::BridgeCommittedPatchTarget::entity_field_path(
                        worth_foundational::facade::AspectLocator::new(
                            worth_foundational::facade::LocatorAuthority::Authoritative,
                            AspectKey::new("profile").expect("valid aspect key"),
                        ),
                        worth_foundational::facade::CanonicalFieldPath::single(
                            worth_foundational::facade::FieldKey::new("name".to_owned())
                                .expect("valid foundational field key"),
                        ),
                    ),
                ),
            ],
        )
        .map_err(|error| RelationalBridgeSourceError::new(error.to_string()))
    }
}

struct TestSnapshotReader;

impl TruthSnapshotReader for TestSnapshotReader {
    fn snapshot_identity(&self) -> TruthSnapshotIdentity {
        truth_snapshot(1, 1)
    }

    fn read_packet(
        &self,
        _request: &SnapshotReadPacket,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<SnapshotReadPacketResult, BridgeSnapshotReadError> {
        unreachable!("builder tests do not read snapshots")
    }
}

pub(in crate::builder::tests) struct TestSink;

impl InvalidationSink for TestSink {
    fn deliver_invalidation(
        &self,
        delivery: crate::routing::BridgeSignalInvalidationDelivery,
        _lease: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<BridgeDeliveryReceipt, SignalBridgeSinkError> {
        Ok(BridgeDeliveryReceipt::new(
            delivery.invalidation_targets().len(),
            delivery.source_snapshot().clone(),
        ))
    }
}

pub(in crate::builder::tests) struct TestLineageSource;

impl ContinuityLineageSource for TestLineageSource {
    fn historical_lineage(
        &self,
        request: BridgeHistoricalLineageRequest,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        BridgeHistoricalLineageAuthority::try_new(
            request.authority_basis().clone(),
            vec![BridgeHistoricalResolvedLineageIdentity::admit_bridge_owned(
                "lineage:test",
            )],
            vec![BridgeHistoricalResolvedRecordIdentity::admit_bridge_owned(
                "entity:test",
            )],
            vec![],
        )
    }
}

pub(in crate::builder::tests) struct TestUnsupportedLineageSource;

impl ContinuityLineageSource for TestUnsupportedLineageSource {
    fn historical_lineage(
        &self,
        _request: BridgeHistoricalLineageRequest,
    ) -> Result<BridgeHistoricalLineageAuthority, BridgeLineageSourceError> {
        Err(BridgeLineageSourceError::new(
            BridgeLineageSourceErrorKind::UnsupportedContinuityClass,
            "unsupported continuity class",
        ))
    }
}

pub(in crate::builder::tests) struct TestSourceAdapter {
    pub(in crate::builder::tests) capabilities: BridgeSourceCapabilitySet,
}

impl BridgeSourceAdapter for TestSourceAdapter {
    fn declared_capabilities(&self) -> BridgeSourceCapabilitySet {
        self.capabilities.clone()
    }

    fn open_snapshot(
        &self,
        _identity: &TruthSnapshotIdentity,
        _execution: worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Box<dyn TruthSnapshotReader>, RelationalBridgeSourceError> {
        unreachable!("builder tests do not materialize source snapshots")
    }
}

pub(in crate::builder::tests) struct TestWritebackAuthority;

impl TruthWritebackAuthority for TestWritebackAuthority {
    fn execute_writeback(
        &self,
        request: TruthWritebackRequest,
    ) -> Result<TruthWritebackReceipt, TruthWritebackAuthorityError> {
        Ok(TruthWritebackReceipt::new(
            crate::facade::BridgeWritebackOutcomeClass::AuthoritativeCommit,
            &request,
        ))
    }
}
