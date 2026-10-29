pub use crate::facade::query::support;
pub use worth_foundational::facade::{CanonicalFieldPath, FieldKey};
pub use worth_query::facade::foundation::WorthQueryEntityIdentity;
pub use worth_query::facade::runtime::{
    WorthQueryExistingEntityTarget, WorthQueryExistingTruthBindingAuthorityLabel,
    WorthQueryExistingTruthProbeDenialKind, WorthQueryExistingTruthProbeMode,
    WorthQueryExistingTruthProbeRequest, WorthQueryExistingTruthTargetBinding, WorthQueryLiveView,
    WorthQueryMutationAuthorityIdentity, WorthQueryRuntimeError, WorthQueryUnrefinedLiveShape,
};

pub use support::aspect_touch as touch;
pub use support::public_bridge_runtime::{
    public_bridge_runtime_bootstrap_invocation_count, public_graph_support_profile,
    reset_public_bridge_runtime_bootstrap_invocations, PublicBridgeRuntimeBootstrapPath,
    PublicBridgeRuntimeHarness,
};

pub fn public_entity_verified_profile(
) -> worth_query::facade::runtime::WorthQueryRuntimeSupportProfile {
    public_graph_support_profile().with_bridge_backed_verification_support(
        "probe_existing",
        "direct_entity_identity",
        true,
        true,
        None,
    )
}

pub fn existing_authority(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
        WorthQueryExistingTruthBindingAuthorityLabel::new(label)
            .expect("existing-truth authority label"),
    )
    .expect("existing-truth authority identity")
}

pub fn existing_task_binding(
    authority_label: &str,
    entity_token: &str,
) -> WorthQueryExistingTruthTargetBinding {
    WorthQueryExistingTruthTargetBinding::from_entity_target(
        WorthQueryExistingEntityTarget::new(
            existing_authority(authority_label),
            WorthQueryEntityIdentity::admit_authored_entity_token(
                worth_query::facade::foundation::QueryExternalIdentityToken::new(
                    std::sync::Arc::from(entity_token),
                ),
            ),
        )
        .expect("existing entity target should build")
        .in_target_collection("Task")
        .expect("existing entity target collection should build"),
    )
    .expect("binding should build")
}

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
