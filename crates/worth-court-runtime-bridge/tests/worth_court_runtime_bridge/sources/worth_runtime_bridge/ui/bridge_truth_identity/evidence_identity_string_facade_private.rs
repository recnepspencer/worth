use worth_runtime_bridge::facade::{
    bridge_truth_external_identity_token, BridgeIdentityEvidence,
};

fn main() {
    let evidence = BridgeIdentityEvidence::from_external_authority(
        bridge_truth_external_identity_token("bridge-evidence"),
    );

    let _text = evidence.as_str();
}
