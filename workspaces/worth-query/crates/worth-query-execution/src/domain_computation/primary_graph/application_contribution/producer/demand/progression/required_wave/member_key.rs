//! The semantic order of required producer/source occurrences.
use super::*;
use crate::domain_computation::primary_graph::application_query::WorthQueryObservedSourceEpoch;
use std::cmp::Ordering;

// Binding fixes family; source fixes profile; lifecycle handoff stops the older Ready.
pub(super) struct RequiredMemberKey {
    producer: String,
    source: WorthQueryObservedSourceEpoch,
    pub index: usize,
    pub inspection: super::performed::InspectionStamp,
}
impl RequiredMemberKey {
    pub fn retain(
        index: usize,
        ready: &SelectedReadyReadmission,
        inspection: super::performed::InspectionStamp,
    ) -> Result<Self, WorthQueryOutputDemandDenial> {
        let producer = ready.producer_identity();

        Ok(Self {
            producer: producer.to_owned(),
            source: ready.key().source_epoch().clone(),
            index,
            inspection,
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
