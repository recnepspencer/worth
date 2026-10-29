pub use std::sync::{Arc, Mutex};

pub use worth_proof::TransitionOutcome;
pub use worth_query::facade::{domain, installed};

pub use super::provider_fixture::{RemoteA, RemoteB, SelectiveProvider, SharedCommit};
pub use super::{atomic_definition, read_definition};
pub use worth_query_testkit_installed_operation_fixture::facade::query::suite::installed_operation_fixture::{
    configured_runtime_for_package, configured_runtime_for_partial_effect_package,
    federated_package, federated_touch_package, FederatedRead, GeometryDomain, ReadFamily,
};

pub fn assert_resource_mismatch(
    denial: &installed::operation::WorthQueryExecutionResourceAdmissionDenial,
    subject: &str,
) {
    assert_eq!(
        denial.kind(),
        &installed::operation::WorthQueryExecutionResourceAdmissionDenialKind::
            CancellationSafePointUnsupported
    );
    assert!(denial.detail().contains(subject));
    assert_eq!(denial.counters().provider_session_mints, 0);
}

pub fn mismatched_provider_resource_support() -> domain::WorthQueryExecutionResourceSupport {
    mismatched_provider_resource_support_with(domain::WorthQueryPartialEffectPosture::EffectFree)
}

pub fn mismatched_partial_effect_provider_resource_support(
) -> domain::WorthQueryExecutionResourceSupport {
    mismatched_provider_resource_support_with(
        domain::WorthQueryPartialEffectPosture::PartialEffectsMayRemain,
    )
}

pub fn mismatched_provider_resource_support_with(
    partial_effect_posture: domain::WorthQueryPartialEffectPosture,
) -> domain::WorthQueryExecutionResourceSupport {
    domain::WorthQueryExecutionResourceSupport::new(
        domain::WorthQueryExecutionProviderFamily::new("fixture-provider").unwrap(),
        domain::WorthQueryExecutionAccessProductFamily::new("fixture-access").unwrap(),
        domain::WorthQueryExecutionAllocatorFamily::new("fixture-arena").unwrap(),
        domain::WorthQueryExecutionResourceEnvelope::bounded(
            1_000_000,
            1_000_000,
            domain::WorthQueryExecutionMode::Synchronous,
            domain::WorthQueryCancellationSafePointFamily::new("incompatible-safe-point").unwrap(),
        )
        .with_partial_effect_posture(partial_effect_posture),
        std::sync::Arc::new(
            domain::WorthQueryFixedExecutionCapacity::mint("mismatched-graph-provider", 8).unwrap(),
        ),
    )
}
