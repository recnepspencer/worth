use std::collections::VecDeque;

use worth_proof::NonEmpty;

use super::{
    PhysicalWalAppendFailureCause, PhysicalWalAppendPort, PhysicalWalGroupMemberAppendOutcome,
};
use crate::physical_runtime::durability::{
    PhysicalDurabilityGroupSealingFailure, PhysicalMutationIdempotencyGroupSealDenial,
    ReservedPhysicalWalGroupMembers,
};
use crate::physical_runtime::{
    AdmittedPhysicalDurabilityGroupMember, PhysicalDurabilityGroupBasis,
    PhysicalDurabilityGroupSealingDenial, PhysicalWalReservationDenial, PreparedPhysicalMutation,
    RejectedPhysicalDurabilityGroup, SealedPhysicalDurabilityGroupMembers,
    WalAppendedPhysicalMutation, WalBarrierMember, WalRangeReservedPhysicalMutation,
};

pub enum PhysicalWalGroupAppendOutcome {
    Appended(SealedPhysicalDurabilityGroupMembers),
    NotAdmitted {
        members: NonEmpty<PreparedPhysicalMutation>,
        cause: PhysicalWalGroupAppendFailureCause,
    },
    AdmissionRejected(RejectedPhysicalDurabilityGroup),
    NotStarted(PhysicalWalGroupAppendContinuation),
    PartiallyAppended(PhysicalWalGroupAppendContinuation),
    Indeterminate(IndeterminatePhysicalWalGroupAppend),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PhysicalWalGroupAppendFailureCause {
    RuntimeReleased,
    SignalClockUnavailable,
    Reservation(PhysicalWalReservationDenial),
    PublicationGrowth,
    Append(PhysicalWalAppendFailureCause),
}

pub struct PhysicalWalGroupAppendContinuation {
    basis: PhysicalDurabilityGroupBasis,
    appended: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
    remaining: PhysicalWalGroupAppendRemainder,
    cause: PhysicalWalGroupAppendFailureCause,
}

enum PhysicalWalGroupAppendRemainder {
    Admitted(NonEmpty<AdmittedPhysicalDurabilityGroupMember>),
    Reserved(NonEmpty<WalBarrierMember<WalRangeReservedPhysicalMutation>>),
}

pub struct IndeterminatePhysicalWalGroupAppend {
    basis: PhysicalDurabilityGroupBasis,
    state: IndeterminatePhysicalWalGroupAppendState,
}

enum IndeterminatePhysicalWalGroupAppendState {
    Member {
        appended: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
        uncertain: WalBarrierMember<WalRangeReservedPhysicalMutation>,
        remaining: Vec<WalBarrierMember<WalRangeReservedPhysicalMutation>>,
    },
    Sealing {
        appended: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
        cause: PhysicalDurabilityGroupSealingDenial,
    },
}

impl PhysicalWalAppendPort {
    pub(in crate::physical_runtime) fn append_prepared_group(
        &self,
        members: NonEmpty<PreparedPhysicalMutation>,
    ) -> PhysicalWalGroupAppendOutcome {
        let Some(runtime) = self.runtime.upgrade() else {
            return PhysicalWalGroupAppendOutcome::NotAdmitted {
                members,
                cause: PhysicalWalGroupAppendFailureCause::RuntimeReleased,
            };
        };
        let clock = match runtime.signal.clock_observation() {
            Ok(clock) => clock,
            Err(_) => {
                return PhysicalWalGroupAppendOutcome::NotAdmitted {
                    members,
                    cause: PhysicalWalGroupAppendFailureCause::SignalClockUnavailable,
                }
            }
        };
        let byte_limit = self
            .scheduler
            .capacity_snapshot()
            .configured()
            .bandwidth_tokens();
        let admitted =
            match self
                .grouping
                .admit(members, self.durability, clock.current_tick(), byte_limit)
            {
                Ok(admitted) => admitted,
                Err(rejected) => return PhysicalWalGroupAppendOutcome::AdmissionRejected(rejected),
            };
        let sealing_bindings = admitted.idempotency_sealing_bindings();
        let (basis, members) = admitted.into_parts();
        self.reserve_charge_and_append(basis, Vec::new(), members, &sealing_bindings)
    }

    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(in crate::physical_runtime) fn continue_prepared_group(
        &self,
        continuation: PhysicalWalGroupAppendContinuation,
    ) -> PhysicalWalGroupAppendOutcome {
        let PhysicalWalGroupAppendContinuation {
            basis,
            appended,
            remaining,
            ..
        } = continuation;
        match remaining {
            PhysicalWalGroupAppendRemainder::Admitted(members) => {
                self.reserve_charge_and_append(basis, appended, members, &[])
            }
            PhysicalWalGroupAppendRemainder::Reserved(members) => {
                self.append_reserved_group(basis, appended, members)
            }
        }
    }

