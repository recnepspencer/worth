use crate::runtime::tests::support::*;

pub(in crate::runtime::tests) fn test_product_world_resources() -> WorthQueryProductWorldResources {
    WorthQueryProductWorldResources::install(
        RuntimeWorldBudgetInstallation {
            branches: RuntimeWorldBranchBudgetInstallation {
                live_product_branches: 128,
            },
            history: RuntimeWorldHistoryBudgetInstallation {
                retained_composite_commits: 1_024,
                history_metadata_bytes: 16 * 1024 * 1024,
            },
            observations: RuntimeWorldObservationBudgetInstallation {
                active_observations: 512,
            },
            publication: RuntimeWorldPublicationBudgetInstallation {
                active_publication_attempts: 128,
            },
            recovery: RuntimeWorldRecoveryBudgetInstallation {
                retained_product_unpublished_records: 128,
                retained_partial_metadata_bytes: 16 * 1024 * 1024,
            },
            retention: RuntimeWorldRetentionBudgetInstallation {
                unique_exact_component_pins: 1_024,
                in_flight_pin_acquisition_reservations: 256,
            },
            custody: RuntimeWorldCustodyBudgetInstallation {
                owner_created_component_custody_records: 256,
            },
        },
        WorthQueryProductWorldClock::start(),
    )
    .expect("the test Product World resources are valid")
}

pub(in crate::runtime::tests) fn bridge_backed_runtime_with_support(
    profile: WorthQueryRuntimeSupportProfile,
) -> WorthQueryRuntime {
    bridge_backed_runtime_with_support_and_intent_authority(profile, TestIntentAuthority)
}

pub(in crate::runtime::tests) fn bridge_runtime_with_support(
    profile: WorthQueryRuntimeSupportProfile,
) -> WorthQueryRuntime {
    bridge_backed_runtime_with_support(profile)
}

pub(in crate::runtime::tests) fn bridge_backed_runtime_with_support_and_intent_authority<
    T: WorthQueryIntentAuthorityAdapter + 'static,
>(
    profile: WorthQueryRuntimeSupportProfile,
    intent_authority: T,
) -> WorthQueryRuntime {
    bridge_backed_runtime_builder(profile)
        .intent_authority(intent_authority)
        .build_backend_from_parts()
        .build()
        .expect("complete bridge-backed runtime test support should build")
}

pub(in crate::runtime::tests) fn bridge_runtime_with_support_and_intent_authority<
    T: WorthQueryIntentAuthorityAdapter + 'static,
>(
    profile: WorthQueryRuntimeSupportProfile,
    intent_authority: T,
) -> WorthQueryRuntime {
    bridge_backed_runtime_with_support_and_intent_authority(profile, intent_authority)
}

pub(in crate::runtime::tests) fn bridge_backed_runtime_with_existing_truth_verification(
    profile: WorthQueryRuntimeSupportProfile,
    adapter: TestExistingTruthVerificationAdapter,
) -> WorthQueryRuntime {
    bridge_backed_runtime_builder(profile)
        .existing_truth_verification(adapter)
        .build_backend_from_parts()
        .build()
        .expect("complete bridge-backed runtime test support should build")
}

pub(in crate::runtime::tests) fn bridge_runtime_with_support_and_existing_truth_verification(
    profile: WorthQueryRuntimeSupportProfile,
    adapter: TestExistingTruthVerificationAdapter,
) -> WorthQueryRuntime {
    bridge_backed_runtime_with_existing_truth_verification(profile, adapter)
}

pub(in crate::runtime::tests) fn intent_support_profile() -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::bridge_backed(
        "test-subscription-activation",
        "test-preview-basis",
        "test-inspector-evidence",
    )
    .with_family_support(WorthQueryRuntimeFamilySupport::supported(
        WorthQueryRuntimeFacadeFamily::Intent,
        [
            WorthQueryAuthorityLane::AuthoritativeTruth,
            WorthQueryAuthorityLane::BranchLocalTruth,
            WorthQueryAuthorityLane::PreviewTruth,
        ],
        [],
        ["test-intent-authority"],
    ))
}

pub(in crate::runtime::tests) fn bridge_verified_direct_relation_profile(
    operation_family: &str,
) -> WorthQueryRuntimeSupportProfile {
    WorthQueryRuntimeSupportProfile::bridge_backed(
        "test-subscription-activation",
        "test-preview-basis",
        "test-inspector-evidence",
    )
    .with_bridge_backed_verification_support(
        operation_family,
        "direct_relation_identity",
        true,
        true,
        None,
    )
}

fn bridge_backed_runtime_builder(
    profile: WorthQueryRuntimeSupportProfile,
) -> WorthQueryRuntimeBuilder {
    complete_backend_from_parts_builder().support_profile(profile)
}

pub(in crate::runtime::tests) fn complete_backend_from_parts_builder() -> WorthQueryRuntimeBuilder {
    complete_test_backend(test_product_runtime_builder())
}

pub(in crate::runtime::tests) fn complete_query_owned_backend_from_parts_builder(
) -> WorthQueryRuntimeBuilder {
    complete_test_backend(query_owned_product_runtime_builder())
}

fn complete_test_backend(builder: WorthQueryRuntimeBuilder) -> WorthQueryRuntimeBuilder {
    builder
        .schema_adapter(TestSchemaAdapter)
        .source_adapter(TestSourceAdapter::default())
        .snapshot_identity(TestSnapshotIdentityAdapter)
        .write_authority(TestWriteAuthority)
        .signal_sink(TestSignalSink)
        .subscription_activation(TestSubscriptionActivation)
        .preview_basis(TestPreviewBasis)
        .inspector_evidence(TestInspectorEvidence)
        .aspect_contracts(stateful_bridge_aspect_contracts())
        .expect("complete test backend aspect contracts should admit")
}

pub(in crate::runtime::tests) fn test_product_runtime_builder() -> WorthQueryRuntimeBuilder {
    let product = test_product_root();
    WorthQueryRuntime::builder(test_product_world_resources())
        .runtime_bridge(product.bridge)
        .relational_source_owner(product.source)
        .conditional_execution_resources(WorthQueryConditionalExecutionResources::development())
}

pub(in crate::runtime::tests) fn query_owned_product_runtime_builder() -> WorthQueryRuntimeBuilder {
    WorthQueryRuntime::builder(test_product_world_resources())
        .relational_product_bridge("high-level-query-owned-product", |source| {
            build_test_product_bridge(source, true)
        })
        .conditional_execution_resources(WorthQueryConditionalExecutionResources::development())
}
