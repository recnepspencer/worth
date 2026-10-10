//! The semantic order of required producer/source occurrences.
use super::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch;
use std::cmp::Ordering;

pub(super) struct RequiredMemberKey {
    producer: String,
    source: WorthQueryObservedSourceEpoch,
    pub index: usize,
}
impl RequiredMemberKey {
    pub fn comparison_work(&self, other: &Self) -> Result<u64, WorthQueryOutputDemandDenial> {
        let work = self
            .producer
            .len()
            .min(other.producer.len())
            .checked_add(8)
            .ok_or_else(work_denial)?;
        u64::try_from(work).map_err(|_| work_denial())
    }

    pub fn retain(
        index: usize,
        ready: &SelectedReadyReadmission,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let producer = ready.producer_identity();
        let bytes = producer
            .len()
            .checked_add(std::mem::size_of::<Self>())
            .ok_or_else(capacity_denial)?;
        admission
            .admit_read_scratch(u64::try_from(bytes).map_err(|_| capacity_denial())?)
            .map_err(admission_denial)?;
        admission
            .charge_external_work(u64::try_from(bytes).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        Ok(Self {
            producer: producer.to_owned(),
            source: ready.key().source_epoch().clone(),
            index,
        })
    }
}
impl PartialEq for RequiredMemberKey {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for RequiredMemberKey {}
impl PartialOrd for RequiredMemberKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for RequiredMemberKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.producer
            .cmp(&other.producer)
            .then_with(|| self.source.cmp(&other.source))
    }
}
