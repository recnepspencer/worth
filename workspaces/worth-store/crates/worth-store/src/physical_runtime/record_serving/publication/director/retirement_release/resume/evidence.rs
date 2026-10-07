use super::*;
use crate::physical_runtime::integrity::resident_admission::{
    free_space::admit_resident_free_space_header, root_manifest::project_loaded_root_manifest,
};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::ManifestDiscoveryCounterSnapshot,
    planning::free_space_routing::FreeSpaceReader,
    residency::record_frame_reader::RecordFrameReader,
};
use worth_store_physical_format::{
    DurableArtifactCrc32c, FreeSpaceHeaderScopeIdentity, FreeSpaceKey, PhysicalGeneration,
    PhysicalTreeIdentity,
};
use worth_store_physical_integrity::{PhysicalArtifactScope, PhysicalByteRange};

impl RecordPublicationDirector {
    pub(super) fn verify_released_root(
        &self,
        state: &PendingRetirementRelease,
        allocation: &worth_store_buffer_pool::OperationAllocationGrant,
    ) -> Result<(), PhysicalRetirementDenial> {
        let fail = |_| PhysicalRetirementDenial::WalPlan;
        let generation = state.release.candidate_generation();
        let reader = RecordFrameReader::serving(self.residency.clone());
        let root_bytes = reader
            .load_bounded(
                allocation,
                RecordArtifactFile::RootManifest { generation },
                384,
            )
            .map_err(fail)?;
        if <[u8; 32]>::from(Sha256::digest(&*root_bytes)) != state.release.candidate_digest() {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let context = self.residency.resident_admission_context();
        let root = project_loaded_root_manifest(
            root_bytes.lease(),
            self.residency.store_identity(),
            self.format.declaration(),
            generation,
            context.clone(),
        )
        .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let bytes = reader
            .load_bounded(
                allocation,
                RecordArtifactFile::FreeSpaceManifest { generation },
                216,
            )
            .map_err(fail)?;
        let identity = FreeSpaceHeaderScopeIdentity::new(
            PhysicalGeneration::from_raw(generation)
                .map_err(|_| PhysicalRetirementDenial::WalPlan)?,
            PhysicalTreeIdentity::new(root.tree_identity())
                .ok_or(PhysicalRetirementDenial::WalPlan)?,
            root.free_space_root(),
            DurableArtifactCrc32c::new(root.free_space_checksum()),
        );
        let scope = PhysicalArtifactScope::free_space_header(
            self.residency.store_identity(),
            self.format.declaration(),
            identity,
            PhysicalByteRange::new(0, bytes.len() as u64)
                .map_err(|_| PhysicalRetirementDenial::WalPlan)?,
        );
        let admitted = admit_resident_free_space_header(bytes.lease(), scope, context.clone())
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        let (header, format) = admitted
            .with_owner_decoder(context, |view| view.project_header(u16::MAX))
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
        if format != self.format.declaration() {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        let reader =
            FreeSpaceReader::serving(self.residency.clone(), self.format, self.access, &header);
        if let RetiredArtifact::Arena { arena, .. } = state.displaced.artifact {
            let arena = worth_store_physical_format::ExtentArenaId::new(arena)
                .ok_or(PhysicalRetirementDenial::WalPlan)?;
            let prior = reader
                .floor(
                    allocation,
                    FreeSpaceKey::arena(arena, u64::MAX),
                    &mut ManifestDiscoveryCounterSnapshot::default(),
                )
                .map_err(|_| PhysicalRetirementDenial::WalPlan)?;
            if prior
                .and_then(|entry| entry.arena_free_range())
                .is_some_and(|range| range.arena() == arena)
            {
                return Err(PhysicalRetirementDenial::WalPlan);
            }
            return Ok(());
        }
        let RetiredArtifact::Extent { range, .. } = state.displaced.artifact else {
            return Err(PhysicalRetirementDenial::WalPlan);
        };
        let entry = reader
            .floor(
                allocation,
                FreeSpaceKey::arena(range.arena(), range.offset()),
                &mut ManifestDiscoveryCounterSnapshot::default(),
            )
            .map_err(|_| PhysicalRetirementDenial::WalPlan)?
            .ok_or(PhysicalRetirementDenial::WalPlan)?;
        let released = entry
            .arena_free_range()
            .ok_or(PhysicalRetirementDenial::WalPlan)?;
        if released.arena() != range.arena()
            || released.offset() > range.offset()
            || released.end() < range.end()
            || entry.generation() <= state.displaced.source_root
            || entry.generation() > generation
        {
            return Err(PhysicalRetirementDenial::WalPlan);
        }
        Ok(())
    }
}
