use crate::filesystem_media::ArtifactTreeFailureKind;
use worth_store_physical_format::RecordArtifactFile;

use super::{
    record_artifact, BoundedRecoveryFilesystemDiscovery, ObservedRecoveryArtifact,
    RecoveryDiscoveryArtifact, RecoveryDiscoveryByteLimitScope, RecoveryDiscoveryFailure,
};

impl BoundedRecoveryFilesystemDiscovery {
    pub(super) fn read_addressed_range(
        &mut self,
        address: RecordArtifactFile,
        offset: u64,
        length: u32,
        byte_limit: u64,
    ) -> Result<ObservedRecoveryArtifact, RecoveryDiscoveryFailure> {
        let context = RecoveryDiscoveryArtifact::Record(address);
        let artifact = record_artifact(address)?;
        let length = u64::from(length);
        let admitted = self.remaining_bytes.min(byte_limit);
        if length == 0 || length > admitted {
            return Err(RecoveryDiscoveryFailure::ByteLimitExceeded {
                observed: self.counters.bytes_read.saturating_add(length),
                admitted: self.maximum_bytes.min(byte_limit),
                scope: if self.remaining_bytes <= byte_limit {
                    RecoveryDiscoveryByteLimitScope::Observation
                } else {
                    RecoveryDiscoveryByteLimitScope::Requested
                },
            });
        }
        if self.remaining_entries == 0 {
            return Err(RecoveryDiscoveryFailure::EntryLimitExceeded {
                observed: 1,
                admitted: 0,
            });
        }
        let capacity = usize::try_from(length)
            .map_err(|_| RecoveryDiscoveryFailure::invalid(context.clone()))?;
        self.remaining_entries -= 1;
        let mut bytes = vec![0; capacity];
        match self
            .parts
            .artifact_tree()
            .read_exact_at(&artifact, offset, &mut bytes)
        {
            Ok(()) => {
                self.remaining_bytes -= length;
                self.counters.bytes_read += length;
                self.counters.addressed_artifacts_read += 1;
                Ok(ObservedRecoveryArtifact::new(
                    self.parts.store_identity,
                    context,
                    offset,
                    Some(bytes),
                ))
            }
            Err(failure) if failure.kind() == ArtifactTreeFailureKind::Absent => Ok(
                ObservedRecoveryArtifact::new(self.parts.store_identity, context, offset, None),
            ),
            Err(failure) => Err(RecoveryDiscoveryFailure::Media {
                artifact: context,
                failure,
            }),
        }
    }
}
