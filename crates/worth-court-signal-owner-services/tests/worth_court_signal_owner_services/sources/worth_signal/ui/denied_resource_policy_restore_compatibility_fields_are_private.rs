use worth_signal::facade::core::{
    DeniedResourcePolicyRestoreCompatibility, ResourcePolicyCompatibilityReport, ResourcePolicyKind,
    ResourcePolicyRestoreCompatibilityDenialClass,
};

fn fake<T>() -> T {
    panic!("not executed in compile-fail fixtures")
}

fn main() {
    let _denial = DeniedResourcePolicyRestoreCompatibility {
        class: ResourcePolicyRestoreCompatibilityDenialClass::ParameterDigestDrift,
        primary_incompatible_kind: Some(ResourcePolicyKind::Timeout),
        compatibility: fake::<ResourcePolicyCompatibilityReport>(),
    };
}
