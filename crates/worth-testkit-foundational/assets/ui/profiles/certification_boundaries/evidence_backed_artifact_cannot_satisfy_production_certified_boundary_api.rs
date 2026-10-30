use worth_foundational::{
    bridge_production_certified_proof_bearing_artifact_trust_boundary,
    EvidenceBackedCertifiedProofBearingArtifact,
};

fn evidence_backed_artifact() -> EvidenceBackedCertifiedProofBearingArtifact<&'static str> {
    unimplemented!()
}

fn main() {
    let _ =
        bridge_production_certified_proof_bearing_artifact_trust_boundary(evidence_backed_artifact());
}
