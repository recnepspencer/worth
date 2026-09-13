use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FrameSlotId(u32);

#[derive(Debug)]
enum FrameSlot {
    Exact {
        coordinate: RecordFrameCoordinate,
        frame: FrameEntry,
    },
    Bounded {
        artifact: RecordArtifactFile,
        entry: bounded_frame_admission::BoundedFrameEntry,
    },
}

/// One fixed-capacity frame population with two semantic lookup indexes.
///
/// A bounded fault starts in one slot. Resolution adds its exact-coordinate
/// index to that same slot, so aliases never duplicate the governed frame
/// entry or grow either index after pool admission.
#[derive(Debug)]
pub(super) struct FrameTable {
    exact_index: HashMap<RecordFrameCoordinate, FrameSlotId>,
    bounded_index: HashMap<RecordArtifactFile, FrameSlotId>,
    slots: Vec<Option<FrameSlot>>,
    free_slots: Vec<FrameSlotId>,
}

impl FrameTable {
    pub(super) fn minimum_metadata_bytes(frame_count: usize) -> Option<usize> {
        frame_count
            .checked_mul(exact_index_bytes())
            .and_then(|exact| {
                frame_count
                    .checked_mul(bounded_index_bytes())
                    .and_then(|bounded| exact.checked_add(bounded))
            })
            .and_then(|bytes| {
                frame_count
                    .checked_mul(std::mem::size_of::<Option<FrameSlot>>())
                    .and_then(|slots| bytes.checked_add(slots))
            })
            .and_then(|bytes| {
                frame_count
                    .checked_mul(std::mem::size_of::<FrameSlotId>())
                    .and_then(|free| bytes.checked_add(free))
            })
    }

    pub(super) fn open(frame_count: usize) -> Result<Self, PhysicalResidencyDenial> {
        let mut exact_index = HashMap::new();
        exact_index
            .try_reserve(frame_count)
            .map_err(|_| PhysicalResidencyDenial::AllocationFailed)?;
        let mut bounded_index = HashMap::new();
        bounded_index
            .try_reserve(frame_count)
            .map_err(|_| PhysicalResidencyDenial::AllocationFailed)?;
        let mut slots = Vec::new();
        slots
            .try_reserve_exact(frame_count)
            .map_err(|_| PhysicalResidencyDenial::AllocationFailed)?;
        slots.resize_with(frame_count, || None);
        let mut free_slots = Vec::new();
        free_slots
            .try_reserve_exact(frame_count)
            .map_err(|_| PhysicalResidencyDenial::AllocationFailed)?;
        for index in (0..frame_count).rev() {
            free_slots.push(FrameSlotId(
                u32::try_from(index).map_err(|_| PhysicalResidencyDenial::AllocationFailed)?,
            ));
        }
        Ok(Self {
            exact_index,
            bounded_index,
            slots,
            free_slots,
        })
    }

    pub(super) fn allocated_metadata_bytes(&self) -> Option<usize> {
        self.exact_index
            .capacity()
            .checked_mul(exact_index_bytes())
            .and_then(|exact| {
                self.bounded_index
                    .capacity()
                    .checked_mul(bounded_index_bytes())
                    .and_then(|bounded| exact.checked_add(bounded))
            })
            .and_then(|bytes| {
                self.slots
                    .capacity()
                    .checked_mul(std::mem::size_of::<Option<FrameSlot>>())
                    .and_then(|slots| bytes.checked_add(slots))
            })
            .and_then(|bytes| {
                self.free_slots
                    .capacity()
                    .checked_mul(std::mem::size_of::<FrameSlotId>())
                    .and_then(|free| bytes.checked_add(free))
            })
    }

    pub(super) fn get(&self, coordinate: &RecordFrameCoordinate) -> Option<&FrameEntry> {
        let slot = self.slot(*self.exact_index.get(coordinate)?);
        match slot {
            FrameSlot::Exact { frame, .. } => Some(frame),
            FrameSlot::Bounded { entry, .. } => entry.resident_frame(),
        }
    }

