use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationQueryAccessReceipt, WorthQueryApplicationQueryBasisPosture,
};

/// Authority-free description of the exact Relational basis used by a
/// published application result.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryPublishedApplicationBasis {
    runtime_instance: u64,
    branch: String,
    snapshot: u64,
    version: u64,
    posture: WorthQueryPublishedApplicationBasisPosture,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryPublishedApplicationBasisPosture {
    SelectedProduct,
}

impl WorthQueryPublishedApplicationBasis {
    pub(super) fn capture(receipt: &WorthQueryApplicationQueryAccessReceipt) -> Self {
        let identity = receipt.basis_identity();
        Self {
            runtime_instance: identity.runtime_instance_id(),
            branch: identity.branch_id().0.clone(),
            snapshot: identity.snapshot_id().0,
            version: receipt.basis_version().as_u64(),
            posture: match receipt.basis_posture() {
                WorthQueryApplicationQueryBasisPosture::SelectedProduct => {
                    WorthQueryPublishedApplicationBasisPosture::SelectedProduct
                }
            },
        }
    }

    pub const fn runtime_instance(&self) -> u64 {
        self.runtime_instance
    }

    pub fn branch(&self) -> &str {
        &self.branch
    }

    pub const fn snapshot(&self) -> u64 {
        self.snapshot
    }

    pub const fn version(&self) -> u64 {
        self.version
    }

    pub const fn posture(&self) -> WorthQueryPublishedApplicationBasisPosture {
        self.posture
    }
}
