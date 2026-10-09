use super::super::{selected_segment_rewrite::damaged, RecordPublicationDirector};
use super::obligation::{CopyBinding, CopyDestination, CopyObligation, CopyResolution};
use crate::physical_runtime::integrity::resident_admission::root_manifest::project_loaded_root_manifest;
use crate::physical_runtime::record_serving::{
    access::manifest_routing::{ManifestDiscoveryCounterSnapshot, ManifestReader},
    RecordFramePorts,
};
use crate::physical_runtime::record_serving::{
    residency::record_frame_reader::RecordFrameReader, RecordAppendError,
};
use std::sync::{Arc, Mutex};
use worth_store_physical_backend::QualifiedFilesystemMedia;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurablePhysicalRootManifest, RecordArtifactFile,
};

impl RecordPublicationDirector {
    /// Before serving escapes, bind retained WAL facts to admitted historical
    /// routes, retain source authority, and fence exact destination reuse.
    pub(in crate::physical_runtime::record_serving::publication::director) fn seed_extent_copy(
        &self,
        displaced: &[crate::physical_runtime::durability::DisplacedArtifact],
        media: &QualifiedFilesystemMedia,
        ports: &RecordFramePorts,
        lifecycle: Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    ) -> Result<(), RecordAppendError> {
        let obligations = self.wal.recovered_copy_obligations();
        if obligations.is_empty() {
            return Ok(());
        }
        if obligations.len() != 1 {
            return Err(damaged());
        }
        let recovered = obligations[0];
        let intent = recovered.intent();
        let source = self.load_copy_root(intent.source_root(), media, ports)?;
        if self.bootstrap_extent_source(
            &source,
            intent.source().record(),
            media,
            ports,
            Arc::clone(&lifecycle),
        )? != intent.source()
        {
            return Err(damaged());
        }
        let publication = recovered.publication();
        // Only an unresolved staging arena is bounded by this old range.
        // Published or durably cancelled arenas may later grow through reuse.
        if publication.is_none() && recovered.resolution().is_none() {
            let destination_file = RecordArtifactFile::ExtentArena {
                arena: intent.destination().arena_range().arena().get(),
            };
            let artifacts = crate::physical_runtime::record_serving::residency::artifact_tree::PhysicalRecordArtifactTree::new(media);
            if artifacts
                .file_exists(destination_file)
                .map_err(|_| damaged())?
                && !bounded_staging_file(
                    artifacts
                        .file_length(destination_file)
                        .map_err(|_| damaged())?,
                    intent.destination().arena_range(),
                )
            {
                return Err(damaged());
            }
        }
        let source_lease = self
            .root_owner
            .protect_recovered_copy_source(&source)
            .map_err(|_| damaged())?;
        let (destination, physical_growth) = if let Some((generation, _)) = publication {
            let published = self.load_copy_root(generation, media, ports)?;
            if self.bootstrap_extent_source(
                &published,
                intent.source().record(),
                media,
                ports,
                lifecycle,
            )? != intent.destination()
                || generation > self.root_owner.snapshot().0.generation()
            {
                return Err(damaged());
            }
            // Bootstrap reconstructs garbage from admitted root transitions,
            // independent of redo domains. Never silently omit the source's
            // charge when replacing the staged destination lease on reopen.
            let source_artifact = crate::physical_runtime::durability::RetiredArtifact::Extent {
                extent: intent.source().extent().get(),
                generation: intent.source().extent_generation(),
                range: intent.source().arena_range(),
            };
            if displaced
                .iter()
                .filter(|charge| {
                    charge.artifact == source_artifact
                        && charge.source_root.checked_add(1) == Some(generation)
                        && charge.bytes == intent.source().arena_range().length()
                })
                .count()
                != 1
            {
                return Err(damaged());
            }
            (CopyDestination::Published, None)
        } else {
            let growth = self
                .root_owner
                .publication_admission()
                .reserve_retained_bytes(intent.destination().arena_range().length())
                .map_err(|_| damaged())?;
            (
                CopyDestination::Deferred(intent.destination().arena_range()),
                Some(growth),
            )
        };
        let destination = Arc::new(Mutex::new(destination));
        self.preparation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .recovered_copy_destination = Some(Arc::clone(&destination));
        let resolution =
            recovered
                .resolution()
                .map(|(resolution, end_lsn)| CopyResolution::Recovered {
                    kind: resolution.kind(),
                    end_lsn,
                });
        *self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner()) = Some(Arc::new(Mutex::new(CopyObligation {
            operation: intent.operation(),
            source_lease,
            reservation: destination,
            binding: CopyBinding::RecoveredUnpublished,
            intent: Some((intent, recovered.intent_lsn(), recovered.intent_digest())),
            carrier_alive: false,
            publication_lsn: publication.map(|(_, lsn)| lsn),
            resolved: false,
            published_root: publication.map(|(generation, _)| generation),
            resolution,
            inspection: false,
            resolution_requested: recovered
                .resolution()
                .map(|(resolution, _)| resolution.kind()),
            physical_growth,
        })));
        Ok(())
    }

    fn load_copy_root(
        &self,
        generation: u64,
        media: &QualifiedFilesystemMedia,
        ports: &RecordFramePorts,
    ) -> Result<DurablePhysicalRootManifest, RecordAppendError> {
        let allocation = self.copy_allocation()?;
        let reader = RecordFrameReader::bootstrap(media, ports.loader());
        let bytes = reader
            .load_bounded(
                &allocation,
                RecordArtifactFile::RootManifest { generation },
                384,
            )
            .map_err(|_| damaged())?;
        project_loaded_root_manifest(
            bytes.lease(),
            self.residency.store_identity(),
            self.format.declaration(),
            generation,
            self.residency.resident_admission_context(),
        )
        .map_err(|_| damaged())
    }

    fn bootstrap_extent_source(
        &self,
        root: &DurablePhysicalRootManifest,
        record: worth_store_physical_format::PersistedRecordIdentity,
        media: &QualifiedFilesystemMedia,
        ports: &RecordFramePorts,
        lifecycle: Arc<crate::physical_runtime::lifecycle::LifecycleState>,
    ) -> Result<worth_store_physical_format::DurableExtentRecordPlacement, RecordAppendError> {
        let allocation = self.copy_allocation()?;
        let counters = ports.resident_integrity_counter_owner();
        let reader = ManifestReader::with_loader(
            media,
            ports.loader(),
            self.format,
            self.access,
            root,
            lifecycle,
            &counters,
        );
        match reader
            .locate(
                &allocation,
                record,
                &mut ManifestDiscoveryCounterSnapshot::default(),
            )
            .map_err(|_| damaged())?
        {
            Some(CurrentPhysicalRecordPlacement::Extent(source)) => Ok(source),
            _ => Err(damaged()),
        }
    }

    /// Certification observes the exact pending claim posture. Before the
    /// allocator is constructed, a deferred claim is valid only while no
    /// allocator has escaped; after construction the reservation token must
    /// still name this exact range in the allocator's private index.
    #[cfg(feature = "certification-test-authority")]
    pub(in crate::physical_runtime) fn certification_holds_recovered_copy_destination(
        &self,
        range: worth_store_physical_format::ExtentArenaRange,
    ) -> bool {
        let obligation = self
            .copy_obligation
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone();
        let Some(obligation) = obligation else {
            return false;
        };
        let state = obligation.lock().unwrap_or_else(|e| e.into_inner());
        if state.resolved
            || state.published_root.is_some()
            || !matches!(state.binding, CopyBinding::RecoveredUnpublished)
        {
            return false;
        }
        let reservation = Arc::clone(&state.reservation);
        drop(state);
        let destination = reservation.lock().unwrap_or_else(|e| e.into_inner());
        if let CopyDestination::Reserved(claim) = &*destination {
            return claim.certification_has_exact_claim(range);
        }
        let deferred_exact =
            matches!(&*destination, CopyDestination::Deferred(held) if *held == range);
        drop(destination);
        // `arenas` only moves from None to Some; this second observation cannot
        // mistake a concurrently exposed allocator for a deferred private hold.
        deferred_exact
            && self
                .preparation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .arenas
                .is_none()
    }
}

fn bounded_staging_file(
    file_length: u64,
    destination: worth_store_physical_format::ExtentArenaRange,
) -> bool {
    file_length <= destination.end()
}

#[cfg(test)]
mod tests {
    use super::bounded_staging_file;
    use worth_store_physical_format::{ExtentArenaId, ExtentArenaRange};

    #[test]
    fn same_arena_bytes_beyond_held_destination_fail_closed() {
        let range = ExtentArenaRange::new(ExtentArenaId::new(7).unwrap(), 0, 4096).unwrap();
        assert!(bounded_staging_file(0, range));
        assert!(bounded_staging_file(4096, range));
        assert!(!bounded_staging_file(4097, range));
    }
}
