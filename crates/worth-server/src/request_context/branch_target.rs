#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum WorthServerBranchTarget {
    Main,
    Branch { branch_id: String },
    Preview { preview_id: String },
}

impl WorthServerBranchTarget {
    pub(crate) fn owned_allocation_capacity_bytes(&self) -> u64 {
        use crate::product_adapter::execution_pipeline::read_batch_accounting::string;
        match self {
            Self::Main => 0,
            Self::Branch { branch_id } => string(branch_id),
            Self::Preview { preview_id } => string(preview_id),
        }
    }
    pub fn canonical_label(&self) -> String {
        match self {
            Self::Main => "main".to_string(),
            Self::Branch { branch_id } => format!("branch:{branch_id}"),
            Self::Preview { preview_id } => format!("preview:{preview_id}"),
        }
    }

    pub fn branch_digest(&self) -> String {
        format!("worth-server-branch-target-v1:{}", self.canonical_label())
    }
}