    pub(super) fn get_mut(
        &mut self,
        coordinate: &RecordFrameCoordinate,
    ) -> Option<&mut FrameEntry> {
        let id = *self.exact_index.get(coordinate)?;
        match self.slot_mut(id) {
            FrameSlot::Exact { frame, .. } => Some(frame),
            FrameSlot::Bounded { entry, .. } => entry.resident_frame_mut(),
        }
    }

    pub(super) fn contains_key(&self, coordinate: &RecordFrameCoordinate) -> bool {
        self.exact_index.contains_key(coordinate)
    }

    pub(super) fn contains_artifact_alias(&self, artifact: RecordArtifactFile) -> bool {
        self.bounded_index.contains_key(&artifact)
    }

    pub(super) fn values(&self) -> impl Iterator<Item = &FrameEntry> {
        self.slots.iter().filter_map(|slot| match slot.as_ref()? {
            FrameSlot::Exact { frame, .. } => Some(frame),
            FrameSlot::Bounded { entry, .. } => entry.resident_frame(),
        })
    }

    pub(super) fn values_mut(&mut self) -> impl Iterator<Item = &mut FrameEntry> {
        self.slots
            .iter_mut()
            .filter_map(|slot| match slot.as_mut()? {
                FrameSlot::Exact { frame, .. } => Some(frame),
                FrameSlot::Bounded { entry, .. } => entry.resident_frame_mut(),
            })
    }

    pub(super) fn slot_count(&self) -> usize {
        self.slots.len()
    }

    pub(super) fn resident_entries_from(
        &self,
        start: usize,
    ) -> impl Iterator<Item = (usize, RecordFrameCoordinate, &FrameEntry)> {
        self.slots
            .iter()
            .enumerate()
            .skip(start)
            .filter_map(|(index, slot)| match slot.as_ref()? {
                FrameSlot::Exact { coordinate, frame }
                    if matches!(frame.state, FrameState::Resident(_)) =>
                {
                    Some((index, *coordinate, frame))
                }
                FrameSlot::Exact { .. } => None,
                FrameSlot::Bounded { artifact, entry } => entry
                    .resident_coordinate_for_artifact(*artifact)
                    .zip(entry.resident_frame())
                    .filter(|(_, frame)| matches!(frame.state, FrameState::Resident(_)))
                    .map(|(coordinate, frame)| (index, coordinate, frame)),
            })
    }

    pub(super) fn insert(
        &mut self,
        coordinate: RecordFrameCoordinate,
        frame: FrameEntry,
    ) -> Option<FrameEntry> {
        if let Some(existing) = self.get_mut(&coordinate) {
            return Some(std::mem::replace(existing, frame));
        }
        let complete_artifact = frame.artifact_posture != FrameArtifactPosture::Fragment;
        let id = self.claim_slot(FrameSlot::Exact { coordinate, frame });
        assert!(
            self.exact_index.insert(coordinate, id).is_none(),
            "an absent exact coordinate cannot replace an index"
        );
        if complete_artifact {
            assert!(
                self.bounded_index
                    .insert(coordinate.artifact(), id)
                    .is_none(),
                "a complete candidate requires an absent artifact identity"
            );
        }
        None
    }

    pub(super) fn remove(&mut self, coordinate: &RecordFrameCoordinate) -> Option<FrameEntry> {
        let id = self.exact_index.remove(coordinate)?;
        if self.bounded_index.get(&coordinate.artifact()) == Some(&id) {
            self.bounded_index.remove(&coordinate.artifact());
        }
        let slot = self.release_slot(id);
        match slot {
            FrameSlot::Exact { frame, .. } => Some(frame),
            FrameSlot::Bounded { entry, .. } => {
                assert_eq!(
                    self.bounded_index.remove(&coordinate.artifact()),
                    None,
                    "a bounded resident has one alias index"
                );
                entry.into_resident_frame()
            }
        }
    }

    pub(super) fn get_complete_artifact(
        &self,
        key: &PhysicalBoundedFrameKey,
    ) -> Option<&FrameEntry> {
        match self.slot(*self.bounded_index.get(&key.artifact())?) {
            FrameSlot::Exact { frame, .. } => Some(frame),
            FrameSlot::Bounded { .. } => None,
        }
    }

