use super::super::DemandState;
use super::super::WorthQueryOutputAdvancement;
use super::super::WorthQueryOutputCheckpoint;

use super::selection::SelectedReadyReadmission;
use super::*;

/// The first failed predicate at the actual refresh selection under the registry lock.
/// Descriptions only; these values retain no row, source or Ready custody.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RequiredRefreshRejection {
    MissingRow,
    NotRequired {
        interests: usize,
        required_interests: usize,
        framework_required_count: usize,
    },
    MembershipChanged,
    SourceChanged,
    NotOutput,
    NotIdle {
        advancement: RequiredRefreshAdvancement,
        checkpoint: RequiredRefreshCheckpoint,
    },
    MissingReady,
    ReadyCellChanged,
    RequiredKeyMissing,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RequiredRefreshAdvancement {
    Claimed {
        claim: u64,
    },
    Stopped {
        denial: WorthQueryOutputDemandDenialKind,
        recovery_posture:
            crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture,
        interrupted_claim: Option<u64>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RequiredRefreshCheckpoint {
    Absent,
    Published,
    Delivered,
    ReadyCommitted,
    ReadyStable,
    ReadyRestored,
}

/// A temporary real interest in the exact Ready row selected for a required
/// wave. The key's owned String remains funded until its interest drops.
pub(in crate::domain_computation::primary_graph) struct SelectedRequiredRefreshClaim {
    interest: super::super::WorthQueryOutputDemandInterest,

    selected: SelectedReadyReadmission,
}

impl SelectedRequiredRefreshClaim {
    /// Transfer the existing exact Ready/source/member pin to the admitted
    /// successor after its predecessor interest has served registration and
    /// replacement. The key ticket outlives the cloned interest key.
    pub(in crate::domain_computation::primary_graph) fn into_predecessor(
        self,
    ) -> SelectedReadyReadmission {
        let Self { interest, selected } = self;
        drop(interest);
        selected
    }

    pub(in crate::domain_computation::primary_graph) fn interest(
        &self,
    ) -> &super::super::WorthQueryOutputDemandInterest {
        &self.interest
    }

    pub(in crate::domain_computation::primary_graph) fn selected(
        &self,
    ) -> &SelectedReadyReadmission {
        &self.selected
    }

    pub(in crate::domain_computation::primary_graph) fn commit_authority(
        &self,
    ) -> &crate::domain_computation::primary_graph::application_contribution::WorthQueryProducerCommitAuthority{
        self.selected
            .completion
            .producer_mode()
            .expect("refresh claim is minted only from a Ready cell with a producer mode")
    }
}

impl WorthQueryOutputDemandRegistry {
    /// Retain the actual selected row through a fresh required continuation.
    /// This neither certifies currentness nor authorizes a new producer effect.
    pub(in crate::domain_computation::primary_graph) fn claim_required_ready_refresh(
        &self,
        selected: &SelectedReadyReadmission,
    ) -> Result<
        Result<SelectedRequiredRefreshClaim, RequiredRefreshRejection>,
        WorthQueryOutputDemandDenial,
    > {
        let key = selected.membership.key.as_ref();
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        let Some(record) = state.records.get(key) else {
            return Ok(Err(RequiredRefreshRejection::MissingRow));
        };
        // The wave selected this row as required: held, or awaited by an
        // owner that has yet to rejoin it. Its claim accepts the same rows.
        let required = if record.is_required() {
            true
        } else {
            let prepaid = &state.records;
            crate::domain_computation::primary_graph::application_output_demand::registry::refreshed_rejoin::awaited_by_stale_owner(prepaid, key)
        };
        if !required {
            return Ok(Err(RequiredRefreshRejection::NotRequired {
                interests: record.interests,
                required_interests: record.required_interests,
                framework_required_count: record.framework_required_count,
            }));
        }
        if !record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &selected.membership))
        {
            return Ok(Err(RequiredRefreshRejection::MembershipChanged));
        }
        if !record
            .readmission_source
            .as_ref()
            .is_some_and(|source| Arc::ptr_eq(source, &selected.readmission))
        {
            return Ok(Err(RequiredRefreshRejection::SourceChanged));
        }
        let DemandState::Output(output) = &record.state else {
            return Ok(Err(RequiredRefreshRejection::NotOutput));
        };
        let advancement = match output.advancement() {
            WorthQueryOutputAdvancement::Idle => None,
            WorthQueryOutputAdvancement::Claimed(claim) => {
                Some(RequiredRefreshAdvancement::Claimed { claim: claim.0 })
            }
            WorthQueryOutputAdvancement::Stopped {
                denial,
                interrupted_claim,
            } => Some(RequiredRefreshAdvancement::Stopped {
                denial: denial.kind(),
                recovery_posture: denial.recovery_posture(),
                interrupted_claim: interrupted_claim.map(|claim| claim.0),
            }),
        };
        if let Some(advancement) = advancement {
            use super::super::WorthQueryAcceptedOutputAuthority as Authority;
            let checkpoint = match &output.checkpoint {
                None => RequiredRefreshCheckpoint::Absent,
                Some(WorthQueryOutputCheckpoint::Published { .. }) => {
                    RequiredRefreshCheckpoint::Published
                }
                Some(WorthQueryOutputCheckpoint::Delivered { .. }) => {
                    RequiredRefreshCheckpoint::Delivered
                }
                Some(WorthQueryOutputCheckpoint::Ready(ready)) => match &ready.authority {
                    Authority::Committed(_) => RequiredRefreshCheckpoint::ReadyCommitted,
                    Authority::Stable(_) => RequiredRefreshCheckpoint::ReadyStable,
                    Authority::Restored(_) => RequiredRefreshCheckpoint::ReadyRestored,
                },
            };
            return Ok(Err(RequiredRefreshRejection::NotIdle {
                advancement,
                checkpoint,
            }));
        }
        let Some(WorthQueryOutputCheckpoint::Ready(current)) = &output.checkpoint else {
            return Ok(Err(RequiredRefreshRejection::MissingReady));
        };
        if !current.same_cell(&selected.completion) {
            return Ok(Err(RequiredRefreshRejection::ReadyCellChanged));
        }
        if !state.required_keys.contains(key) {
            return Ok(Err(RequiredRefreshRejection::RequiredKeyMissing));
        }
        if selected.completion.producer_mode().is_none() {
            // A restored output no advance has reached, met by queued work
            // for another owner: it has no mode to execute its producer in
            // until its own demand or a dependent's chain advances it.
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "restored required output awaits the first advance that reaches it",
            )
            .with_recovery_posture(
                crate::domain_computation::primary_graph::WorthQueryOutputDemandRecoveryPosture::Retryable,
            ));
        }
        // The claim shares all three exact pins only after the current Ready,
        // readmission and membership have been revalidated under this guard.

        let _ = record
            .interests
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        let _ = record
            .required_interests
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        // The mutable row selection below is a second ordered descent after
        // the first immutable Ready validation.

        let interest_key = key.clone();
        let mut record = state
            .records
            .get_mut(key)
            .expect("selected Ready row retained");
        record.interests += 1;
        record.required_interests += 1;
        let interest = super::super::WorthQueryOutputDemandInterest {
            key: interest_key,
            requires_output: true,
            notifications: super::super::WorthQueryOutputDemandNotifications {
                wake: Arc::clone(&record.wake),
            },
            owner: self.clone(),
        };
        record.finish();
        drop(state);
        Ok(Ok(SelectedRequiredRefreshClaim {
            interest,

            selected: SelectedReadyReadmission {
                completion: selected.completion.clone(),
                readmission: Arc::clone(&selected.readmission),
                membership: Arc::clone(&selected.membership),
            },
        }))
    }
}

fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "selected Ready interest exceeds required registry custody",
    )
}
