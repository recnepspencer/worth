use super::SUBSCRIPTION_IDENTITY_SCOPE;
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{WorthQueryEvidenceIdentity, WorthQueryEvidenceTag};

pub(in crate::subscription) fn typed_identity_drift(
    left: &WorthQueryEvidenceIdentity,
    right: &WorthQueryEvidenceIdentity,
) -> bool {
    !matches!(left.eq_same_scheme(right), Ok(true))
}

pub(in crate::subscription) fn active_lane_identity(
    activation_identity: &WorthQueryEvidenceIdentity,
    admission_identity: &WorthQueryEvidenceIdentity,
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_declaration_identity: &WorthQueryEvidenceIdentity,
    future_selection_projection_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    scoped_declaration_basis_digest: &str,
    scoped_activation_basis_digest: &str,
    checkpoint_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    lifecycle_posture: &str,
    delivery_posture: &str,
    lookup_class: &str,
    allocation_policy: &str,
    registry_lookup_width: usize,
    fanout_width: usize,
    allocation_scope_width: usize,
    performance_receipt_identity: &WorthQueryEvidenceIdentity,
    counters_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "active_subscription_lane_v1",
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("activation"),
            activation_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity)
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("query_declaration"),
            query_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("bridge_declaration"),
            bridge_declaration_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("future_selection"),
            future_selection_projection_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_binding_identity)
        .field_shape(
            WorthQueryEvidenceTag::new("scoped_declaration_basis"),
            scoped_declaration_basis_digest,
        )
        .field_shape(
            WorthQueryEvidenceTag::new("scoped_activation_basis"),
            scoped_activation_basis_digest,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("checkpoint"),
            checkpoint_identity,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("signal_strategy"),
            signal_strategy_identity,
        )
        .field_shape(WorthQueryEvidenceTag::new("lifecycle"), lifecycle_posture)
        .field_shape(WorthQueryEvidenceTag::new("delivery"), delivery_posture)
        .field_shape(WorthQueryEvidenceTag::new("lookup"), lookup_class)
        .field_shape(WorthQueryEvidenceTag::new("allocation"), allocation_policy)
        .field_usize(
            WorthQueryEvidenceTag::new("budget_registry"),
            registry_lookup_width,
        )
        .field_usize(WorthQueryEvidenceTag::new("budget_fanout"), fanout_width)
        .field_usize(
            WorthQueryEvidenceTag::new("budget_allocation"),
            allocation_scope_width,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("performance"),
            performance_receipt_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("counters"), counters_identity)
        .seal()
}

pub(in crate::subscription) fn scale_counter_snapshot_identity(
    fixture_size: &str,
    fixture_row_count: u64,
    activation_identity: &WorthQueryEvidenceIdentity,
    admission_identity: &WorthQueryEvidenceIdentity,
    counter_identity: &WorthQueryEvidenceIdentity,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_subscription_scale_counter_snapshot_v1",
        )
        .field_shape(WorthQueryEvidenceTag::new("fixture_size"), fixture_size)
        .field_usize(
            WorthQueryEvidenceTag::new("fixture_row_count"),
            fixture_row_count as usize,
        )
        .field_evidence_identity(
            WorthQueryEvidenceTag::new("activation"),
            activation_identity,
        )
        .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity)
        .field_evidence_identity(WorthQueryEvidenceTag::new("counter"), counter_identity)
        .seal()
}

pub(in crate::subscription) fn scale_slope_report_identity(
    activation_identity: &WorthQueryEvidenceIdentity,
    admission_identity: &WorthQueryEvidenceIdentity,
    small_snapshot_identity: &WorthQueryEvidenceIdentity,
    medium_snapshot_identity: &WorthQueryEvidenceIdentity,
    large_snapshot_identity: &WorthQueryEvidenceIdentity,
    small_row_count: u64,
    medium_row_count: u64,
    large_row_count: u64,
    structural_counter_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_scale_slope_report_v1",
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("activation"),
                activation_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("small_snapshot"),
                small_snapshot_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("medium_snapshot"),
                medium_snapshot_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("large_snapshot"),
                large_snapshot_identity,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("small_row_count"),
                small_row_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("medium_row_count"),
                medium_row_count as usize,
            )
            .field_usize(
                WorthQueryEvidenceTag::new("large_row_count"),
                large_row_count as usize,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("structural_counter"),
                structural_counter_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}

pub(in crate::subscription) fn certification_activation_bundle_identity(
    admission_identity: &WorthQueryEvidenceIdentity,
    activation_identity: &WorthQueryEvidenceIdentity,
    query_declaration_identity: &WorthQueryEvidenceIdentity,
    bridge_declaration_identity: &WorthQueryEvidenceIdentity,
    basis_binding_identity: &WorthQueryEvidenceIdentity,
    signal_strategy_identity: &WorthQueryEvidenceIdentity,
    diagnostics_identity: &WorthQueryEvidenceIdentity,
    support_identity: &WorthQueryEvidenceIdentity,
    admission_counters_identity: &WorthQueryEvidenceIdentity,
    activation_counters_identity: &WorthQueryEvidenceIdentity,
    scale_slope_identity: &WorthQueryEvidenceIdentity,
) -> WorthQueryEvidenceIdentity {
    WorthQueryEvidenceIdentity::compose(SUBSCRIPTION_IDENTITY_SCOPE)
            .field_shape(
                WorthQueryEvidenceTag::new("identity_family"),
                "query_subscription_certification_bundle_v1",
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("admission"), admission_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("activation"),
                activation_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("query_declaration"),
                query_declaration_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("bridge_declaration"),
                bridge_declaration_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("basis"), basis_binding_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("signal_strategy"),
                signal_strategy_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("diagnostics"),
                diagnostics_identity,
            )
            .field_evidence_identity(WorthQueryEvidenceTag::new("support"), support_identity)
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("admission_counters"),
                admission_counters_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("activation_counters"),
                activation_counters_identity,
            )
            .field_evidence_identity(
                WorthQueryEvidenceTag::new("scale_slope"),
                scale_slope_identity,
            )
            .seal().expect("fixed digest grammar: literal tags and values, primitive values, or sealed identity tokens")
}
