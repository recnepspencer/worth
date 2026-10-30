use worth_runtime_bridge::facade::{
    bridge_truth_external_identity_token, BridgeIdentityEvidence,
};

fn main() {
    let token = bridge_truth_external_identity_token("query-evidence:external");

    let _evidence = BridgeIdentityEvidence::from_query_evidence_identity(token, todo!());
}
