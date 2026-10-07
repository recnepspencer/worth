use worth_proof::AdmittedBlobReleaseProof;

fn main() {
    let _forged = AdmittedBlobReleaseProof {
        store: [1; 16],
        object: [2; 16],
        generation: 3,
        publication_allocation_epoch: [4; 16],
        publication_record_ordinal: 5,
        publication_frame_sha256: [6; 32],
        issuer_evidence_sha256: [7; 32],
    };
}
