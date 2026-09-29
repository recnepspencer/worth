//! Versioned, authority-free codec for authored workflow definitions.
//!
//! An archive carries a definition draft only: instances, approvals and live
//! authority never travel. Decoding yields an untrusted draft that names its
//! vocabulary by identifier. Authoring it against an installed workflow spec
//! rebuilds the typed definition, which then validates, binds and publishes
//! like any other; decoded bytes alone mint nothing.

use worth_query_declaration::facade::application_program::{
    ApplicationWorkflowConnectionKind, ApplicationWorkflowDefinitionLimits,
};

mod codec;
mod compatibility;
mod records;
#[cfg(test)]
mod tests;

pub use codec::{
    decode_workflow_definition_draft, encode_workflow_definition_draft,
    WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_OLDEST_READABLE_VERSION,
    WORTH_QUERY_WORKFLOW_DEFINITION_DRAFT_PROTOCOL_VERSION,
};
pub use compatibility::{
    WorthQueryWorkflowDefinitionDraftDenial, WorthQueryWorkflowDefinitionDraftDenialKind,
};

/// A structurally decoded workflow definition naming its vocabulary by
/// identifier. It carries no Query authority and is no validated definition:
/// `author` rebuilds typed meaning against an installed spec.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryUntrustedWorkflowDefinitionDraft {
    spec: String,
    identity: String,
    limits: ApplicationWorkflowDefinitionLimits,
    start: String,
    nodes: Box<[DraftNode]>,
    connections: Box<[DraftConnection]>,
}

impl WorthQueryUntrustedWorkflowDefinitionDraft {
    /// The workflow spec the draft was authored under.
    pub fn spec(&self) -> &str {
        &self.spec
    }

    /// The workflow identity the draft would publish under.
    pub fn identity(&self) -> &str {
        &self.identity
    }

    pub const fn limits(&self) -> ApplicationWorkflowDefinitionLimits {
        self.limits
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn connection_count(&self) -> usize {
        self.connections.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftNode {
    identity: String,
    member: DraftMember,
}

/// One node's vocabulary member as the draft names it, with the portable
/// types it was authored against so a changed vocabulary is refused.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DraftMember {
    Operation {
        identifier: String,
        input_type: String,
        binding: Option<String>,
        requires_workflow_authority: bool,
    },
    Assessment {
        identifier: String,
        parameter_type: String,
        result_type: String,
        subject: String,
        related_relation: Option<[String; 3]>,
    },
    Condition(DraftCondition),
    Approval {
        identifier: String,
        capability_type: String,
    },
    EvidenceJoin {
        policy: String,
    },
    AwaitInbound(DraftInbound),
    Terminal,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftInbound {
    origin: String,
    effect: String,
    protocol:
        worth_query_declaration::facade::application_schema::ApplicationInboundOccurrenceProtocol,
    source: String,
    limits: worth_query_declaration::facade::application_schema::ApplicationInboundOccurrenceLimits,
}

/// A condition as the draft names it.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DraftCondition {
    /// An encoded expression over operands in name order.
    Expression {
        draft: Box<[u8]>,
        operands: Box<[DraftConditionOperand]>,
    },
    /// A version-1 condition: one Bool query read as the whole decision.
    Migrated(DraftConditionOperand),
}

/// One named operand and the installed query it reads.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftConditionOperand {
    name: String,
    identifier: String,
    parameter_type: String,
    result_type: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DraftConnection {
    source: String,
    target: String,
    kind: ApplicationWorkflowConnectionKind,
}
