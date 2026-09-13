use worth_store::physical_runtime::{
    IntegrityAdmittedRecoveryWalSegment, ObservedWalArtifact,
};
use worth_store::physical_runtime::recovery_wal::WalSegmentArtifactIdentity;

fn forge(observed: &ObservedWalArtifact) -> IntegrityAdmittedRecoveryWalSegment {
    let identity = WalSegmentArtifactIdentity::parse("segment-1-generation-1.wal").unwrap();
    IntegrityAdmittedRecoveryWalSegment::from_complete_frames(
        observed,
        identity,
        Vec::new(),
    )
    .unwrap()
}

fn main() {}
