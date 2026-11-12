//! Physical byte damage on an actual immutable member source.
use super::immutable_members::ImmutableCheckpointMemberSource;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::{
    durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug)]
pub struct ChangedMemberSource {
    pub producer: Arc<ImmutableCheckpointMemberSource>,
    target: Mutex<Option<RelationalDurableMemberLocator>>,
    corrupt: AtomicBool,
}
impl ChangedMemberSource {
    pub fn new(producer: Arc<ImmutableCheckpointMemberSource>) -> Arc<Self> {
        Arc::new(Self {
            producer,
            target: Mutex::new(None),
            corrupt: AtomicBool::new(false),
        })
    }
    pub fn corrupt_member(&self, member: RelationalDurableMemberLocator) {
        *self.target.lock().unwrap() = Some(member);
        self.corrupt.store(true, Ordering::Release);
    }
    pub fn repair(&self) {
        self.corrupt.store(false, Ordering::Release);
    }
    fn changes(&self, member: RelationalDurableMemberLocator) -> bool {
        self.corrupt.load(Ordering::Acquire) && *self.target.lock().unwrap() == Some(member)
    }
}
impl RelationalDurableColdSource for ChangedMemberSource {
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        self.producer.checkpoint_selected_read_protection(request)
    }
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        self.producer.validate_member_custody_retirement(original)
    }
    fn advance_member_custody_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        self.producer
            .advance_member_custody_retirement(request, original)
    }
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        use RelationalDurableOwnedReadRefusal as Refusal;
        let bytes = self
            .producer
            .read_member_owned(request, original, member, limit)?;
        if !self.changes(member) {
            return Ok(bytes);
        }
        let Some((&last, prefix)) = bytes.split_last() else {
            return Err(Refusal::Bounds);
        };
        let mut changed = ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
            .map_err(Refusal::Buffer)?;
        changed.extend_from_slice(prefix).map_err(Refusal::Buffer)?;
        changed.push(last ^ 1).map_err(Refusal::Buffer)?;
        changed.seal_immutable_bytes().map_err(Refusal::Buffer)
    }
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.producer.retain_members(members)
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let mut bytes = self.producer.read_member(member, limit)?;
        if self.changes(member) {
            let last = bytes.last_mut().ok_or(Cause::Damaged)?;
            *last ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.producer.read_commit(position, limit)
    }
}
