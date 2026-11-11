use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::evidence_identity::{
    WorthQueryEvidenceIdentity, WorthQueryEvidenceScope, WorthQueryEvidenceTag,
};

use super::delivery_error::{QueryDeliveryDenialKind, QueryDeliveryError};
use super::maintenance_delta::QuerySubscriptionMaintenanceDelta;
use super::ActiveSubscriptionCounters;

fn raw_delivery_denial_source_identity(
    source_label: &str,
) -> Result<WorthQueryEvidenceIdentity, IdentityDerivationDenial> {
    WorthQueryEvidenceIdentity::compose(WorthQueryEvidenceScope::SubscriptionActivationReceipt)
        .field_shape(
            WorthQueryEvidenceTag::new("identity_family"),
            "query_raw_delivery_denial_source_v1",
        )
        .field_value(WorthQueryEvidenceTag::new("source"), source_label)
        .seal()
}

pub fn deny_raw_cdc_delivery_fallback(
    source_label: impl AsRef<str>,
) -> Result<QuerySubscriptionMaintenanceDelta, QueryDeliveryError> {
    let mut counters = ActiveSubscriptionCounters::default();
    counters.raw_cdc_delivery_denial_count = 1;
    Err(QueryDeliveryError::with_source_derivation(
        QueryDeliveryDenialKind::RawCdcFallbackDenied,
        "raw CDC cannot be consumed as active query delivery",
        raw_delivery_denial_source_identity(source_label.as_ref()),
        counters,
    ))
}

pub fn deny_raw_bridge_invalidation_delivery(
    source_label: impl AsRef<str>,
) -> Result<QuerySubscriptionMaintenanceDelta, QueryDeliveryError> {
    let mut counters = ActiveSubscriptionCounters::default();
    counters.raw_bridge_invalidation_denial_count = 1;
    Err(QueryDeliveryError::with_source_derivation(
        QueryDeliveryDenialKind::RawBridgeInvalidationDenied,
        "raw bridge invalidation must lower into a query maintenance delta first",
        raw_delivery_denial_source_identity(source_label.as_ref()),
        counters,
    ))
}
