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
