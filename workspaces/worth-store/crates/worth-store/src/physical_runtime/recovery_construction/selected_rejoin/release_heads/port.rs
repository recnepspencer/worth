//! Mechanical native storage and actual-read port for the Integrity walk.

use super::{storage::HeadWalkStorage, Denial, SelectedArtifactSlice};
use worth_store_physical_format::{
    RecordArtifactFile, ReleaseCustodyHeadBlockReferenceV1, ReleaseCustodyHeadEntryV1,
};
use worth_store_physical_integrity::ReleaseCustodyHeadWalkPort;

pub(super) struct StoreHeadWalkPort<'scope, 'owner, Read> {
    entries: Vec<ReleaseCustodyHeadEntryV1>,
    slices: Vec<SelectedArtifactSlice>,
    storage: HeadWalkStorage<'scope, 'owner>,
    read: Read,
}

impl<'scope, 'owner, Read> StoreHeadWalkPort<'scope, 'owner, Read> {
    pub(super) fn new(
        storage: HeadWalkStorage<'scope, 'owner>,
        read: Read,
        entries: Vec<ReleaseCustodyHeadEntryV1>,
        slices: Vec<SelectedArtifactSlice>,
    ) -> Self {
        Self {
            entries,
            slices,
            storage,
            read,
        }
    }
    pub(super) fn into_outputs(
        self,
    ) -> (Vec<ReleaseCustodyHeadEntryV1>, Vec<SelectedArtifactSlice>) {
        (self.entries, self.slices)
    }
}

impl<'scope, 'owner, Read> ReleaseCustodyHeadWalkPort for StoreHeadWalkPort<'scope, 'owner, Read>
where
    Read: FnMut(
        ReleaseCustodyHeadBlockReferenceV1,
        u64,
        &mut HeadWalkStorage<'scope, 'owner>,
    ) -> Result<Vec<u8>, Denial>,
{
    type Error = Denial;
    fn reserve_vec<T>(&mut self, count: usize) -> Result<Vec<T>, Denial> {
        self.storage.reserve_vec(count)
    }
    fn grow_vec<T>(&mut self, values: &mut Vec<T>, additional: usize) -> Result<(), Denial> {
        self.storage.grow_vec(values, additional)
    }
    fn discard_vec<T>(&mut self, values: Vec<T>) {
        self.storage
            .discard_vec(values)
            .expect("discard only charged walk storage");
    }
    fn read_node(
        &mut self,
        reference: ReleaseCustodyHeadBlockReferenceV1,
        remaining: u64,
    ) -> Result<Vec<u8>, Denial> {
        self.storage
            .resident
            .transient(remaining)
            .map_err(Denial::Resident)?;
        (self.read)(reference, remaining, &mut self.storage)
    }
    fn visit_node(
        &mut self,
        reference: ReleaseCustodyHeadBlockReferenceV1,
        frame: &[u8],
    ) -> Result<(), Denial> {
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
    fn visit_entry(&mut self, entry: ReleaseCustodyHeadEntryV1) -> Result<(), Denial> {
        if self.entries.len() >= self.entries.capacity() {
            return Err(Denial::BoundExceeded);
        }
        self.entries.push(entry);
        Ok(())
    }
}
