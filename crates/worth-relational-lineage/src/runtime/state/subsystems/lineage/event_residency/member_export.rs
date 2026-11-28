//! Export exact frame bytes through the same event cell and sticky failure state.
use super::*;
impl RelationalDurableColdSource for RuntimeLineageEventResidency {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.custody.source()?.retain_members(members)
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let cell = self.pages.get(&locator).ok_or(Cause::SourceUnavailable)?;
        cell.export_member(|| self.custody.source(), limit)
    }
    fn read_commit(
        &self,
        position: worth_relational_contracts::internal_port::publication::patch::data::patch_position::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.custody.source()?.read_commit(position, limit)
    }
}
