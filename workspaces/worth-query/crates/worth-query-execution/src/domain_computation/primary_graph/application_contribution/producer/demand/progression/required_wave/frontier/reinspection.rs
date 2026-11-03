//! The caller has no reinspection transition after a refused publication.
use super::super::performed::ReinspectionCredit;
use super::*;
pub(in super::super) struct QueuedMember<'frontier, Ready> {
    member: &'frontier mut RequiredMember<Ready>,
}
impl<Ready> RequiredFrontier<Ready> {
    pub(in super::super) fn queued_member(
        &mut self,
        index: usize,
    ) -> Option<QueuedMember<'_, Ready>> {
        let member = &mut self.members[index];
        (!member.caller).then_some(QueuedMember { member })
    }
}
impl<Ready> QueuedMember<'_, Ready> {
    pub(in super::super) fn refused(self, credit: Option<ReinspectionCredit>) -> bool {
        let Some(_credit) = credit else {
            self.member.state = MemberState::Pending;
            return false;
        };
        self.member.ready.clear_claim_binding();
        self.member.state = MemberState::Inspect;
        true
    }
}