    fn reserve_charge_and_append(
        &self,
        basis: PhysicalDurabilityGroupBasis,
        appended: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
        members: NonEmpty<AdmittedPhysicalDurabilityGroupMember>,
        sealing_bindings: &[crate::physical_runtime::durability::PhysicalMutationGroupSealingBinding],
    ) -> PhysicalWalGroupAppendOutcome {
        let sealing_bindings = if sealing_bindings.is_empty() {
            members
                .as_slice()
                .iter()
                .map(|member| {
                    crate::physical_runtime::durability::PhysicalMutationGroupSealingBinding::new(
                        crate::physical_runtime::durability::PhysicalMutationUnresolvedBindingObservation::new(
                            member.prepared.idempotency_identity(),
                            member.prepared.request_fingerprint(),
                            member.prepared.mutation_identity(),
                        ),
                        member.binding(),
                    )
                })
                .collect::<Vec<_>>()
        } else {
            sealing_bindings.to_vec()
        };
        match self.owner.reserve_group(members) {
            Ok(reserved) => self.charge_and_append(basis, appended, reserved, &sealing_bindings),
            Err((members, cause)) => continuation_outcome(
                basis,
                appended,
                PhysicalWalGroupAppendRemainder::Admitted(members),
                PhysicalWalGroupAppendFailureCause::Reservation(cause),
            ),
        }
    }

    fn charge_and_append(
        &self,
        basis: PhysicalDurabilityGroupBasis,
        appended: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
        reserved: ReservedPhysicalWalGroupMembers,
        sealing_bindings: &[crate::physical_runtime::durability::PhysicalMutationGroupSealingBinding],
    ) -> PhysicalWalGroupAppendOutcome {
        let growth = reserved.retained_publication_bytes();
        let (segment, generation) = reserved.publication_segment();
        let charged = self
            .publication
            .get()
            .and_then(|admission| admission.reserve_retained_bytes(growth).ok());
        if growth > 0 && charged.is_none() {
            return self.growth_denied(reserved);
        }
        if !sealing_bindings.is_empty() {
            if let Err(denial) = self.idempotency.seal_group(sealing_bindings) {
                if !matches!(
                    denial,
                    PhysicalMutationIdempotencyGroupSealDenial::AlreadyGroupSealed
                ) {
                    drop(charged);
                    return PhysicalWalGroupAppendOutcome::AdmissionRejected(
                        RejectedPhysicalDurabilityGroup::before_effect(
                            self.release_before_effect(reserved),
                            denial.admission_denial(),
                        ),
                    );
                }
            }
        }
        let outcome = self.append_reserved_group(basis, appended, reserved.into_members());
        if let Some(lease) = charged {
            if !matches!(outcome, PhysicalWalGroupAppendOutcome::NotStarted(_)) {
                lease.seal();
                self.owner
                    .note_sealed_publication(segment, generation, growth);
            }
        }
        outcome
    }

    fn growth_denied(
        &self,
        reserved: ReservedPhysicalWalGroupMembers,
    ) -> PhysicalWalGroupAppendOutcome {
        PhysicalWalGroupAppendOutcome::NotAdmitted {
            members: self.release_before_effect(reserved),
            cause: PhysicalWalGroupAppendFailureCause::PublicationGrowth,
        }
    }

    fn release_before_effect(
        &self,
        reserved: ReservedPhysicalWalGroupMembers,
    ) -> NonEmpty<PreparedPhysicalMutation> {
        let prepared = reserved
            .release_after_no_effect()
            .into_vec()
            .into_iter()
            .map(|member| member.into_parts().0)
            .collect();
        self.owner.release_group_before_effect();
        nonempty(prepared)
    }

