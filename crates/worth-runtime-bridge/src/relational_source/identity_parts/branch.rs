use super::{BridgeIdentity, BridgeIdentityPayload, TruthBranchTag, RELATIONAL_BRANCH_PREFIX};
use std::sync::Arc;

impl BridgeIdentity<TruthBranchTag> {
    /// Exact initialized width of the label made by the relational branch
    /// projection. Callers preparing that projection can fund both its
    /// temporary String and final Arc backing before construction.
    pub fn relational_branch_framed_len(branch_id: &str) -> Option<usize> {
        RELATIONAL_BRANCH_PREFIX.len().checked_add(branch_id.len())
    }

    /// The Bridge identity of a Relational branch.
    pub fn from_relational_branch_id(branch_id: impl Into<Arc<str>>) -> Self {
        let branch_id = branch_id.into();
        let mut framed = String::with_capacity(
            Self::relational_branch_framed_len(&branch_id)
                .expect("an existing branch label fits its framed identity"),
        );
        framed.push_str(RELATIONAL_BRANCH_PREFIX);
        framed.push_str(&branch_id);
        Self::with_payload(
            framed,
            BridgeIdentityPayload::RelationalBranch { branch_id },
        )
    }

    /// The Relational branch id, if this identity names a Relational branch.
    pub fn relational_branch_id(&self) -> Option<&str> {
        match self.payload() {
            BridgeIdentityPayload::RelationalBranch { branch_id } => Some(branch_id.as_ref()),
            _ => None,
        }
    }

    /// A test branch identity whose branch id is the fixture label itself.
    pub fn from_bridge_harness_label(label: impl Into<Arc<str>>) -> Self {
        Self::from_relational_branch_id(label)
    }
}
