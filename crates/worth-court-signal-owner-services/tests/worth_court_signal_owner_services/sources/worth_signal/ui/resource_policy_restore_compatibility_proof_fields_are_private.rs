use worth_signal::facade::core::{
    ResourcePolicyCompatibilityReport, ResourcePolicyRestoreCompatibilityProof,
};

fn fake<T>() -> T {
    panic!("not executed in compile-fail fixtures")
}

fn main() {
    let _proof = ResourcePolicyRestoreCompatibilityProof {
        compatibility: fake::<ResourcePolicyCompatibilityReport>(),
    };
}
