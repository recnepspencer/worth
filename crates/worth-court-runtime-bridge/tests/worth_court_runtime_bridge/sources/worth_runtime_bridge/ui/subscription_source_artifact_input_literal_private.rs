use worth_runtime_bridge::facade::{
    BridgeSubscriptionSourceArtifactInput, BridgeSubscriptionSourceArtifactKind,
};


fn main() {
    let _source_artifact = BridgeSubscriptionSourceArtifactInput {
        artifact_kind: BridgeSubscriptionSourceArtifactKind::Declaration,
        artifact_identity: sealed_authority_placeholder(),
        artifact_digest: sealed_authority_placeholder(),
    };
}

fn sealed_authority_placeholder<T>() -> T {
    panic!("compile-fail fixture never executes")
}
