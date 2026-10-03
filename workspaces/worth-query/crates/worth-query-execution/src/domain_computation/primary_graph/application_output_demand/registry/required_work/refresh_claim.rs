use super::super::{DemandState, WorthQueryOutputAdvancement, WorthQueryOutputCheckpoint};
use super::selection::{charge_required_key_lookup, work_denial, SelectedReadyReadmission};
use super::*;

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
            .producer_commit_authority
            .as_ref()
            .expect("refresh claim is minted only from an executed Ready cell")
    }
}

impl WorthQueryOutputDemandRegistry {
    /// Retain the actual selected row through a fresh required continuation.
    /// This neither certifies currentness nor authorizes a new producer effect.
    pub(in crate::domain_computation::primary_graph) fn claim_required_ready_refresh(
        &self,
        selected: &SelectedReadyReadmission,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<SelectedRequiredRefreshClaim>, WorthQueryOutputDemandDenial> {
        let key = selected.membership.key.as_ref();
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
            return Ok(None);
        };
        let same_member = record.is_required()
            && record
                .work_membership
                .as_ref()
                .is_some_and(|member| Arc::ptr_eq(member, &selected.membership));
        let same_source = record
            .readmission_source
            .as_ref()
            .is_some_and(|source| Arc::ptr_eq(source, &selected.readmission));
        let same_ready = matches!(
            &record.state,
            DemandState::Output(output)
                if matches!(output.advancement, WorthQueryOutputAdvancement::Idle)
                    && matches!(&output.checkpoint,
                        Some(WorthQueryOutputCheckpoint::Ready(current))
                            if current.same_cell(&selected.completion))
        );
        if !same_member || !same_source || !same_ready || !state.required_keys.contains(key) {
            return Ok(None);
        }
        if selected.completion.producer_commit_authority.is_none() {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
                "required Ready output has no executed producer mode",
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
        let key_bytes = std::mem::size_of::<super::super::WorthQueryOutputDemandInterest>()
            .checked_add(key.producer.len())
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(key_bytes).map_err(|_| capacity_denial())?)
            .map_err(|_| capacity_denial())?;
        let capacity = state.reserve_required_custody_capacity(key_bytes)?;
        let interest_key = key.clone();
        let record = state
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
        drop(state);
        Ok(Some(SelectedRequiredRefreshClaim {
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
