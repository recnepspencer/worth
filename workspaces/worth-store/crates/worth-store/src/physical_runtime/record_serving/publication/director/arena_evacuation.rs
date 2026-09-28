use super::RecordPublicationDirector;
use crate::physical_runtime::record_serving::{
    access::manifest_routing::{ManifestRangeCursor, ManifestReader},
    arena::{ArenaEvacuationLease, ExtentArenaCapacity},
    planning::inline_plan_failure::manifest_lookup_failure,
    AdmittedRecordPlacementPolicy, RecordAppendDenial, RecordAppendError,
};
use std::num::NonZeroU64;
use worth_store_physical_format::{
    CurrentPhysicalRecordPlacement, DurableExtentRecordPlacement, ExtentArenaId,
    PersistedRecordIdentity,
};

const SCAN_QUANTUM: usize = 64;

pub(in crate::physical_runtime::record_serving) enum ArenaEvacuationSelection {
    Idle,
    Continue,
    Extent {
        source: DurableExtentRecordPlacement,
    },
    ReleasePending {
        arena: ExtentArenaId,
    },
    Empty {
        source_root: u64,
        arena: ExtentArenaId,
    },
}

pub(super) struct ArenaEvacuationProgress {
    pub(super) arena: ExtentArenaId,
    pub(super) exclusion: ArenaEvacuationLease,
    after: Option<PersistedRecordIdentity>,
    best: Option<(bool, DurableExtentRecordPlacement)>,
    selected: Option<DurableExtentRecordPlacement>,
    pub(super) scanned_entries: u64,
    capacity: ExtentArenaCapacity,
    pub(super) empty_at: Option<u64>,
    pub(super) retirement_root: Option<u64>,
}

impl RecordPublicationDirector {
    /// One bounded routing quantum. A selected source is not advanced until a
    /// later root routes that record elsewhere, so failed preparations retry it.
    pub(in crate::physical_runtime::record_serving) fn select_arena_evacuation(
        &self,
        placement: AdmittedRecordPlacementPolicy,
    ) -> Result<ArenaEvacuationSelection, RecordAppendError> {
        let (root, free) = self.root_owner.snapshot();
        let page = u64::from(self.format.declaration().page_size().bytes());
        let depth = root
            .routing_root()
            .map_or(0, |reference| u64::from(reference.level()));
        let bytes = page
            .checked_mul(depth + 3)
            .and_then(NonZeroU64::new)
            .ok_or_else(pressure)?;
        let allocation = self
            .residency
            .begin_foreground_write_operation(bytes)
            .map_err(|denial| {
                RecordAppendError::Denied(RecordAppendDenial::from_residency(denial))
            })?;
        let mut pending = self
            .arena_evacuation
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        if pending.is_none() {
            let owner = self.arena_allocation_owner(&allocation, placement, &free)?;
            let arena = owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .sparse_candidate(free.next_arena(), placement.arena_evacuation());
            let Some(arena) = arena else {
                return Ok(ArenaEvacuationSelection::Idle);
            };
            let exclusion = ArenaEvacuationLease::acquire(&owner, arena).map_err(|_| pressure())?;
            *pending = Some(ArenaEvacuationProgress {
                arena,
                exclusion,
                after: None,
                best: None,
                selected: None,
                scanned_entries: 0,
                capacity: placement.arena_capacity(),
                empty_at: None,
                retirement_root: None,
            });
        }
        let progress = pending.as_mut().expect("selection owns evacuation state");
        if progress.capacity != placement.arena_capacity() {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PlacementFormatMismatch,
            ));
        }
        let owner = self.arena_allocation_owner(&allocation, placement, &free)?;
        if let Some(selected) = progress.selected {
            if self.current_extent_source(&root, selected.record())? == selected {
                return Ok(ArenaEvacuationSelection::Extent { source: selected });
            }
            if !owner
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .contains_released(selected.arena_range())
            {
                return Ok(ArenaEvacuationSelection::ReleasePending {
                    arena: progress.arena,
                });
            }
            progress.selected = None;
            progress.after = None;
            progress.best = None;
        }
        let reader = ManifestReader::serving(
            self.residency.clone(),
            self.format,
            self.access,
            root.clone(),
        );
        let mut cursor = ManifestRangeCursor::new(reader);
        cursor
            .seek(&allocation, root.routing_root(), progress.after)
            .map_err(manifest_lookup_failure)?;
        for _ in 0..SCAN_QUANTUM {
            let Some(route) = cursor.next(&allocation).map_err(manifest_lookup_failure)? else {
                if let Some((_, source)) = progress.best.take() {
                    if self.current_extent_source(&root, source.record())? != source {
                        progress.after = None;
                        return Ok(ArenaEvacuationSelection::Continue);
                    }
                    let allocator = owner.lock().unwrap_or_else(|e| e.into_inner());
                    match allocator.preflight_release(source.arena_range()) {
                        Ok(()) => {},
                        Err(crate::physical_runtime::record_serving::arena::ArenaAllocationDenial::RangeBudget) => {
                            progress.after = None;
                            return Ok(ArenaEvacuationSelection::ReleasePending { arena: progress.arena });
                        },
                        Err(_) => return Err(RecordAppendError::Denied(RecordAppendDenial::PublishedLayoutDamaged)),
                    }
                    progress.selected = Some(source);
                    return Ok(ArenaEvacuationSelection::Extent { source });
                }
                progress.empty_at = Some(root.generation());
                return Ok(ArenaEvacuationSelection::Empty {
                    source_root: root.generation(),
                    arena: progress.arena,
                });
            };
            if Some(route.record()) == progress.after {
                continue;
            }
            progress.scanned_entries = progress.scanned_entries.saturating_add(1);
            if let CurrentPhysicalRecordPlacement::Extent(source) = route {
                if source.arena_range().arena() == progress.arena {
                    let touches = owner
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .release_touches_free(source.arena_range());
                    let rank = (!touches, source.arena_range().offset());
                    if progress.best.is_none_or(|(old_touches, old)| {
                        rank < (!old_touches, old.arena_range().offset())
                    }) {
                        progress.best = Some((touches, source));
                    }
                }
            }
            progress.after = Some(route.record());
        }
        Ok(ArenaEvacuationSelection::Continue)
    }
}

