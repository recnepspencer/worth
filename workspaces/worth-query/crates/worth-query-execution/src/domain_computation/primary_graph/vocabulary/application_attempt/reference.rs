//! Moved whole from `domain_computation::primary_graph::application_attempt::workflow_proposal_program::publication::reference`: the items below and every impl they own.
use worth_relational::facade::identity::EntityId;

#[derive(Clone, Debug)]
pub(in crate::domain_computation::primary_graph) enum ProposalText {
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
    pub(in crate::domain_computation::primary_graph) branch: crate::basis::WorthQueryProductBranch,
    pub(in crate::domain_computation::primary_graph) entity: EntityId,
    pub(in crate::domain_computation::primary_graph) instance: EntityId,
    pub(in crate::domain_computation::primary_graph) definition: EntityId,
    pub(in crate::domain_computation::primary_graph) identity: ProposalText,
    pub(in crate::domain_computation::primary_graph) operation: ProposalText,
    pub(in crate::domain_computation::primary_graph) input_type: ProposalText,
    pub(in crate::domain_computation::primary_graph) input_identity: [u8; 32],
    pub(in crate::domain_computation::primary_graph) source_identity: Option<[u8; 32]>,
    pub(in crate::domain_computation::primary_graph) node_path: ProposalText,
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
