use std::num::NonZeroU64;

use sha2::{Digest, Sha256};

const HARD_MAXIMUM: u64 = u32::MAX as u64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryLimitDeclaration {
    pub selector_candidates: u64,
    pub checkpoint_candidates: u64,
    pub manifest_bytes: u64,
    pub manifest_entries: u64,
    pub wal_segments: u64,
    pub wal_frames: u64,
    pub wal_bytes: u64,
    pub redo_targets: u64,
    pub redo_bytes: u64,
    pub distinct_pages_and_extents: u64,
    pub operation_bindings: u64,
    pub staging_bytes: u64,
    pub recovery_memory_bytes: u64,
    pub dirty_frames: u64,
    pub concurrent_commands: u64,
    pub publication_effects: u64,
    pub cleanup_candidates: u64,
    pub cleanup_bytes: u64,
    pub observation_bytes: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhysicalRecoveryLimits {
    declaration: PhysicalRecoveryLimitDeclaration,
    /// Each admitted dimension, as the nonzero count admission proved it.
    admitted: [NonZeroU64; 19],
    identity: [u8; 32],
}

impl PhysicalRecoveryLimits {
    pub fn admit(
        declaration: PhysicalRecoveryLimitDeclaration,
    ) -> Result<Self, PhysicalRecoveryLimitDenial> {
        let values = declaration.values();
        let mut admitted = [NonZeroU64::MIN; 19];
        for ((dimension, value), slot) in
            LIMIT_DIMENSIONS.into_iter().zip(values).zip(&mut admitted)
        {
            let Some(nonzero) = NonZeroU64::new(value) else {
                return Err(PhysicalRecoveryLimitDenial::Zero { dimension });
            };
            *slot = nonzero;
            if value > HARD_MAXIMUM {
                return Err(PhysicalRecoveryLimitDenial::AboveHardMaximum {
                    dimension,
                    declared: value,
                    hard_maximum: HARD_MAXIMUM,
                });
            }
        }
        Ok(Self {
            declaration,
            admitted,
            identity: limit_identity(values),
        })
    }

    /// The WAL segment ceiling, which admission proved is at least one.
    pub(crate) const fn wal_segments(self) -> NonZeroU64 {
        self.admitted[WAL_SEGMENTS]
    }

    pub(crate) const fn identity(self) -> [u8; 32] {
        self.identity
    }

    pub(crate) const fn declaration(self) -> PhysicalRecoveryLimitDeclaration {
        self.declaration
    }
}

impl PhysicalRecoveryLimitDeclaration {
    const fn values(self) -> [u64; 19] {
        [
            self.selector_candidates,
            self.checkpoint_candidates,
            self.manifest_bytes,
            self.manifest_entries,
            self.wal_segments,
            self.wal_frames,
            self.wal_bytes,
            self.redo_targets,
            self.redo_bytes,
            self.distinct_pages_and_extents,
            self.operation_bindings,
            self.staging_bytes,
            self.recovery_memory_bytes,
            self.dirty_frames,
            self.concurrent_commands,
            self.publication_effects,
            self.cleanup_candidates,
            self.cleanup_bytes,
            self.observation_bytes,
        ]
    }
}

/// Where `values` places the WAL segment ceiling.
const WAL_SEGMENTS: usize = 4;

fn limit_identity(values: [u64; 19]) -> [u8; 32] {
    let mut digest = Sha256::new();
    for value in values {
        digest.update(value.to_le_bytes());
    }
    digest.finalize().into()
}

const LIMIT_DIMENSIONS: [&str; 19] = [
    "selector-candidates",
    "checkpoint-candidates",
    "manifest-bytes",
    "manifest-entries",
    "wal-segments",
    "wal-frames",
    "wal-bytes",
    "redo-targets",
    "redo-bytes",
    "distinct-pages-and-extents",
    "operation-bindings",
    "staging-bytes",
    "recovery-memory-bytes",
    "dirty-frames",
    "concurrent-commands",
    "publication-effects",
    "cleanup-candidates",
    "cleanup-bytes",
    "observation-bytes",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhysicalRecoveryLimitDenial {
    Zero {
        dimension: &'static str,
    },
    AboveHardMaximum {
        dimension: &'static str,
        declared: u64,
        hard_maximum: u64,
    },
}

#[cfg(test)]
mod tests {
    use super::{
        PhysicalRecoveryLimitDeclaration, PhysicalRecoveryLimitDenial, PhysicalRecoveryLimits,
        HARD_MAXIMUM,
    };

    #[test]
    fn each_dimension_must_be_finite_and_nonzero() {
        let baseline = [1; 19];
        for index in 0..baseline.len() {
            let mut values = baseline;
            values[index] = 0;
            assert!(matches!(
                PhysicalRecoveryLimits::admit(declaration(values)),
                Err(PhysicalRecoveryLimitDenial::Zero { .. })
            ));
            values[index] = 7;
            let admitted = PhysicalRecoveryLimits::admit(declaration(values)).unwrap();
            assert_eq!(admitted.admitted[index].get(), 7);
            assert_eq!(admitted.declaration().values()[index], 7);
            values[index] = HARD_MAXIMUM + 1;
            assert!(matches!(
                PhysicalRecoveryLimits::admit(declaration(values)),
                Err(PhysicalRecoveryLimitDenial::AboveHardMaximum { .. })
            ));
        }
    }

    #[test]
    fn the_wal_segment_ceiling_is_the_declared_one() {
        let mut values = [1; 19];
        values[4] = 9;
        let limits = PhysicalRecoveryLimits::admit(declaration(values)).unwrap();
        assert_eq!(limits.wal_segments().get(), 9);
        assert_eq!(limits.declaration().wal_segments, 9);
    }

    fn declaration(values: [u64; 19]) -> PhysicalRecoveryLimitDeclaration {
        PhysicalRecoveryLimitDeclaration {
            selector_candidates: values[0],
            checkpoint_candidates: values[1],
            manifest_bytes: values[2],
            manifest_entries: values[3],
            wal_segments: values[4],
            wal_frames: values[5],
            wal_bytes: values[6],
            redo_targets: values[7],
            redo_bytes: values[8],
            distinct_pages_and_extents: values[9],
            operation_bindings: values[10],
            staging_bytes: values[11],
            recovery_memory_bytes: values[12],
            dirty_frames: values[13],
            concurrent_commands: values[14],
            publication_effects: values[15],
            cleanup_candidates: values[16],
            cleanup_bytes: values[17],
            observation_bytes: values[18],
        }
    }
}
