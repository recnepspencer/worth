use super::super::{DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint};
use super::selection::{charge_required_key_lookup, work_denial, SelectedReadyReadmission};
use super::*;

/// The first failed predicate at the actual refresh selection under the registry lock.
/// Descriptions only; these values retain no row, source or Ready custody.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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
    NotIdle,
    MissingReady,
    ReadyCellChanged,
    RequiredKeyMissing,
}

/// A temporary real interest in the exact Ready row selected for a required
/// wave. The key's owned String remains funded until its interest drops.
pub(in crate::domain_computation::primary_graph) struct SelectedRequiredRefreshClaim {
    interest: super::super::WorthQueryOutputDemandInterest,
    _key_capacity: super::super::required_custody::RequiredOutputCustodyCapacity,
    selected: SelectedReadyReadmission,
}

impl SelectedRequiredRefreshClaim {
    /// Transfer the existing exact Ready/source/member pin to the admitted
    /// successor after its predecessor interest has served registration and
    /// replacement. The key ticket outlives the cloned interest key.
    pub(in crate::domain_computation::primary_graph) fn into_predecessor(
        self,
    ) -> SelectedReadyReadmission {
        let Self {
            interest,
            _key_capacity,
            selected,
        } = self;
        drop(interest);
        drop(_key_capacity);
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Result<SelectedRequiredRefreshClaim, RequiredRefreshRejection>,
        WorthQueryOutputDemandDenial,
    > {
        let key = selected.membership.key.as_ref();
        let key_bytes = std::mem::size_of::<super::super::WorthQueryOutputDemandInterest>()
            .checked_add(key.producer.len())
            .ok_or_else(capacity_denial)?;
        // The refresh retains one more Ready. A producer is not contacted for
        // an output custody cannot keep; the row stays Ready for a later claim.
        let kept = key_bytes
            .checked_add(super::super::ready_backing::PreparedReadyBacking::retained_bytes())
            .ok_or_else(capacity_denial)?;
        self.reclaim_cached_rows(kept, admission)?;
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.charge_record_lookup(key, admission)?;
        charge_required_key_lookup(&state, key, admission)?;
        let copy_work = key.producer.len().checked_add(9).ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(copy_work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        let Some(record) = state.records.get(key) else {
            return Ok(Err(RequiredRefreshRejection::MissingRow));
        };
        // The wave selected this row as required: held, or awaited by an
        // owner that has yet to rejoin it. Its claim accepts the same rows.
        let required = if record.is_required() {
            true
        } else {
            let mut prepaid = super::super::refreshed_rejoin::charge_whole_map(
                &state.records,
                admission,
                super::super::refreshed_rejoin::WholeMapPasses::One,
                0,
            )
            .map_err(|_| work_denial())?;
            prepaid.stale_owner(key)?
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
        if !matches!(output.advancement(), WorthQueryOutputAdvancement::Idle) {
            return Ok(Err(RequiredRefreshRejection::NotIdle));
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
        admission
            .charge_external_work(3)
            .map_err(|_| work_denial())?;
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
        state.charge_record_lookup(key, admission)?;
        let refreshed = state
            .required_reserved_bytes
            .checked_add(kept)
            .ok_or_else(capacity_denial)?;
        if !state.has_required_capacity(refreshed) {
            return Err(super::super::required_custody::full_custody_denial(
                kept,
                state.required_budget_bytes,
            ));
        }
        admission
            .admit_read_scratch(u64::try_from(key_bytes).map_err(|_| capacity_denial())?)
            .map_err(|_| capacity_denial())?;
        let capacity = state.reserve_required_custody_capacity(key_bytes)?;
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
            _key_capacity: capacity,
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
