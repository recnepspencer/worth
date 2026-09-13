use worth_store_recovery_runtime::RecoveryOperationFateSet;

pub(super) fn render(fates: &RecoveryOperationFateSet) {
    for operation in fates.operations() {
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
