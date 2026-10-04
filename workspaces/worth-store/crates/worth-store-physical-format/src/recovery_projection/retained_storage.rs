use super::*;

fn boxed_bytes<T>(length: usize) -> Option<u64> {
    u64::try_from(length)
        .ok()?
        .checked_mul(u64::try_from(std::mem::size_of::<T>()).ok()?)
}

impl PersistedPhysicalRecoveryProjection {
    /// Heap owned by the decoded projection, excluding the inline value itself.
    pub fn owned_heap_bytes(&self) -> Option<u64> {
        let mut bytes = self.root_state.owned_heap_bytes()?;
        bytes = bytes.checked_add(boxed_bytes::<PersistedRecordIdentity>(
            self.record_identities.len(),
        )?)?;
        bytes = bytes.checked_add(boxed_bytes::<CurrentPhysicalRecordPlacement>(
            self.placements.len(),
        )?)?;
        bytes = bytes.checked_add(boxed_bytes::<RecordSegmentPageManifestEntry>(
            self.segment_updates.len(),
        )?)?;
        bytes = bytes.checked_add(boxed_bytes::<PersistedPhysicalRecoveryManifest>(
            self.manifests.len(),
        )?)?;
        bytes = self.manifests.iter().try_fold(bytes, |sum, manifest| {
            sum.checked_add(u64::try_from(manifest.bytes.len()).ok()?)
        })?;
        if let PersistedPhysicalRecoveryOperation::DerivedDirectory {
            retirement: Some(retirement),
            ..
        } = &self.operation
        {
            bytes = bytes.checked_add(boxed_bytes::<PersistedRecordIdentity>(
                retirement.dropped_records.len(),
            )?)?;
        }
        if let Some(claim) = self.operation.release_head_tree_claim() {
            bytes = bytes.checked_add(claim.owned_heap_bytes()?)?;
        }
        match &self.payload {
            PersistedPhysicalRecoveryPayload::Frames(frames) => {
                bytes = bytes
                    .checked_add(boxed_bytes::<PersistedPhysicalRecoveryFrame>(frames.len())?)?;
                frames.iter().try_fold(bytes, |sum, frame| {
                    sum.checked_add(u64::try_from(frame.bytes.len()).ok()?)
                })
            }
            PersistedPhysicalRecoveryPayload::SourceCopy(_) => Some(bytes),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn retained_manifest_frame_counts_its_boxed_payload() {
        let root_state = PersistedPhysicalRecoveryRootState::new(1, 1, 2, vec![], None, None)
            .expect("root state");
        let coordinate =
            RecordFrameCoordinate::new(RecordArtifactFile::ExtentArena { arena: 1 }, 0, 4)
                .expect("coordinate");
        let projection = PersistedPhysicalRecoveryProjection {
            source_root_generation: 1,
            root_state,
            record_identities: Box::new([]),
            payload: PersistedPhysicalRecoveryPayload::Frames(Box::new([])),
            operation: PersistedPhysicalRecoveryOperation::None,
            placements: Box::new([]),
            segment_updates: Box::new([]),
            manifests: vec![PersistedPhysicalRecoveryManifest {
                coordinate,
                bytes: vec![1, 2, 3, 4].into_boxed_slice(),
            }]
            .into_boxed_slice(),
        };
        assert_eq!(
            projection.owned_heap_bytes(),
            Some(std::mem::size_of::<PersistedPhysicalRecoveryManifest>() as u64 + 4)
        );
    }
}
