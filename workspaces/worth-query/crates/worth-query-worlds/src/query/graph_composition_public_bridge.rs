pub use crate::facade::query::support;
pub use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
pub use worth_query::facade::runtime::{
    WorthQueryGraphCompositionLifecycleOutcomeKind, WorthQueryGraphCompositionProgramStepKind,
    WorthQueryInspection, WorthQueryLiveView, WorthQueryUnrefinedLiveShape,
};

pub use support::aspect_touch as touch;
pub use support::public_bridge_runtime::PublicBridgeRuntimeHarness;
pub use support::test_entity_identities::relational_test_entity_identity;

pub fn authored_text(
    value: impl Into<String>,
) -> worth_query::facade::runtime::WorthQueryAuthoredAspectValue {
    worth_query::facade::runtime::WorthQueryAuthoredAspectValue::string(value)
}

pub fn text(value: impl Into<String>) -> worth_foundational::facade::AspectValue {
    worth_foundational::facade::AspectValue::String(value.into().into())
}

pub fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}