    pub(super) fn get_bounded(
        &self,
        key: &PhysicalBoundedFrameKey,
    ) -> Option<&bounded_frame_admission::BoundedFrameEntry> {
        match self.slot(*self.bounded_index.get(&key.artifact())?) {
            FrameSlot::Bounded { entry, .. } => Some(entry),
            FrameSlot::Exact { .. } => None,
        }
    }

    pub(super) fn get_bounded_mut(
        &mut self,
        key: &PhysicalBoundedFrameKey,
    ) -> Option<&mut bounded_frame_admission::BoundedFrameEntry> {
        let id = *self.bounded_index.get(&key.artifact())?;
        match self.slot_mut(id) {
            FrameSlot::Bounded { entry, .. } => Some(entry),
            FrameSlot::Exact { .. } => None,
        }
    }

    pub(super) fn insert_bounded(
        &mut self,
        key: PhysicalBoundedFrameKey,
        entry: bounded_frame_admission::BoundedFrameEntry,
    ) {
        assert!(
            !self.bounded_index.contains_key(&key.artifact()),
            "bounded admission requires an absent alias"
        );
        let id = self.claim_slot(FrameSlot::Bounded {
            artifact: key.artifact(),
            entry,
        });
        assert!(
            self.bounded_index.insert(key.artifact(), id).is_none(),
            "an absent bounded identity cannot replace an index"
        );
    }

    pub(super) fn resolve_bounded(
        &mut self,
        key: PhysicalBoundedFrameKey,
        coordinate: RecordFrameCoordinate,
        frame: FrameEntry,
    ) {
        assert!(
            !self.exact_index.contains_key(&coordinate),
            "bounded completion requires an absent exact coordinate"
        );
        let id = *self
            .bounded_index
            .get(&key.artifact())
            .expect("bounded completion retains its alias index");
        self.get_bounded_mut(&key)
            .expect("bounded completion retains its slot")
            .resolve(coordinate.length(), frame);
        assert!(
            self.exact_index.insert(coordinate, id).is_none(),
            "bounded completion installs one exact index"
        );
    }

    pub(super) fn remove_bounded(
        &mut self,
        key: &PhysicalBoundedFrameKey,
    ) -> Option<bounded_frame_admission::BoundedFrameEntry> {
        let id = *self.bounded_index.get(&key.artifact())?;
        if matches!(self.slot(id), FrameSlot::Exact { .. }) {
            return None;
        }
        self.bounded_index.remove(&key.artifact());
        let slot = self.release_slot(id);
        match slot {
            FrameSlot::Bounded { entry, .. } => {
                if let Some(coordinate) = entry.resident_coordinate(*key) {
                    assert_eq!(
                        self.exact_index.remove(&coordinate),
                        Some(id),
                        "a bounded resident has one exact index"
                    );
                }
                Some(entry)
            }
            FrameSlot::Exact { .. } => None,
        }
    }

    fn claim_slot(&mut self, slot: FrameSlot) -> FrameSlotId {
        let id = self
            .free_slots
            .pop()
            .expect("frame accounting reserves one preallocated table slot");
        let target = self
            .slots
            .get_mut(id.0 as usize)
            .expect("a frame slot identity is in bounds");
        assert!(target.is_none(), "a free frame slot is empty");
        *target = Some(slot);
        id
    }

    fn release_slot(&mut self, id: FrameSlotId) -> FrameSlot {
        let slot = self
            .slots
            .get_mut(id.0 as usize)
            .and_then(Option::take)
            .expect("an indexed frame slot is occupied");
        self.free_slots.push(id);
        slot
    }

    fn slot(&self, id: FrameSlotId) -> &FrameSlot {
        self.slots[id.0 as usize]
            .as_ref()
            .expect("an indexed frame slot is occupied")
    }

    fn slot_mut(&mut self, id: FrameSlotId) -> &mut FrameSlot {
        self.slots[id.0 as usize]
            .as_mut()
            .expect("an indexed frame slot is occupied")
    }
}

fn exact_index_bytes() -> usize {
    std::mem::size_of::<RecordFrameCoordinate>()
        .saturating_add(std::mem::size_of::<FrameSlotId>())
        .saturating_add(32)
}

fn bounded_index_bytes() -> usize {
    std::mem::size_of::<RecordArtifactFile>()
        .saturating_add(std::mem::size_of::<FrameSlotId>())
        .saturating_add(32)
}
