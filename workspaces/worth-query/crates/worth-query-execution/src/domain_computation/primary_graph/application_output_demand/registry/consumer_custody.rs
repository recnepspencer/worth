//! A wave handoff holds the posted consumer; a released row yields no custody.
use super::required_custody::RequiredOutputCustodyCapacity;
use super::*;
use crate::domain_computation::primary_graph::output_lineage::{
    invalidation::InvalidationEditAdmission, RecordedSettlementIdentity,
};
use crate::domain_computation::primary_graph::WorthQueryOutputDemandDenialKind;

pub(in crate::domain_computation::primary_graph) enum ConsumerCustody {
    Handoff(ConsumerHandoff),
    Released,
}
pub(in crate::domain_computation::primary_graph) struct ConsumerHandoff {
    identity: Arc<RecordedSettlementIdentity>,
    _interest: WorthQueryOutputDemandInterest,
    _capacity: RequiredOutputCustodyCapacity,
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
        admission: &mut InvalidationEditAdmission,
    ) -> Result<ConsumerCustody, WorthQueryOutputDemandDenial> {
        let mut state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let Some(key) = state
            .settlement_keys
            .get_exact_admitted(identity, admission)?
        else {
            return Ok(ConsumerCustody::Released);
        };
        state.charge_record_lookup(&key, admission)?;
        let Some(record) = state.records.get(key.as_ref()) else {
            return Ok(ConsumerCustody::Released);
        };
        let interests = record
            .interests
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        let required = record
            .required_interests
            .checked_add(1)
            .ok_or_else(capacity_denial)?;
        let bytes = ConsumerHandoff::retained_bytes(&key.producer).ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(bytes as u64)
            .map_err(|_| capacity_denial())?;
        admission
            .charge_external_work(bytes as u64 + 3)
            .map_err(|_| work_denial())?;
        let capacity = state.reserve_required_custody_capacity(bytes)?;
        let member = state.prepare_required_member(key.as_ref())?;
        let interest_key = key.as_ref().clone();
        let record = state
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
        state.install_required_member(member);
        drop(state);
        Ok(ConsumerCustody::Handoff(ConsumerHandoff {
            identity: Arc::clone(identity),
            _interest: interest,
            _capacity: capacity,
        }))
    }
}
fn capacity_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::RetentionBudgetExceeded,
        "consumer handoff custody",
    )
}
fn work_denial() -> WorthQueryOutputDemandDenial {
    WorthQueryOutputDemandDenial::new(
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        "consumer handoff custody",
    )
}

impl ConsumerHandoff {
    pub(super) fn retained_bytes(producer: &str) -> Option<usize> {
        std::mem::size_of::<WorthQueryOutputDemandInterest>().checked_add(producer.len())
    }
}
