use worth_runtime_bridge::facade::{
    bridge_truth_external_identity_token, BridgeIdentityEvidence,
};

fn main() {
    let token = bridge_truth_external_identity_token("query-evidence:external");
    let evidence = BridgeIdentityEvidence::from_external_authority(token);

    let _bridged = evidence.revalidate_bridge_retained_reference();
}
