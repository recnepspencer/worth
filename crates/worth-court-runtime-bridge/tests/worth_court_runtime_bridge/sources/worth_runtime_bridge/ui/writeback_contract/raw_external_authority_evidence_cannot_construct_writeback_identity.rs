use worth_runtime_bridge::facade::BridgeWritebackEffectIdentity;

fn main() {
    let _ = BridgeWritebackEffectIdentity::from_external_authority_evidence("forged-effect");
}
