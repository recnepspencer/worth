use super::RecordPublicationDirector;
use crate::physical_runtime::durability::{
    DisplacedArtifact, PhysicalRetirementDenial, RetiredArtifact,
};
use crate::physical_runtime::record_serving::{
    access::manifest_routing::ManifestDiscoveryCounterSnapshot,
    planning::free_space_routing::FreeSpaceReader,
};
use worth_store_physical_format::{ExtentArenaId, FreeSpaceKey};

impl RecordPublicationDirector {
    pub(in crate::physical_runtime::record_serving) fn prepare_empty_arena_retirement(
        &self,
        source_root: u64,
        arena: ExtentArenaId,
    ) -> Result<(), PhysicalRetirementDenial> {
        let (root, free) = self.root_owner.snapshot();
        if root.generation() != source_root {
            return Err(PhysicalRetirementDenial::Waiting);
        }
        let mut pending = self
            .arena_evacuation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let progress = pending.as_mut().ok_or(PhysicalRetirementDenial::Absent)?;
        if progress.arena != arena || progress.empty_at != Some(source_root) {
            return Err(PhysicalRetirementDenial::Retained);
        }
        if progress.retirement_root.is_some() {
            return Ok(());
        }
        let bytes =
            std::num::NonZeroU64::new(u64::from(self.format.declaration().page_size().bytes()))
                .ok_or(PhysicalRetirementDenial::Waiting)?;
        let allocation = self
            .residency
            .begin_foreground_write_operation(bytes)
            .map_err(|_| PhysicalRetirementDenial::Waiting)?;
        let run = FreeSpaceReader::serving(self.residency.clone(), self.format, self.access, &free)
            .locate(
                &allocation,
                FreeSpaceKey::arena(arena, 0),
                &mut ManifestDiscoveryCounterSnapshot::default(),
            )
            .map_err(|_| PhysicalRetirementDenial::Retained)?
            .and_then(|entry| entry.arena_free_range());
        if !run.is_some_and(|range| range.offset() == 0 && range.length() == free.arena_capacity())
        {
            return Err(PhysicalRetirementDenial::Retained);
        }
        let displaced = DisplacedArtifact {
            source_root,
            artifact: RetiredArtifact::Arena {
                arena: arena.get(),
                generation: source_root,
            },
            bytes: free.arena_capacity(),
        };
        if !self
            .root_owner
            .publication_admission()
            .admit_displaced_arena(displaced)
        {
            return Err(PhysicalRetirementDenial::Waiting);
        }
        progress.retirement_root = Some(source_root);
        Ok(())
    }

    pub(super) fn expose_arena_retirement(
        &self,
        arena: u64,
    ) -> Result<(), PhysicalRetirementDenial> {
        let current = self.root_owner.snapshot().0.generation();
        let mut pending = self
            .arena_evacuation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let progress = pending
            .as_mut()
            .ok_or(PhysicalRetirementDenial::Unresolved)?;
        if progress.arena.get() != arena || progress.empty_at != Some(current) {
            return Err(PhysicalRetirementDenial::Retained);
        }
        progress.exclusion.expose_to_retirement();
        Ok(())
    }

    pub(super) fn forget_published_arena(
        &self,
        arena: u64,
    ) -> Result<(), PhysicalRetirementDenial> {
        let mut pending = self
            .arena_evacuation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if pending
            .as_ref()
            .is_some_and(|progress| progress.arena.get() != arena)
        {
            return Err(PhysicalRetirementDenial::Retained);
        }
        if let Some(progress) = pending.take() {
            progress.exclusion.forget_after_publication();
        }
        let mut preparation = self.preparation.lock().unwrap_or_else(|e| e.into_inner());
        if let Some(recovered) = preparation.recovered_retiring_arena {
            if recovered.get() != arena {
                return Err(PhysicalRetirementDenial::Retained);
            }
            if let Some(owner) = &preparation.arenas {
                owner
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .forget_evacuated(recovered);
            }
            preparation.recovered_retiring_arena = None;
        }
        Ok(())
    }
}
