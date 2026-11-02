//! Fault the actual held selected-read port without replacing its snapshot or issuer.
use super::{OwnedNativeReadSnapshot, TestDurableMedium};
use std::collections::BTreeMap;
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_relational::facade::durability::{
    RelationalDurableMemberLocator as Locator, RelationalDurableOwnedReadRefusal as Refusal,
};
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;

#[derive(Debug, Clone, Copy)]
enum MemberFault {
    Refuse(Cause),
    CorruptLastByte,
}
#[derive(Debug, Default)]
pub(super) struct MemberFaultState {
    faults: BTreeMap<Locator, MemberFault>,
    last: Option<Locator>,
}
impl TestDurableMedium {
    pub fn refuse_member_read_for_test(&self, member: Locator, cause: Cause) {
        let previous = self
            .member_faults
            .lock()
            .unwrap()
            .faults
            .insert(member, MemberFault::Refuse(cause));
        assert!(previous.is_none(), "one original fault per held member");
    }
    pub fn corrupt_member_read_for_test(&self, member: Locator) {
        let previous = self
            .member_faults
            .lock()
            .unwrap()
            .faults
            .insert(member, MemberFault::CorruptLastByte);
        assert!(previous.is_none(), "one original fault per held member");
    }
    pub fn last_member_fault_for_test(&self) -> Option<Locator> {
        self.member_faults.lock().unwrap().last
    }
    pub(super) fn read_original_member(
        &self,
        request: &ExecutionRequest<'_, '_>,
        snapshot: &OwnedNativeReadSnapshot,
        member: Locator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, Refusal> {
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        if member.length > limit {
            return Err(Refusal::Storage(Cause::LimitExceeded {
                length: member.length,
                limit,
            }));
        }
        let fault = {
            let mut state = self.member_faults.lock().unwrap();
            let fault = state.faults.get(&member).copied();
            if fault.is_some() {
                state.last = Some(member);
            }
            fault
        };
        if let Some(MemberFault::Refuse(cause)) = fault {
            return Err(Refusal::Storage(cause));
        }
        let bytes = snapshot
            .member(request, member.sha256, member.length, limit)
            .map_err(super::cold_source::owned_failure)?;
        match fault {
            None => Ok(bytes),
            Some(MemberFault::CorruptLastByte) => {
                let (last, prefix) = bytes.split_last().ok_or(Refusal::Storage(Cause::Damaged))?;
                let mut changed = ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
                    .map_err(Refusal::Buffer)?;
                changed.extend_from_slice(prefix).map_err(Refusal::Buffer)?;
                changed.push(*last ^ 0xff).map_err(Refusal::Buffer)?;
                changed.seal_immutable_bytes().map_err(Refusal::Buffer)
            }
            Some(MemberFault::Refuse(cause)) => Err(Refusal::Storage(cause)),
        }
    }
}