    fn append_reserved_group(
        &self,
        basis: PhysicalDurabilityGroupBasis,
        mut appended: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
        members: NonEmpty<WalBarrierMember<WalRangeReservedPhysicalMutation>>,
    ) -> PhysicalWalGroupAppendOutcome {
        let mut remaining = VecDeque::from(members.into_vec());
        while let Some(member) = remaining.pop_front() {
            match self.append_group_member(member) {
                PhysicalWalGroupMemberAppendOutcome::Appended(member) => appended.push(member),
                PhysicalWalGroupMemberAppendOutcome::NotStarted { member, cause } => {
                    let mut pending = Vec::with_capacity(1 + remaining.len());
                    pending.push(member);
                    pending.extend(remaining);
                    let pending = nonempty(pending);
                    let remainder = if appended.is_empty() {
                        self.owner.release_group_before_effect();
                        PhysicalWalGroupAppendRemainder::Admitted(
                            ReservedPhysicalWalGroupMembers::from_members(pending)
                                .release_after_no_effect(),
                        )
                    } else {
                        PhysicalWalGroupAppendRemainder::Reserved(pending)
                    };
                    return continuation_outcome(
                        basis,
                        appended,
                        remainder,
                        PhysicalWalGroupAppendFailureCause::Append(cause),
                    );
                }
                PhysicalWalGroupMemberAppendOutcome::Indeterminate(uncertain) => {
                    return PhysicalWalGroupAppendOutcome::Indeterminate(
                        IndeterminatePhysicalWalGroupAppend {
                            basis,
                            state: IndeterminatePhysicalWalGroupAppendState::Member {
                                appended,
                                uncertain,
                                remaining: remaining.into(),
                            },
                        },
                    );
                }
            }
        }
        self.owner.finish_group();
        match SealedPhysicalDurabilityGroupMembers::seal(basis, appended) {
            Ok(sealed) => PhysicalWalGroupAppendOutcome::Appended(sealed),
            Err(failure) => sealing_indeterminate(basis, failure),
        }
    }
}

fn sealing_indeterminate(
    basis: PhysicalDurabilityGroupBasis,
    failure: PhysicalDurabilityGroupSealingFailure,
) -> PhysicalWalGroupAppendOutcome {
    let (appended, cause) = failure.into_parts();
    PhysicalWalGroupAppendOutcome::Indeterminate(IndeterminatePhysicalWalGroupAppend {
        basis,
        state: IndeterminatePhysicalWalGroupAppendState::Sealing { appended, cause },
    })
}

fn continuation_outcome(
    basis: PhysicalDurabilityGroupBasis,
    appended: Vec<WalBarrierMember<WalAppendedPhysicalMutation>>,
    remaining: PhysicalWalGroupAppendRemainder,
    cause: PhysicalWalGroupAppendFailureCause,
) -> PhysicalWalGroupAppendOutcome {
    let continuation = PhysicalWalGroupAppendContinuation {
        basis,
        appended,
        remaining,
        cause,
    };
    if continuation.appended.is_empty() {
        PhysicalWalGroupAppendOutcome::NotStarted(continuation)
    } else {
        PhysicalWalGroupAppendOutcome::PartiallyAppended(continuation)
    }
}

fn nonempty<T>(members: Vec<T>) -> NonEmpty<T> {
    NonEmpty::try_from_vec(members).unwrap_or_else(|_| {
        unreachable!("a failed current member keeps continuation membership nonempty")
    })
}

impl PhysicalWalGroupAppendContinuation {
    #[cfg_attr(not(feature = "certification-test-authority"), allow(dead_code))]
    pub(in crate::physical_runtime) fn runtime_released(mut self) -> PhysicalWalGroupAppendOutcome {
        self.cause = PhysicalWalGroupAppendFailureCause::RuntimeReleased;
        if self.appended.is_empty() {
            PhysicalWalGroupAppendOutcome::NotStarted(self)
        } else {
            PhysicalWalGroupAppendOutcome::PartiallyAppended(self)
        }
    }

    pub const fn basis(&self) -> PhysicalDurabilityGroupBasis {
        self.basis
    }

    pub const fn cause(&self) -> &PhysicalWalGroupAppendFailureCause {
        &self.cause
    }

    pub fn appended_member_count(&self) -> usize {
        self.appended.len()
    }

    pub fn remaining_member_count(&self) -> usize {
        match &self.remaining {
            PhysicalWalGroupAppendRemainder::Admitted(members) => members.len(),
            PhysicalWalGroupAppendRemainder::Reserved(members) => members.len(),
        }
    }
}

impl IndeterminatePhysicalWalGroupAppend {
    pub const fn basis(&self) -> PhysicalDurabilityGroupBasis {
        self.basis
    }

    pub fn appended_member_count(&self) -> usize {
        match &self.state {
            IndeterminatePhysicalWalGroupAppendState::Member { appended, .. } => appended.len(),
            IndeterminatePhysicalWalGroupAppendState::Sealing { appended, .. } => appended.len(),
        }
    }

    pub const fn sealing_denial(&self) -> Option<PhysicalDurabilityGroupSealingDenial> {
        match &self.state {
            IndeterminatePhysicalWalGroupAppendState::Sealing { cause, .. } => Some(*cause),
            IndeterminatePhysicalWalGroupAppendState::Member { .. } => None,
        }
    }

    pub fn uncertain_member(&self) -> Option<&WalBarrierMember<WalRangeReservedPhysicalMutation>> {
        match &self.state {
            IndeterminatePhysicalWalGroupAppendState::Member { uncertain, .. } => Some(uncertain),
            IndeterminatePhysicalWalGroupAppendState::Sealing { .. } => None,
        }
    }

    pub fn unstarted_member_count(&self) -> usize {
        match &self.state {
            IndeterminatePhysicalWalGroupAppendState::Member { remaining, .. } => remaining.len(),
            IndeterminatePhysicalWalGroupAppendState::Sealing { .. } => 0,
        }
    }
}
