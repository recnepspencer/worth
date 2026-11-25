use std::any::Any;
use std::sync::Weak;

use worth_execution::ExecutionImmutableBytes as ImmBytes;
use worth_testkit_relational::durable_medium::OwnedNativeReadSnapshot;

use super::owned_medium::{bounded_bytes, downcast_snapshot, map_owned_denial};
use super::{
    ColdSource, MemberCustody, MemberLoc, OwnedTestDurableMedium, ReadBounds, ReadPayload,
    ReadRefusal, Req, RetireObs, RetireRefusal, Seq, StorageCause,
};
use crate::publication::patch::data::PatchStreamPosition;

impl ColdSource for OwnedTestDurableMedium {
    fn read_member_prefix_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        member: MemberLoc,
        length: usize,
    ) -> Result<ImmBytes, ReadRefusal> {
        self.check_and_record_member_read(member.sha256)
            .map_err(ReadRefusal::Storage)?;
        downcast_snapshot(original)?
            .member_prefix(request, member.sha256, member.length, length)
            .map_err(map_owned_denial)
    }
    fn read_initial_inputs_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        limit: u64,
    ) -> Result<Option<ImmBytes>, ReadRefusal> {
        downcast_snapshot(original)?
            .initial_inputs(request, limit)
            .map_err(map_owned_denial)
    }
    fn checkpoint_read_protection(
        &self,
        request: &Req<'_, '_>,
    ) -> Result<(ReadBounds, ReadPayload), ReadRefusal> {
        let contents = self.lock();
        let snapshot = OwnedNativeReadSnapshot::capture(
            request,
            contents
                .commits
                .entries
                .iter()
                .map(|(id, b)| (*id, b.as_ref())),
            contents
                .transitions
                .entries
                .iter()
                .map(|(id, b)| (*id, b.as_ref())),
            &contents.members,
            contents.initial_inputs.as_ref().map(|a| a.as_bytes()),
        )
        .map_err(map_owned_denial)?;
        let [canonical, transitions] = snapshot.bounds();
        Ok((
            ReadBounds {
                selected_checkpoint: contents.checkpoint.is_some(),
                canonical: PatchStreamPosition(canonical),
                transitions: Seq(transitions),
            },
            snapshot,
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &Req<'_, '_>,
    ) -> Result<ReadPayload, ReadRefusal> {
        let contents = self.lock();
        OwnedNativeReadSnapshot::capture(
            request,
            std::iter::empty(),
            std::iter::empty(),
            &contents.members,
            contents.initial_inputs.as_ref().map(|a| a.as_bytes()),
        )
        .map(|original| original as ReadPayload)
        .map_err(map_owned_denial)
    }
    fn read_commit_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<ImmBytes, ReadRefusal> {
        self.check_and_record_commit_read(position)
            .map_err(ReadRefusal::Storage)?;
        downcast_snapshot(original)?
            .canonical(request, position.0, limit)
            .map_err(map_owned_denial)
    }
    fn read_transition_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        sequence: Seq,
        limit: u64,
    ) -> Result<ImmBytes, ReadRefusal> {
        downcast_snapshot(original)?
            .transition(request, sequence.0, limit)
            .map_err(map_owned_denial)
    }
    fn read_member_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: MemberLoc,
        limit: u64,
    ) -> Result<ImmBytes, ReadRefusal> {
        self.check_and_record_member_read(locator.sha256)
            .map_err(ReadRefusal::Storage)?;
        downcast_snapshot(original)?
            .member(request, locator.sha256, locator.length, limit)
            .map_err(map_owned_denial)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn Any + Send + Sync),
    ) -> Result<(), RetireRefusal> {
        if original.is::<()>() || original.is::<OwnedNativeReadSnapshot>() {
            Ok(())
        } else {
            Err(RetireRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _: &Req<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> RetireObs {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RetireObs::Complete,
            Err(cause) => RetireObs::Refused(cause),
        }
    }
    fn retain_members_for_request(
        &self,
        request: &Req<'_, '_>,
        _: &[MemberLoc],
        issuer: Weak<dyn ColdSource>,
    ) -> Result<MemberCustody, ReadRefusal> {
        MemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }
    fn retain_members(&self, _: &[MemberLoc]) -> Result<MemberCustody, StorageCause> {
        Ok(MemberCustody::new(()))
    }
    fn read_member(&self, locator: MemberLoc, limit: u64) -> Result<Vec<u8>, StorageCause> {
        self.check_and_record_member_read(locator.sha256)?;
        bounded_bytes(
            self.lock().members.get(locator.sha256, locator.length),
            limit,
        )
    }
    fn read_commit(
        &self,
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, StorageCause> {
        self.check_and_record_commit_read(position)?;
        bounded_bytes(
            self.lock()
                .commits
                .entries
                .get(&position.0)
                .map(AsRef::as_ref),
            limit,
        )
    }
}
