//! A supersession request pays its member search before installing the stop.
use super::super::refreshed_rejoin::{
    price_member_release, price_mutation, PricedMutation, WalkKind,
};
use super::*;
use crate::domain_computation::primary_graph::InvalidationEditAdmission;

pub(super) fn price_member<'a>(
    state: &'a mut DemandRegistryState,
    key: &WorthQueryOutputDemandKey,
    admission: &mut InvalidationEditAdmission,
) -> Result<PricedMutation<'a>, WorthQueryOutputDemandDenial> {
    let row = &state.records[key];
    // Installing the stop releases obligations, but these holds do not change.
    // A held member returns before the occurrence search, so it needs no walk.
    if row.interests != 0 || row.required_interests != 0 || row.framework_required_count != 0 {
        price_mutation(state, admission, &[], WalkKind::RequiredMember)
    } else {
        price_member_release(state, admission, key)
    }
}

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_superseded(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        subject: &str,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (denial, claims, member) = price_member(&mut state, &interest.key, admission)?
            .complete(|priced| {
                let (denial, claims) = priced
                    .state_mut()
                    .finish_superseded_row(&interest.key, subject);
                let member = priced.release_member(&interest.key);
                Ok((denial, claims, member))
            });
        let refund = member.refund_required_bytes;
        drop(state);
        drop(claims);
        drop(member);
        if refund != 0 {
            let mut state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.required_reserved_bytes = state.required_reserved_bytes.saturating_sub(refund);
        }
        Ok(denial)
    }
}
