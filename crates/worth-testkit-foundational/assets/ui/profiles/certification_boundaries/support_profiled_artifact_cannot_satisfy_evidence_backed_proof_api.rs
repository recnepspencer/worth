use worth_foundational::{
    certify_evidence_backed_proof_bearing_artifact, foundational_profile_certification_authority,
    SupportProfiledArtifact,
};

fn support_profiled_artifact() -> SupportProfiledArtifact<&'static str> {
    unimplemented!()
}

fn main() {
    let _ = certify_evidence_backed_proof_bearing_artifact(
        support_profiled_artifact(),
        foundational_profile_certification_authority(),
    );
}
