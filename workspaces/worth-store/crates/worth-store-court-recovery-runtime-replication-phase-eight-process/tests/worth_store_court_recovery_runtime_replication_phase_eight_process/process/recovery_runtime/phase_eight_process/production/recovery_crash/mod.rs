#[allow(unused_imports)]
use crate::phase_eight_process::production::recovery_crash::*;
use worth_store::physical_runtime::PhysicalRecoveryYieldpointStage;

use crate::phase_eight_process::production::harness::ProcessWorld;

#[test]
fn killed_recovery_process_reopens_after_each_named_recovery_seam() {
    let publication_world =
        crate::phase_eight_process::production::harness::ProcessWorld::start_with_operation_count(
            "candidate-binding-record",
            0xC8_08_11,
            0xC8_18_11,
            RECOVERY_SEAM_OPERATION_COUNT,
        );
    for (index, stage) in RECOVERY_SEAMS
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, stage)| !is_cleanup_stage(*stage))
    {
        crate::phase_eight_process::production::recovery_crash::seam::run(
            &publication_world,
            index,
            stage,
        );
    }

    let cleanup_world = crate::phase_eight_process::production::harness::ProcessWorld::start_cleanup_world_with_operation_count(
        0xC8_08_21,
        0xC8_18_21,
        RECOVERY_SEAM_OPERATION_COUNT,
    );
    crate::phase_eight_process::production::recovery_crash::cleanup_world::require_raw_candidate(
        &cleanup_world,
    );
    for (index, stage) in RECOVERY_SEAMS
        .iter()
        .copied()
        .enumerate()
        .filter(|(_, stage)| is_cleanup_stage(*stage))
    {
        crate::phase_eight_process::production::recovery_crash::seam::run(
            &cleanup_world,
            index,
            stage,
        );
    }
}

#[test]
fn cancelled_recovery_process_reports_a_blocked_post_effect_outcome() {
    let publication_world =
        crate::phase_eight_process::production::harness::ProcessWorld::start_with_operation_count(
            "candidate-binding-record",
            0xC8_08_31,
            0xC8_18_31,
            RECOVERY_SEAM_OPERATION_COUNT,
        );
    crate::phase_eight_process::production::recovery_crash::interruption::run_publication(
        &publication_world,
        0,
    );

    let cleanup_world = crate::phase_eight_process::production::harness::ProcessWorld::start_cleanup_world_with_operation_count(
        0x00C8_0832,
        0x00C8_1832,
        RECOVERY_SEAM_OPERATION_COUNT,
    );
    crate::phase_eight_process::production::recovery_crash::cleanup_world::require_raw_candidate(
        &cleanup_world,
    );
    crate::phase_eight_process::production::recovery_crash::interruption::run_cleanup(
        &cleanup_world,
        RECOVERY_SEAMS.len(),
    );
}
