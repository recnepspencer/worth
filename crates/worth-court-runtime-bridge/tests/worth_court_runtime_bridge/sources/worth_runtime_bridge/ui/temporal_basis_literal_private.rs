use worth_foundational::facade::{CanonicalBasisSequence, CanonicalDerivedDigest};
use worth_runtime_bridge::facade::{
    AdmittedBridgeTemporalBasis, AdmittedBridgeTemporalSignalBasis,
    AdmittedBridgeTemporalTruthViewBasis, AdmittedBridgeTemporalWakeEvidence,
    BridgeTemporalBasisIdentity, BridgeTemporalBasisKind,
};

fn main() {
    let _ = AdmittedBridgeTemporalBasis {
        identity: BridgeTemporalBasisIdentity::new("temporal"),
        kind: BridgeTemporalBasisKind::Authoritative,
        truth_basis: unsafe_truth_basis(),
        signal_basis: unsafe_signal_basis(),
        wake_evidence: unsafe_wake(),
        canonical_basis: unsafe_basis(),
        canonical_digest: unsafe_digest(),
    };
}

fn unsafe_truth_basis() -> AdmittedBridgeTemporalTruthViewBasis {
    panic!("not constructed")
}

fn unsafe_signal_basis() -> AdmittedBridgeTemporalSignalBasis {
    panic!("not constructed")
}

fn unsafe_wake() -> AdmittedBridgeTemporalWakeEvidence {
    panic!("not constructed")
}

fn unsafe_basis() -> CanonicalBasisSequence {
    panic!("not constructed")
}

fn unsafe_digest() -> CanonicalDerivedDigest {
    panic!("not constructed")
}
