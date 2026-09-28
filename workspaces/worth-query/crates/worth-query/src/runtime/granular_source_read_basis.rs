/// Query-carried identity of the primary truth snapshot admitted for one
/// granular maintenance read.
///
/// This is not read authority. The primary source adapter revalidates it
/// against its retained graph immediately before and after projection.
#[doc(hidden)]
#[derive(Clone, Debug)]
pub struct WorthQueryGranularSourceReadBasis {
    snapshot: worth_runtime_bridge::facade::TruthSnapshotIdentity,
    branch: worth_runtime_bridge::facade::TruthBranchIdentity,
    _observation: std::sync::Arc<worth_runtime_bridge::facade::RelationalBridgeObservationLease>,
}

impl WorthQueryGranularSourceReadBasis {
    pub(crate) fn from_execution_basis(
        basis: &worth_query_execution::facade::primary_graph::WorthQueryGranularSourceReadBasis,
    ) -> Self {
        Self {
            snapshot: basis.snapshot().clone(),
            branch: basis.branch().clone(),
            _observation: basis.retain_observation(),
        }
    }

    #[doc(hidden)]
    pub fn snapshot(&self) -> &worth_runtime_bridge::facade::TruthSnapshotIdentity {
        &self.snapshot
    }

    #[doc(hidden)]
    pub fn branch(&self) -> &worth_runtime_bridge::facade::TruthBranchIdentity {
        &self.branch
    }
}

impl PartialEq for WorthQueryGranularSourceReadBasis {
    fn eq(&self, other: &Self) -> bool {
        self.snapshot == other.snapshot && self.branch == other.branch
    }
}

impl Eq for WorthQueryGranularSourceReadBasis {}
