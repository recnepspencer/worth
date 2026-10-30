pub use worth_query_testkit_installed_operation_fixture::facade::query::support::public_bridge_runtime::public_graph_support_profile;
pub use worth_foundational::facade::{AspectValue, CanonicalFieldPath, FieldKey};
pub use worth_query::facade::runtime::{
    WorthQueryAuthoredAspectValue, WorthQueryContinuityPriorAuthorityLabel,
    WorthQueryContinuitySuccessorAuthorityLabel, WorthQueryExistingTruthBindingAuthorityLabel,
    WorthQueryMutationAuthorityIdentity, WorthQueryNamingAttachmentAuthorityLabel,
    WorthQueryNamingPriorAuthorityLabel, WorthQueryNamingTargetAuthorityLabel,
    WorthQueryRuntimeSupportProfile,
};

pub fn public_multi_verified_relation_profile() -> WorthQueryRuntimeSupportProfile {
    ["update_existing_verified", "delete_existing_verified"]
        .into_iter()
        .fold(
            public_graph_support_profile(),
            |profile, operation_family| {
                profile.with_bridge_backed_verification_support(
                    operation_family,
                    "direct_relation_identity",
                    true,
                    true,
                    None,
                )
            },
        )
}

pub fn existing_authority(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::existing_truth_binding_authority(
        WorthQueryExistingTruthBindingAuthorityLabel::new(label)
            .expect("existing-truth authority label"),
    )
    .expect("existing-truth authority identity")
}

pub fn naming_attachment(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::naming_attachment(
        WorthQueryNamingAttachmentAuthorityLabel::new(label).expect("naming attachment label"),
    )
    .expect("naming attachment identity")
}

pub fn naming_prior(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::naming_prior_authority(
        WorthQueryNamingPriorAuthorityLabel::new(label).expect("naming prior label"),
    )
    .expect("naming prior identity")
}

pub fn naming_target(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::naming_target_authority(
        WorthQueryNamingTargetAuthorityLabel::new(label).expect("naming target label"),
    )
    .expect("naming target identity")
}

pub fn continuity_prior(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::continuity_prior_authority(
        WorthQueryContinuityPriorAuthorityLabel::new(label).expect("continuity prior label"),
    )
    .expect("continuity prior identity")
}

pub fn continuity_successor(label: &str) -> WorthQueryMutationAuthorityIdentity {
    WorthQueryMutationAuthorityIdentity::continuity_successor_authority(
        WorthQueryContinuitySuccessorAuthorityLabel::new(label)
            .expect("continuity successor label"),
    )
    .expect("continuity successor identity")
}

pub fn authored_text(value: impl Into<String>) -> WorthQueryAuthoredAspectValue {
    WorthQueryAuthoredAspectValue::string(value)
}

pub fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(value.into().into())
}

pub fn field_path(path: &str) -> CanonicalFieldPath {
    CanonicalFieldPath::new(
        path.split('.').map(|segment| {
            FieldKey::new(segment).expect("test field path segment should be valid")
        }),
    )
    .expect("test field path should be non-empty")
}
