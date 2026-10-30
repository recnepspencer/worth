use worth_runtime_bridge::facade::{
    bridge_truth_external_identity_token, bridge_truth_projection_identity_from_external_token,
    BridgeIdentityEvidence,
};

fn main() {
    let token = bridge_truth_external_identity_token("query-evidence:external");
    let projection = bridge_truth_projection_identity_from_external_token(token, "report-label");

    let _evidence = BridgeIdentityEvidence::from_external_authority(projection);
}
