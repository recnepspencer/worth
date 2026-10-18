//! Attribution of an actual source-query observation to its original attempt.
use crate::domain_computation::{
    authorization::WorthQueryOperationAdmissionIdentity,
    primary_graph::{
        application_contribution::OutputSourceAddress,
        output_lineage::invalidation::ObservedSourcePublication,
    },
};
use std::{cmp::Ordering, sync::Arc};

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) struct SourcePublicationRead {
    admission: WorthQueryOperationAdmissionIdentity,
    address: Arc<OutputSourceAddress>,
}
impl SourcePublicationRead {
    pub(in crate::domain_computation::primary_graph) fn new(
        admission: WorthQueryOperationAdmissionIdentity,
        observation: Arc<ObservedSourcePublication>,
    ) -> Self {
        Self {
            admission,
            address: observation.address(),
        }
    }
    pub(in crate::domain_computation::primary_graph) fn admits_observation(
        &self,
        admission: WorthQueryOperationAdmissionIdentity,
        observation: &ObservedSourcePublication,
    ) -> bool {
        self.admits(admission) && Arc::ptr_eq(&self.address, &observation.address())
    }
    pub(in crate::domain_computation::primary_graph) fn admits(
        &self,
        admission: WorthQueryOperationAdmissionIdentity,
    ) -> bool {
        self.admission == admission
    }
}
// Keys identify the original read, not its result. A second read cannot replace
// the first observation whose expected value the original attempt must validate.
impl Ord for SourcePublicationRead {
    fn cmp(&self, other: &Self) -> Ordering {
        self.admission
            .cmp(&other.admission)
            .then_with(|| self.address.cmp(&other.address))
    }
}
impl PartialOrd for SourcePublicationRead {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialEq for SourcePublicationRead {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Eq for SourcePublicationRead {}
