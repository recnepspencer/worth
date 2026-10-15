//! Proposal locator backing: ordinary receipts or request-funded read projections.
use super::*;
#[derive(Clone, Debug)]
pub(super) enum ProposalText {
    Receipt(String),
    Read(worth_execution::ExecutionImmutableBytes),
}
impl From<String> for ProposalText {
    fn from(value: String) -> Self {
        Self::Receipt(value)
    }
}
impl std::ops::Deref for ProposalText {
    type Target = str;
    fn deref(&self) -> &str {
        match self {
            Self::Receipt(v) => v,
            Self::Read(v) => std::str::from_utf8(v.bytes()).expect("native UTF-8 projection"),
        }
    }
}
impl PartialEq for ProposalText {
    fn eq(&self, other: &Self) -> bool {
        **self == **other
    }
}
impl Eq for ProposalText {}
/// A reference to a published workflow proposal: its identity, the operation and input it
/// proposes, and the node path it was proposed at.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PublishedWorkflowProposalRef {
    pub(super) branch: crate::basis::WorthQueryProductBranch,
    pub(super) entity: EntityId,
    pub(super) instance: EntityId,
    pub(super) definition: EntityId,
    pub(super) identity: ProposalText,
    pub(super) operation: ProposalText,
    pub(super) input_type: ProposalText,
    pub(super) input_identity: [u8; 32],
    pub(super) source_identity: Option<[u8; 32]>,
    pub(super) node_path: ProposalText,
}

impl PublishedWorkflowProposalRef {
    pub const fn entity_id(&self) -> EntityId {
        self.entity
    }
    pub const fn instance_entity_id(&self) -> EntityId {
        self.instance
    }
    pub const fn definition_entity_id(&self) -> EntityId {
        self.definition
    }
    pub fn identity(&self) -> &str {
        &self.identity
    }
    pub fn operation(&self) -> &str {
        &self.operation
    }
    pub fn input_type(&self) -> &str {
        &self.input_type
    }
    pub const fn input_identity(&self) -> &[u8; 32] {
        &self.input_identity
    }
    pub const fn source_identity(&self) -> Option<&[u8; 32]> {
        self.source_identity.as_ref()
    }
    pub fn node_path(&self) -> &str {
        &self.node_path
    }
    pub const fn branch(&self) -> crate::basis::WorthQueryProductBranch {
        self.branch
    }
}
