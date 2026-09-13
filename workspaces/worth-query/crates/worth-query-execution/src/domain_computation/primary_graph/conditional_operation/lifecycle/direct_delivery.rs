pub(super) type WorthQueryPendingDirectDelivery =
    Option<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>;

pub(super) const fn empty() -> WorthQueryPendingDirectDelivery {
    None
}

pub(super) fn retain(
    pending: &mut WorthQueryPendingDirectDelivery,
    receipt: &worth_runtime_bridge::facade::BridgeCorrespondenceDeliveryReceipt,
) {
    assert!(
        pending.is_none(),
        "one exact product occurrence can retain only its unique performed change"
    );
    *pending =
        Some(worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery::direct(receipt));
}

pub(super) fn as_slice(
    pending: &WorthQueryPendingDirectDelivery,
) -> &[worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery] {
    pending.as_slice()
}

pub(super) fn move_into(
    pending: &mut WorthQueryPendingDirectDelivery,
    deliveries: &mut Vec<worth_runtime_bridge::facade::BridgeGranularInvalidationDelivery>,
) {
    if let Some(delivery) = pending.take() {
        deliveries.push(delivery);
    }
}
