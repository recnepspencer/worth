use worth_runtime_bridge::facade::{
    BridgeExistingTruthBindingAuthoritativeIdentity, BridgeNamingAttachmentIdentity,
    BridgeWritebackEffectIdentity,
};

fn main() {
    let _ = BridgeWritebackEffectIdentity::from_external_authority_evidence("effect");
    let _ = BridgeNamingAttachmentIdentity::from_external_authority_evidence("attachment");
    let _ = BridgeExistingTruthBindingAuthoritativeIdentity::from_external_authority_evidence(
        "existing-truth",
    );
}
