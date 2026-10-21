//! A supersession request pays its member search before installing the stop.

use super::*;

impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn finish_superseded(
        &self,
        interest: &WorthQueryOutputDemandInterest,
        subject: &str,
    ) -> Result<WorthQueryOutputDemandDenial, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let (denial, claims, member) = {
            let priced: &mut DemandRegistryState = &mut state;
            {
                let (denial, claims) = priced.finish_superseded_row(&interest.key, subject);
                let member = priced.remove_required_member_if_released_detached(&interest.key);
                (denial, claims, member)
            }
        };
        drop(state);
        drop(claims);
        drop(member);
        Ok(denial)
    }
}
