mod intent_authority;
mod mutation_authority;
mod runtime_evidence;
mod schema;
mod signal;
mod snapshot;
mod source;
mod state;
mod subscription;

use worth_query::facade::runtime;

pub(crate) use state::{shared_source_state, SharedSourceState};

pub(crate) fn configure_product_projection_backend(
    builder: runtime::WorthQueryRuntimeBuilder,
    bridge_completion: std::sync::Arc<
        std::sync::OnceLock<worth_runtime_bridge::facade::RuntimeBridge>,
    >,
    source: SharedSourceState,
) -> runtime::WorthQueryRuntimeBuilder {
    builder
        .relational_product_bridge("worth-ui-product", move |source| {
            let bridge = super::bridge::platform_pulse_bridge(source)?;
            assert!(
                bridge_completion.set(bridge.clone()).is_ok(),
                "Query installs the product Bridge exactly once"
            );
            Ok(bridge)
        })
        .schema_adapter(schema::WorthUiScalarProjectionSchema)
        .source_adapter(source::WorthUiScalarProjectionSource::new(source.clone()))
        .write_authority(mutation_authority::WorthUiScalarProjectionMutationAuthority)
        .snapshot_identity(snapshot::WorthUiScalarProjectionSnapshotIdentity::new(
            source.clone(),
        ))
        .signal_sink(signal::WorthUiScalarProjectionSignalSink)
        .subscription_activation(subscription::WorthUiScalarProjectionSubscription)
        .preview_basis(runtime_evidence::WorthUiScalarProjectionUnsupportedPreview)
        .inspector_evidence(runtime_evidence::WorthUiScalarProjectionUnsupportedInspection)
        .intent_authority(intent_authority::WorthUiScalarProjectionIntentAuthority::new(source))
        .support_profile(product_projection_support_profile())
        .build_backend_from_parts()
}

fn product_projection_support_profile() -> runtime::WorthQueryRuntimeSupportProfile {
    use runtime::{
        WorthQueryAuthorityLane as Lane, WorthQueryRuntimeFacadeFamily as Family,
        WorthQueryRuntimeFamilySupport as Support,
    };

    runtime::WorthQueryRuntimeSupportProfile::new([
        Support::supported(
            Family::Read,
            [Lane::AuthoritativeTruth],
            [],
            ["worth-ui-product-live-read"],
        ),
        Support::supported(
            Family::Live,
            [Lane::AuthoritativeTruth],
            [],
            [
                "worth-ui-product-live-source",
                "worth-ui-product-subscription-activation",
            ],
        ),
        Support::supported(
            Family::AsyncResource,
            [Lane::AsyncResourceState],
            [],
            ["worth-ui-bridge-async-source-binding"],
        ),
        Support::supported(
            Family::MixedCauseDelivery,
            [Lane::BridgeExternalState],
            [],
            ["worth-ui-bridge-owner-issued-revalidation"],
        ),
        Support::supported(
            Family::Intent,
            [Lane::AuthoritativeTruth],
            [],
            ["worth-ui-product-intent-authority-v1"],
        ),
    ])
    .with_unsupported_batch_authority()
}
