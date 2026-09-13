use sha2::{Digest, Sha256};
use worth_store::physical_runtime::{
    PhysicalRecoveryStagingCommandDenialKind, PhysicalRecoveryStagingCommandIndeterminate,
    PhysicalRecoveryStagingCommandStage, PhysicalRecoveryStagingMaterialization,
    PhysicalRecoveryStagingMaterializationEvidence, RecoveryStagingWriteDisposition,
};

use crate::entry::PhysicalRecoveryStagingCounters;

pub(super) fn record_completed(
    counters: &mut PhysicalRecoveryStagingCounters,
    content: &mut Sha256,
    completed: &worth_store::physical_runtime::CompletedPhysicalRecoveryStagingCommand,
) {
    counters.commands_submitted = counters.commands_submitted.saturating_add(2);
    counters.commands_settled = counters.commands_settled.saturating_add(2);
    counters.scheduler_settlements = counters.scheduler_settlements.saturating_add(2);
    counters.artifacts_synchronized = counters.artifacts_synchronized.saturating_add(1);
    counters.performed_effects = counters.performed_effects.saturating_add(1);
    let physical = completed.materialization().physical();
    content.update(physical.artifact().file_name().as_bytes());
    content.update(physical.byte_count().to_le_bytes());
    content.update(physical.payload_digest());
    match physical.disposition() {
        RecoveryStagingWriteDisposition::Created => {
            counters.artifacts_created = counters.artifacts_created.saturating_add(1);
            counters.bytes_written = counters.bytes_written.saturating_add(physical.byte_count());
            counters.performed_effects = counters.performed_effects.saturating_add(1);
        }
        RecoveryStagingWriteDisposition::AlreadyMaterialized => {
            counters.artifacts_converged = counters.artifacts_converged.saturating_add(1);
            counters.bytes_verified = counters
                .bytes_verified
                .saturating_add(physical.byte_count());
        }
        RecoveryStagingWriteDisposition::CompletedFromExactPrefix => {
            counters.artifacts_completed_from_prefix =
                counters.artifacts_completed_from_prefix.saturating_add(1);
            counters.bytes_verified = counters.bytes_verified.saturating_add(
                physical
                    .prefix_verified()
                    .map_or(0, |prefix| prefix.completed_bytes()),
            );
            counters.bytes_written = counters.bytes_written.saturating_add(
                physical
                    .appended()
                    .map_or(0, |append| append.range().byte_count()),
            );
            counters.performed_effects = counters.performed_effects.saturating_add(1);
        }
    }
}

pub(super) fn record_denial(
    counters: &mut PhysicalRecoveryStagingCounters,
    denial: &worth_store::physical_runtime::PhysicalRecoveryStagingCommandDenial,
) {
    if let Some(materialization) = denial.materialization() {
        record_materialization(counters, materialization);
    }
    if !matches!(
        denial.denial(),
        PhysicalRecoveryStagingCommandDenialKind::Submission
    ) {
        counters.commands_submitted = counters.commands_submitted.saturating_add(1);
        counters.commands_settled = counters.commands_settled.saturating_add(1);
    }
    if denial.scheduler_posture().is_some() {
        counters.scheduler_settlements = counters.scheduler_settlements.saturating_add(1);
    }
}

pub(super) fn record_indeterminate(
    counters: &mut PhysicalRecoveryStagingCounters,
    denial: &PhysicalRecoveryStagingCommandIndeterminate,
) -> PhysicalRecoveryStagingCommandStage {
    match denial {
        PhysicalRecoveryStagingCommandIndeterminate::Materialization { scheduler, .. } => {
            record_settled_stage(counters, scheduler.is_some());
            PhysicalRecoveryStagingCommandStage::Materialization
        }
        PhysicalRecoveryStagingCommandIndeterminate::Synchronization {
            materialization,
            scheduler,
            ..
        } => {
            record_materialization(counters, materialization);
            record_settled_stage(counters, scheduler.is_some());
            PhysicalRecoveryStagingCommandStage::Synchronization
        }
        PhysicalRecoveryStagingCommandIndeterminate::Scheduler {
            stage,
            materialization,
            synchronization,
            ..
        }
        | PhysicalRecoveryStagingCommandIndeterminate::Signal {
            stage,
            materialization,
            synchronization,
            ..
        }
        | PhysicalRecoveryStagingCommandIndeterminate::Yieldpoint {
            stage,
            materialization,
            synchronization,
            ..
        } => {
            let materialization_was_recorded = materialization.is_some();
            if let Some(materialization) = materialization {
                record_materialization_evidence(counters, materialization);
            }
            if *stage == PhysicalRecoveryStagingCommandStage::Synchronization
                || !materialization_was_recorded
            {
                record_one_settled_stage(counters);
            }
            if synchronization.is_some() {
                counters.artifacts_synchronized = counters.artifacts_synchronized.saturating_add(1);
            }
            *stage
        }
    }
}

