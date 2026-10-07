use super::*;

#[test]
fn retention_identity_exhaustion_survives_idempotency_snapshot_mapping() {
    assert_eq!(
            idempotency_snapshot_denial(
                crate::domain_computation::primary_graph::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted,
            ),
            WorthQueryProviderIdempotencyResolutionDenial::RetentionIdentityExhausted,
        );
}

#[test]
fn mapping_contract_pending_execution_retains_kind_and_ancestry() {
    use crate::domain_computation::primary_graph::WorthQueryManagedComputationResourceDenial as Resource;
    use crate::domain_computation::{
        WorthQueryProviderSessionDenialKind as Kind, WorthQueryProviderSessionFailure,
    };
    let kind = Kind::ExecutionResource {
        denial: Resource::ScratchCapacityExceeded,
        partition_identity: Some(3),
        policy_ancestor: Some(2),
    };
    let provider = pending_publication_denial(WorthQueryProviderSessionFailure::new(
        kind,
        crate::domain_computation::WorthQueryProviderSessionProtocolStage::Commit,
        "mapping contract",
        Default::default(),
    ));
    assert_eq!(
        provider,
        WorthQueryProviderIdempotencyResolutionDenial::ExecutionDenied(kind)
    );
    let application = crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenial::from_provider(provider);
    assert_eq!(application.kind(), crate::domain_computation::primary_graph::WorthQueryApplicationIdempotencyResolutionDenialKind::ExecutionDenied(kind));
}
