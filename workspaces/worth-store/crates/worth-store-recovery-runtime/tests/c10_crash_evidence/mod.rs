use std::path::Path;

use worth_proof::TransitionOutcome;
use worth_store::physical_runtime::{
    PhysicalMutationDeadline, PhysicalMutationIdempotencyMaterial,
    PhysicalMutationPreparationSuccess, PhysicalMutationRequest, PreparedPhysicalMutation,
};
use worth_store_recovery_runtime::PhysicalRecoveryLimits;
use worth_store_test_support::harness::physical_residency::PhysicalResidencyStoreWorld;

use crate::phase_three_support::limit_declaration;

pub fn prepare_rewrite(
    world: &PhysicalResidencyStoreWorld,
    material: [u8; 32],
) -> PreparedPhysicalMutation {
    let submission = world.serving().record_submission();
    let key = submission
        .issue_idempotency_key(PhysicalMutationIdempotencyMaterial::new(material))
        .unwrap();
    match submission
        .rewrite_selected_inline_segment(
            world.placement(),
            PhysicalMutationRequest::platform_durable(
                key,
                PhysicalMutationDeadline::after_milliseconds(1_000).unwrap(),
            ),
        )
        .into_raw()
    {
        TransitionOutcome::Success(PhysicalMutationPreparationSuccess::Prepared(prepared)) => {
            prepared
        }
        _ => panic!("rewrite preparation must succeed"),
    }
}

pub fn ordinary_limits() -> PhysicalRecoveryLimits {
    let mut declaration = limit_declaration(2, 8, 2 * 1024 * 1024);
    declaration.manifest_entries = 4_096;
    declaration.wal_bytes = 2 * 1024 * 1024;
    declaration.redo_targets = 4_096;
    declaration.redo_bytes = 4 * 1024 * 1024;
    declaration.distinct_pages_and_extents = 4_096;
    declaration.operation_bindings = 4_096;
    declaration.staging_bytes = 32 * 1024 * 1024;
    declaration.recovery_memory_bytes = 32 * 1024 * 1024;
    declaration.dirty_frames = 4_096;
    declaration.publication_effects = 64;
    declaration.observation_bytes = 32 * 1024 * 1024;
    PhysicalRecoveryLimits::admit(declaration).unwrap()
}

pub fn segment_snapshot(root: &Path) -> Vec<(String, u64)> {
    directory_snapshot(root, "families/records/segments")
}

pub fn directory_snapshot(root: &Path, relative: &str) -> Vec<(String, u64)> {
    let mut files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(root.join(relative)) {
        for entry in entries.flatten() {
            let bytes = std::fs::read(entry.path()).unwrap_or_default();
            let mut checksum = 0_u64;
            for byte in bytes {
                checksum = checksum
                    .wrapping_mul(16_777_619)
                    .wrapping_add(u64::from(byte));
            }
            files.push((entry.file_name().to_string_lossy().into_owned(), checksum));
        }
    }
    files.sort();
    files
}

pub fn wal_contains_rewrite(root: &Path) -> bool {
    let domain = worth_store_physical_format::REWRITE_REDO_DOMAIN;
    let mut stack = vec![root.join("families").join("wal")];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = std::fs::read_dir(directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                if bytes.windows(domain.len()).any(|window| window == domain) {
                    return true;
                }
            }
        }
    }
    false
}
