mod admission;
mod classification;
mod cleanup;
mod node_preparation;
mod preflight;
#[cfg(test)]
mod test_edits;

#[cfg(test)]
mod preflight_tests;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(crate) struct DependencyReconciliationReport {
    pub added: u32,
    pub removed: u32,
    pub unchanged: u32,
}

pub(super) use classification::SubscriberBatchOp;
