#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WriterDurabilityProfile {
    CheckpointWritebackV1,
    CleanupRotationV1,
}

impl WriterDurabilityProfile {
    pub(super) fn parse(value: &str) -> Result<Self, String> {
        match value {
            "c8-phase8-checkpoint-writeback-v1" => Ok(Self::CheckpointWritebackV1),
            "c8-phase8-cleanup-rotation-v1" => Ok(Self::CleanupRotationV1),
            _ => Err(format!("unknown C8 writer durability profile `{value}`")),
        }
    }

    pub(super) const fn wal_segment_byte_limit(self) -> u64 {
        match self {
            Self::CheckpointWritebackV1 => 128 * 1024,
            Self::CleanupRotationV1 => 24 * 1024,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WriterDurabilityProfile;

    #[test]
    fn profiles_are_explicit_and_stable() {
        for (name, profile) in [
            (
                "c8-phase8-checkpoint-writeback-v1",
                WriterDurabilityProfile::CheckpointWritebackV1,
            ),
            (
                "c8-phase8-cleanup-rotation-v1",
                WriterDurabilityProfile::CleanupRotationV1,
            ),
        ] {
            assert_eq!(WriterDurabilityProfile::parse(name), Ok(profile));
        }
        assert!(WriterDurabilityProfile::parse("128 KiB").is_err());
    }

    #[test]
    fn cleanup_rotation_has_distinct_smaller_segments() {
        assert!(
            WriterDurabilityProfile::CleanupRotationV1.wal_segment_byte_limit()
                < WriterDurabilityProfile::CheckpointWritebackV1.wal_segment_byte_limit()
        );
    }
}
