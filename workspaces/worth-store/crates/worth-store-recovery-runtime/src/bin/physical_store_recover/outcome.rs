use worth_store_recovery_runtime::PhysicalRecoveryOutcome;

pub(super) fn render(outcome: PhysicalRecoveryOutcome) -> Result<(), String> {
    match outcome {
        PhysicalRecoveryOutcome::Recovered(handoff) => {
            eprintln!(
                "C8_RECOVERY_RUNTIME {} {} {}",
                hex(&handoff.core().store_identity().bytes()),
                handoff.core().runtime_identity().get(),
                handoff.core().root().generation()
            );
            let fates = handoff.operation_fates();
            eprintln!(
                "C8_RECOVERY_FATES acknowledged={} durable_unacknowledged={} proven_no_effect={} indeterminate={}",
                fates.acknowledged_durable(),
                fates.durable_unacknowledged(),
                fates.proven_no_effect(),
                fates.indeterminate(),
            );
            super::fate_marker::render(fates);
            eprintln!(
                "recovered Store {:?} into runtime {:?} at root generation {}",
                handoff.core().store_identity().bytes(),
                handoff.core().runtime_identity(),
                handoff.core().root().generation()
            );
            Ok(())
        }
        PhysicalRecoveryOutcome::Refused(refusal) => {
            eprintln!(
                "C8_RECOVERY_REFUSED kind={:?} effects={}",
                refusal.kind,
                refusal.recovery_effects()
            );
            Err(format!("physical recovery was refused: {refusal:?}"))
        }
        PhysicalRecoveryOutcome::Blocked(block) => {
            eprintln!(
                "C8_RECOVERY_BLOCKED kind={:?} store={} source_generation={:?} effects={}",
                block.kind,
                hex(&block.store_identity().bytes()),
                block.evidence().source_generation,
                block.recovery_effects()
            );
            Err(format!("physical recovery was blocked: {block:?}"))
        }
        PhysicalRecoveryOutcome::PublicationIndeterminate(indeterminate) => {
            eprintln!(
                "C8_RECOVERY_PUBLICATION_INDETERMINATE store={} effects={}",
                hex(&indeterminate.store_identity().bytes()),
                indeterminate.recovery_effects()
            );
            Err(format!(
                "physical recovery publication was indeterminate: {indeterminate:?}"
            ))
        }
    }
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
