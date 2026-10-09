//! Bounded replay of a WAL-authenticated source-copy recipe.
use crate::integrity_ingress::RecoveryIntegrityIngressTrace;
use sha2::{Digest, Sha256};
use worth_store::physical_runtime::ObservedRecoveryArtifact;
use worth_store_physical_format::store_namespace::StableStoreIdentity;
use worth_store_physical_format::*;
use worth_store_physical_integrity::{
    IntegrityValidatedExtentMembership, PhysicalArtifactScope, PhysicalByteRange,
};

pub(crate) struct SourceCopyCursor {
    store: StableStoreIdentity,
    format: PhysicalRecordFormatDeclaration,
    recipe: PersistedExtentCopyRecipe,
    manifest: DurableExtentManifest,
    membership: IntegrityValidatedExtentMembership,
    ordinal: u32,
    logical_offset: u64,
    digest: Sha256,
    finished: bool,
}

impl SourceCopyCursor {
    pub(crate) fn open(
        store: StableStoreIdentity,
        format: PhysicalRecordFormatDeclaration,
        recipe: PersistedExtentCopyRecipe,
        observed: &ObservedRecoveryArtifact,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<Self, ()> {
        let source = recipe.intent().source();
        let manifest = recipe.intent().source_manifest(format).ok_or(())?;
        let expected = manifest.encode(format);
        if observed.bytes() != Some(expected.as_slice()) {
            return Err(());
        }
        let scope = PhysicalArtifactScope::extent_manifest(
            store,
            format,
            source,
            PhysicalByteRange::new(source.arena_range().offset(), expected.len() as u64)
                .map_err(|_| ())?,
        );
        let admitted =
            crate::integrity_ingress::admit_extent_manifest_projection(observed, scope, trace)
                .map_err(|_| ())?;
        Ok(Self {
            store,
            format,
            recipe,
            manifest,
            membership: admitted.membership,
            ordinal: 1,
            logical_offset: 0,
            digest: Sha256::new(),
            finished: false,
        })
    }

    pub(crate) fn source_coordinate(&self) -> Result<Option<RecordFrameCoordinate>, ()> {
        if self.ordinal > self.manifest.chunk_count() {
            return Ok(None);
        }
        let frame = self.source_frame()?;
        let range = self.recipe.intent().source().arena_range();
        RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena {
                arena: range.arena().get(),
            },
            range.offset() + frame.offset(),
            frame.length(),
        )
        .map(Some)
        .ok_or(())
    }

    /// The source chunk this cursor reads next.
    fn source_frame(&self) -> Result<ExtentChunkFrame, ()> {
        let layout =
            ExtentArenaFrameLayout::new(self.format, self.manifest.alignment()).ok_or(())?;
        ExtentChunkFrame::of(self.manifest, layout, self.ordinal).ok_or(())
    }

    pub(crate) fn transform(
        &mut self,
        observed: &ObservedRecoveryArtifact,
        trace: &mut RecoveryIntegrityIngressTrace,
    ) -> Result<(RecordFrameCoordinate, Vec<u8>), ()> {
        if self.finished {
            return Err(());
        }
        let source_coordinate = self.source_coordinate()?.ok_or(())?;
        let intent = self.recipe.intent();
        let source = intent.source();
        let chunk = self.source_frame()?.coordinate();
        let scope = PhysicalArtifactScope::extent_chunk(
            self.store,
            self.format,
            chunk,
            PhysicalByteRange::new(
                source_coordinate.offset(),
                u64::from(source_coordinate.length()),
            )
            .map_err(|_| ())?,
            source.arena_range(),
        );
        let admitted = crate::integrity_ingress::admit_extent_chunk_projection(
            observed,
            scope,
            self.membership,
            trace,
        )
        .map_err(|_| ())?;
        if admitted.page_lsn.get() >= self.recipe.intent_lsn() {
            return Err(());
        }
        let payload = observed
            .bytes()
            .ok_or(())?
            .get(DURABLE_EXTENT_FRAME_HEADER_BYTES + EXTENT_CHUNK_METADATA_BYTES..)
            .ok_or(())?;
        self.digest.update(payload);
        let destination = intent.destination();
        let chunk = ExtentChunkCoordinate::new(
            destination.record(),
            destination.extent_cell(),
            destination.payload_bytes(),
            self.logical_offset,
            self.ordinal,
        )
        .ok_or(())?;
        let mut frame = prepare_extent_chunk(self.format, chunk, payload.len()).map_err(|_| ())?;
        frame.payload_mut().copy_from_slice(payload);
        let mut bytes = frame.seal();
        encode_data_frame_page_lsn(
            &mut bytes,
            DurableFrameKind::Extent,
            PhysicalPageLsn::new(self.recipe.intent_lsn()),
        )
        .map_err(|_| ())?;
        let relative = source_coordinate.offset() - source.arena_range().offset();
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena {
                arena: destination.arena_range().arena().get(),
            },
            destination.arena_range().offset() + relative,
            bytes.len() as u32,
        )
        .ok_or(())?;
        self.ordinal = self.ordinal.checked_add(1).ok_or(())?;
        self.logical_offset = self
            .logical_offset
            .checked_add(payload.len() as u64)
            .ok_or(())?;
        Ok((coordinate, bytes))
    }

    pub(crate) fn finish(&mut self) -> Result<(RecordFrameCoordinate, Vec<u8>), ()> {
        let intent = self.recipe.intent();
        if self.finished
            || self.ordinal != intent.chunk_count().checked_add(1).ok_or(())?
            || self.logical_offset != intent.source().payload_bytes()
            || <[u8; 32]>::from(self.digest.clone().finalize()) != intent.source_digest()
        {
            return Err(());
        }
        self.finished = true;
        let bytes = intent
            .destination_manifest(self.format)
            .ok_or(())?
            .encode(self.format);
        let range = intent.destination().arena_range();
        let coordinate = RecordFrameCoordinate::new(
            RecordArtifactFile::ExtentArena {
                arena: range.arena().get(),
            },
            range.offset(),
            bytes.len() as u32,
        )
        .ok_or(())?;
        Ok((coordinate, bytes))
    }
}
