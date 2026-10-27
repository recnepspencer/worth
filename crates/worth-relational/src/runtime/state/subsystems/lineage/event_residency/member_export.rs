//! Export exact frame bytes through the same event cell and sticky failure state.
use super::{
    event_page, Cause, EventPageCell, EventPageState, Failure, RelationalDurableColdSource,
    RelationalDurableMemberCustody, RelationalDurableMemberLocator, RuntimeLineageEventResidency,
    LINEAGE_EVENT_PAGE_BYTES,
};
impl RelationalDurableColdSource for RuntimeLineageEventResidency {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.source.retain_members(members)
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let cell = self.pages.get(&locator).ok_or(Cause::SourceUnavailable)?;
        cell.export_member(self.source.as_ref(), limit)
    }
    fn read_commit(
        &self,
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.source.read_commit(position, limit)
    }
}
impl EventPageCell {
    fn export_member(
        &self,
        source: &dyn RelationalDurableColdSource,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let mut state = self.state.lock().unwrap_or_else(|error| error.into_inner());
        if let EventPageState::Damaged(failure) = &*state {
            return Err(failure.cause);
        }
        let limit = limit.min(LINEAGE_EVENT_PAGE_BYTES);
        let result = if self.descriptor.member.length > limit {
            Err(Cause::LimitExceeded { length: self.descriptor.member.length, limit })
        } else { source.read_member(self.descriptor.member, limit) }.and_then(|bytes| {
            crate::durability::log::native_checkpoint_codec::member_frame::decode(
                &bytes, self.descriptor.member,
                crate::durability::log::native_checkpoint_codec::member_frame::MemberKind::LineageEvents)?;
            Ok(bytes)
        });
        if let Err(cause) = &result
            && cause.is_proven_damage()
        {
            let Failure::StorageUnavailable(failure) =
                event_page::failure(&self.descriptor, *cause)
            else {
                unreachable!()
            };
            *state = EventPageState::Damaged(failure);
        }
        result
    }
}
