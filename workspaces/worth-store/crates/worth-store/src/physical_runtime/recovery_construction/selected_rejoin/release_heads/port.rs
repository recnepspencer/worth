//! Mechanical storage and actual-read port for the canonical Integrity walk.

use worth_store_physical_format::{
    RecordArtifactFile, ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
};
use worth_store_physical_integrity::ReleaseCustodyHeadWalkPort;

use super::super::control_frames::SelectedArtifactSlice;
use super::super::resident::StoreRejoinResidentLedger;
use super::super::SelectedMediaRejoinDenial as Denial;

pub(super) struct StoreHeadWalkPort<'a, Read> {
    resident: &'a mut StoreRejoinResidentLedger,
    read: Read,
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    slices: Vec<SelectedArtifactSlice>,
}

impl<'a, Read> StoreHeadWalkPort<'a, Read> {
    pub(super) fn new(
        resident: &'a mut StoreRejoinResidentLedger,
        read: Read,
        entries: Vec<ReleaseCustodyHeadEntryV1>,
        slices: Vec<SelectedArtifactSlice>,
    ) -> Self {
        Self {
            resident,
            read,
            entries,
            slices,
        }
    }

    pub(super) fn into_outputs(
        self,
    ) -> (Vec<ReleaseCustodyHeadEntryV1>, Vec<SelectedArtifactSlice>) {
        (self.entries, self.slices)
    }
}

impl<Read> ReleaseCustodyHeadWalkPort for StoreHeadWalkPort<'_, Read>
where
    Read: FnMut(
        ReleaseCustodyHeadBlockReferenceV1,
        u64,
        &mut StoreRejoinResidentLedger,
    ) -> Result<Vec<u8>, Denial>,
{
    type Error = Denial;

    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Self::Error> {
        self.resident.reserve_vec(count).map_err(Denial::Resident)
    }

    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Self::Error> {
        self.resident
            .grow_vec(values, additional)
            .map_err(Denial::Resident)
    }

    fn discard_vec<T>(&mut self, values: Vec<T>) {
        let charged = self
            .resident
            .vector_bytes(&values)
            .expect("a previously charged vector has representable capacity");
        drop(values);
        self.resident.release(charged);
    }

    fn read_node(
        &mut self,
        reference: ReleaseCustodyHeadBlockReferenceV1,
        remaining: u64,
    ) -> Result<Vec<u8>, Self::Error> {
        self.resident
            .transient(remaining)
            .map_err(Denial::Resident)?;
        (self.read)(reference, remaining, self.resident)
    }

    fn visit_node(
        &mut self,
        reference: ReleaseCustodyHeadBlockReferenceV1,
        frame: &[u8],
    ) -> Result<(), Self::Error> {
        if self.slices.len() >= self.slices.capacity() {
            return Err(Denial::BoundExceeded);
        }
        let artifact = RecordArtifactFile::ReleaseCustodyHeadBlock {
            generation: reference.generation(),
            block: reference.block(),
        };
        self.slices.push(
            SelectedArtifactSlice::observed(artifact, 0, frame, true)
                .ok_or(Denial::BoundExceeded)?,
        );
        Ok(())
    }

    fn visit_entry(&mut self, entry: ReleaseCustodyHeadEntryV1) -> Result<(), Self::Error> {
        if self.entries.len() >= self.entries.capacity() {
            return Err(Denial::BoundExceeded);
        }
        self.entries.push(entry);
        Ok(())
    }
}
