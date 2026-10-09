mod admission;
mod classification;
mod cleanup;
pub(in crate::data::graph) mod epoch_preparation;
mod epoch_storage_readiness;
mod epoch_subscribers;
mod node_preparation;
pub(crate) use epoch_preparation::PreparedDependencyTopologyEpoch;
pub(crate) use epoch_preparation::PreparedDependencyTopologyStorage;
mod preflight;
#[cfg(test)]
mod test_edits;

#[cfg(test)]
mod consistency_tests;
#[cfg(test)]
mod preflight_tests;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DependencyReconciliationReport {
    pub added: u32,
    pub removed: u32,
    pub unchanged: u32,
}

pub(super) use classification::SubscriberBatchOp;