impl super::PhysicalRecordSubmission {
    /// Actual routing entries inspected across bounded evacuation polls.
    pub fn arena_evacuation_scan_entries(&self) -> u64 {
        self.director.upgrade().map_or(0, |director| {
            director
                .arena_evacuation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .as_ref()
                .map_or(0, |progress| progress.scanned_entries)
        })
    }

    /// Certification selects an actual routed extent without filling a 64 MiB
    /// arena. The source exclusion, copy, WAL, and publication remain the
    /// ordinary production owners; producer eligibility has its own scale test.
    #[cfg(feature = "certification-test-authority")]
    pub fn certification_begin_selected_extent_copy(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: crate::physical_runtime::PhysicalMutationRequest,
        record: PersistedRecordIdentity,
    ) -> Result<
        Result<
            super::extent_copy::PhysicalExtentCopyProgress,
            crate::physical_runtime::PhysicalMutationPreparationOutcome,
        >,
        RecordAppendError,
    > {
        let director = self.director.upgrade().ok_or_else(pressure)?;
        director.certification_begin_selected_extent_copy(placement, request, record)
    }
}

#[cfg(feature = "certification-test-authority")]
impl RecordPublicationDirector {
    fn certification_begin_selected_extent_copy(
        &self,
        placement: AdmittedRecordPlacementPolicy,
        request: crate::physical_runtime::PhysicalMutationRequest,
        record: PersistedRecordIdentity,
    ) -> Result<
        Result<
            super::extent_copy::PhysicalExtentCopyProgress,
            crate::physical_runtime::PhysicalMutationPreparationOutcome,
        >,
        RecordAppendError,
    > {
        if !placement.admits(self.format) {
            return Err(RecordAppendError::Denied(
                RecordAppendDenial::PlacementFormatMismatch,
            ));
        }
        let (root, free) = self.root_owner.snapshot();
        let selected = self.current_extent_source(&root, record)?;
        let allocation = self.copy_allocation()?;
        let owner = self.arena_allocation_owner(&allocation, placement, &free)?;
        let exclusion = ArenaEvacuationLease::acquire(&owner, selected.arena_range().arena())
            .map_err(|_| pressure())?;
        {
            let mut pending = self
                .arena_evacuation
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            if pending.is_some() {
                return Err(pressure());
            }
            *pending = Some(ArenaEvacuationProgress {
                arena: selected.arena_range().arena(),
                exclusion,
                after: None,
                best: None,
                selected: Some(selected),
                scanned_entries: 0,
                capacity: placement.arena_capacity(),
                empty_at: None,
                retirement_root: None,
            });
        }
        let result = self.begin_extent_copy(placement, request, selected);
        if !matches!(&result, Ok(Ok(_))) {
            self.arena_evacuation
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .take();
        }
        result
    }
}

fn pressure() -> RecordAppendError {
    RecordAppendError::Denied(RecordAppendDenial::PhysicalPressure)
}
