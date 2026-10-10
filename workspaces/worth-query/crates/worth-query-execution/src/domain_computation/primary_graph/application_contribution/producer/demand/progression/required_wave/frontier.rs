//! Required members and their actual unresolved consumed-output edges.
use super::*;
mod reinspection;
mod selection;
pub(super) use selection::MemberInput;

pub(super) enum MemberState {
    Inspect,
    Waiting(usize),
    Prepared,
    // Only the caller keeps its return report. Upstream reports live in the
    // basis-bound resolved set and must die when that basis moves.
    Current(Option<Arc<WorthQueryOutputDemandSettlement>>),
    /// Its outcome belongs to this advance; a queue closure cannot reopen it.
    AcceptedCaller,
    Pending,
    Held(WorthQueryOutputDemandKey),
}

impl MemberState {
    /// A decided caller needs no scheduling work; native certification alone
    /// decides whether its output satisfies a queued member.
    pub fn is_done(&self) -> bool {
        matches!(self, Self::Current(_) | Self::AcceptedCaller)
    }
}

pub(super) enum RequiredUpstream {
    Member(usize),
    HeldCaller,
}

/// Scheduling posture only; each basis still requires native certification.
pub(super) enum DependencySchedule {
    OwnSelection,
    PendingConsumed,
}

pub(super) struct RequiredMember<Ready = SelectedReadyReadmission> {
    pub ready: MemberInput<Ready>,
    pub caller: bool,
    pub caller_successor: bool,
    pub resolved_successor: bool,
    pub queued: Option<usize>,
    pub contacts: usize,
    pub state: MemberState,
    pub dependency_schedule: DependencySchedule,
}

pub(super) struct RequiredFrontier<Ready = SelectedReadyReadmission> {
    pub members: Vec<RequiredMember<Ready>>,
}

impl RequiredFrontier {
    pub fn new() -> Self {
        Self {
            members: Vec::new(),
        }
    }

    pub fn insert(
        &mut self,
        ready: SelectedReadyReadmission,
        caller: bool,
        queued: Option<usize>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RequiredUpstream, WorthQueryOutputDemandDenial> {
        for (index, member) in self.members.iter_mut().enumerate() {
            let same = match &member.ready {
                MemberInput::Selected(current) => ready.same_record(current, admission)?,
                MemberInput::Queued { claim, .. } => claim.same_record(&ready, admission)?,
            };
            if same {
                if let MemberState::AcceptedCaller = member.state {
                    return Ok(RequiredUpstream::HeldCaller);
                }
                // Membership identifies the physical row, not its current
                // readmission binding. Keep the exact pin the owner just issued.
                if matches!(&member.ready, MemberInput::Selected(current) if !ready.same_ready_cell(current, admission)?)
                {
                    member.ready.rebind(ready);
                    member.state = MemberState::Inspect;
                }
                member.caller |= caller;
                if queued.is_some() {
                    member.queued = queued;
                    member.state = MemberState::Inspect;
                }
                return Ok(RequiredUpstream::Member(index));
            }
        }
        self.reserve_slot(admission)?;
        let index = self.members.len();
        self.members.push(RequiredMember {
            ready: MemberInput::Selected(ready),
            caller,
            queued,
            caller_successor: false,
            resolved_successor: false,
            contacts: 0,
            state: MemberState::Inspect,
            dependency_schedule: DependencySchedule::OwnSelection,
        });
        Ok(RequiredUpstream::Member(index))
    }

    fn reserve_slot(
        &mut self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let item = std::mem::size_of::<RequiredMember>();
        if self.members.len() == self.members.capacity() {
            let next = self
                .members
                .capacity()
                .checked_mul(2)
                .ok_or_else(capacity_denial)?
                .max(1);
            let old_bytes = self
                .members
                .capacity()
                .checked_mul(item)
                .ok_or_else(capacity_denial)?;
            let new_bytes = next.checked_mul(item).ok_or_else(capacity_denial)?;
            admission
                .admit_read_scratch(
                    u64::try_from(
                        old_bytes
                            .checked_add(new_bytes)
                            .ok_or_else(capacity_denial)?,
                    )
                    .map_err(|_| capacity_denial())?,
                )
                .map_err(admission_denial)?;
            admission
                .charge_external_work(
                    u64::try_from(
                        self.members
                            .len()
                            .checked_mul(item)
                            .ok_or_else(work_denial)?,
                    )
                    .map_err(|_| work_denial())?,
                )
                .map_err(|_| work_denial())?;
            let mut replacement = Vec::new();
            replacement
                .try_reserve_exact(next)
                .map_err(|_| capacity_denial())?;
            if replacement.capacity() != next {
                return Err(capacity_denial());
            }
            replacement.append(&mut self.members);
            self.members = replacement;
        }
        admission
            .charge_external_work(u64::try_from(item + 1).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        Ok(())
    }

    /// An issued prerequisite or observed pending edge can defer a member. A cycle
    /// stays pending; no stack position or level grants readiness.
    pub fn require_upstream(
        &mut self,
        ready: SelectedReadyReadmission,
        caller: bool,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<RequiredUpstream, WorthQueryOutputDemandDenial> {
        let RequiredUpstream::Member(index) = self.insert(ready, caller, None, admission)? else {
            return Ok(RequiredUpstream::HeldCaller);
        };
        // An actual unresolved consumed edge asks for a new certification.
        // An earlier scheduling result cannot answer that owner-issued edge.
        if matches!(self.members[index].state, MemberState::Current(_)) {
            self.members[index].state = MemberState::Inspect;
        }
        Ok(RequiredUpstream::Member(index))
    }
}

impl<Ready> RequiredFrontier<Ready> {
    pub fn wait_for(&mut self, member: usize, upstream: usize) -> bool {
        let mut cursor = upstream;
        for _ in 0..=self.members.len() {
            if cursor == member {
                self.members[member].state = MemberState::Pending;
                return false;
            }
            match self.members[cursor].state {
                MemberState::Waiting(next) => cursor = next,
                _ => {
                    self.members[member].state = MemberState::Waiting(upstream);
                    return true;
                }
            }
        }
        self.members[member].state = MemberState::Pending;
        false
    }

    pub fn awaken(&mut self) -> bool {
        let mut changed = false;
        for index in 0..self.members.len() {
            let MemberState::Waiting(upstream) = self.members[index].state else {
                continue;
            };
            if matches!(self.members[upstream].state, MemberState::Current(_)) {
                self.members[index].state = MemberState::Inspect;
                changed = true;
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests;