fn record_settled_stage(counters: &mut PhysicalRecoveryStagingCounters, has_scheduler: bool) {
    counters.commands_submitted = counters.commands_submitted.saturating_add(1);
    counters.commands_settled = counters.commands_settled.saturating_add(1);
    if has_scheduler {
        counters.scheduler_settlements = counters.scheduler_settlements.saturating_add(1);
    }
}

fn record_one_settled_stage(counters: &mut PhysicalRecoveryStagingCounters) {
    record_settled_stage(counters, true);
}

fn record_materialization(
    counters: &mut PhysicalRecoveryStagingCounters,
    materialization: &PhysicalRecoveryStagingMaterialization,
) {
    record_one_settled_stage(counters);
    let physical = materialization.physical();
    match physical.disposition() {
        RecoveryStagingWriteDisposition::Created => {
            counters.artifacts_created = counters.artifacts_created.saturating_add(1);
            counters.bytes_written = counters.bytes_written.saturating_add(physical.byte_count());
            counters.performed_effects = counters.performed_effects.saturating_add(1);
        }
        RecoveryStagingWriteDisposition::AlreadyMaterialized => {
            counters.artifacts_converged = counters.artifacts_converged.saturating_add(1);
            counters.bytes_verified = counters
                .bytes_verified
                .saturating_add(physical.byte_count());
        }
        RecoveryStagingWriteDisposition::CompletedFromExactPrefix => {
            counters.artifacts_completed_from_prefix =
                counters.artifacts_completed_from_prefix.saturating_add(1);
            counters.bytes_verified = counters.bytes_verified.saturating_add(
                physical
                    .prefix_verified()
                    .map_or(0, |prefix| prefix.completed_bytes()),
            );
            counters.bytes_written = counters.bytes_written.saturating_add(
                physical
                    .appended()
                    .map_or(0, |append| append.range().byte_count()),
            );
            counters.performed_effects = counters.performed_effects.saturating_add(1);
        }
    }
}

fn record_materialization_evidence(
    counters: &mut PhysicalRecoveryStagingCounters,
    materialization: &PhysicalRecoveryStagingMaterializationEvidence,
) {
    record_one_settled_stage(counters);
    let physical = materialization.physical();
    match physical.disposition() {
        RecoveryStagingWriteDisposition::Created => {
            counters.artifacts_created = counters.artifacts_created.saturating_add(1);
            counters.bytes_written = counters.bytes_written.saturating_add(physical.byte_count());
            if materialization.is_performed() {
                counters.performed_effects = counters.performed_effects.saturating_add(1);
            }
        }
        RecoveryStagingWriteDisposition::AlreadyMaterialized => {
            counters.artifacts_converged = counters.artifacts_converged.saturating_add(1);
            counters.bytes_verified = counters
                .bytes_verified
                .saturating_add(physical.byte_count());
        }
        RecoveryStagingWriteDisposition::CompletedFromExactPrefix => {
            counters.artifacts_completed_from_prefix =
                counters.artifacts_completed_from_prefix.saturating_add(1);
            counters.bytes_verified = counters.bytes_verified.saturating_add(
                physical
                    .prefix_verified()
                    .map_or(0, |prefix| prefix.completed_bytes()),
            );
            counters.bytes_written = counters.bytes_written.saturating_add(
                physical
                    .appended()
                    .map_or(0, |append| append.range().byte_count()),
            );
            if materialization.is_performed() {
                counters.performed_effects = counters.performed_effects.saturating_add(1);
            }
        }
    }
}
