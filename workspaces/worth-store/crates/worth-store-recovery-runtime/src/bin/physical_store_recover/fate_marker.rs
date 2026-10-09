use worth_store_recovery_runtime::RecoveryOperationFateSet;

pub(super) fn render(fates: &RecoveryOperationFateSet) {
    for operation in fates.operations() {
        if let Some(descriptor) =
            fates.historical_consuming_descriptor(operation.identity().idempotency())
        {
            eprintln!(
                "C8_RECOVERY_FATE idempotency={} fate=HistoricalReleaseConsumed descriptor={}",
                hex(&operation.identity().idempotency()),
                hex(&descriptor),
            );
            continue;
        }
        eprintln!(
            "C8_RECOVERY_FATE idempotency={} fate={:?}",
            hex(&operation.identity().idempotency()),
            operation.fate()
        );
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
