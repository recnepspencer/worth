use super::RootRoutingBlockDenial;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootRoutingBlockDecodeLimits {
    pub leaf_entries: u64,
    pub branch_children: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundedRootRoutingBlockDecodeDenial {
    Format(RootRoutingBlockDenial),
    LeafEntries { observed: u64, admitted: u64 },
    BranchChildren { observed: u64, admitted: u64 },
    CoordinateScratchInsufficient { required: usize, provided: usize },
}

impl From<RootRoutingBlockDenial> for BoundedRootRoutingBlockDecodeDenial {
    fn from(denial: RootRoutingBlockDenial) -> Self {
        Self::Format(denial)
    }
}
