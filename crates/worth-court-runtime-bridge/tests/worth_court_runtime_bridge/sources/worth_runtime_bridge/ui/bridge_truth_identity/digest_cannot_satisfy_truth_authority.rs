use worth_runtime_bridge::facade::{
    bridge_truth_digest_identity_evidence_from_external_token, bridge_truth_external_identity_token,
    BridgeIdentityEvidence,
};

fn main() {
    let token = bridge_truth_external_identity_token("query-evidence:external");
    let digest = bridge_truth_digest_identity_evidence_from_external_token(token, todo!());

    let _evidence = BridgeIdentityEvidence::from_external_authority(digest);
}
