//! A wave handoff holds the posted consumer; a released row yields no custody.

use super::*;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

pub(in crate::domain_computation::primary_graph) enum ConsumerCustody {
    Handoff(ConsumerHandoff),
    Released,
}
pub(in crate::domain_computation::primary_graph) struct ConsumerHandoff {
    identity: Arc<RecordedSettlementIdentity>,
    _interest: WorthQueryOutputDemandInterest,
}
impl ConsumerCustody {
    pub(in crate::domain_computation::primary_graph) fn matches(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> bool {
        match self {
            Self::Handoff(handoff) => handoff.identity == *identity,
            Self::Released => false,
        }
    }
}
impl WorthQueryOutputDemandRegistry {
    pub(in crate::domain_computation::primary_graph) fn retain_consumer_handoff(
        &self,
        identity: &Arc<RecordedSettlementIdentity>,
    ) -> Result<ConsumerCustody, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(key) = state.settlement_keys.get_exact_admitted(identity)? else {
            return Ok(ConsumerCustody::Released);
        };

        let Some(record) = state.records.get(key.as_ref()) else {
            return Ok(ConsumerCustody::Released);
        };
        let interests = record
            .interests
            .checked_add(1)
            .ok_or_else(interest_count_overflow)?;
        let required = record
            .required_interests
            .checked_add(1)
            .ok_or_else(interest_count_overflow)?;

        let member = state.prepare_required_member(key.as_ref())?;
        let interest_key = key.as_ref().clone();
        let mut record = state
            .records
            .get_mut(key.as_ref())
            .expect("posted consumer remains retained");
        record.interests = interests;
        record.required_interests = required;
        // The wave holds this row through its real interest. It must not add
        // an upstream claim: that would prevent replacement from retiring the
        // predecessor's postings and leave its dependents on obsolete rows.
        let interest = WorthQueryOutputDemandInterest {
            key: interest_key,
            requires_output: true,
            notifications: WorthQueryOutputDemandNotifications {
                wake: Arc::clone(&record.wake),
            },
            owner: self.clone(),
        };
        record.finish();
        state.install_required_member(member);
        drop(state);
        Ok(ConsumerCustody::Handoff(ConsumerHandoff {
            identity: Arc::clone(identity),
            _interest: interest,
        }))
    }
}

fn interest_count_overflow() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "consumer interest count is not representable",
    )
}
