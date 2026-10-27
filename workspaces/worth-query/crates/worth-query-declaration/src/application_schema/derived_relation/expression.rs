//! Portable directed relation composition; indices describe syntax, never records.
/// The installed namespace of a relation recipe's leaf.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationDerivedRelationSourceKind {
    Stored,
    Derived,
}

/// One physical or named-derived leg with its authored endpoint kinds.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationDerivedRelationSource {
    pub kind: ApplicationDerivedRelationSourceKind,
    pub relation: String,
    pub from: String,
    pub to: String,
}

/// A node refers only to earlier nodes. Transport validation enforces that order.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationDerivedRelationNode {
    Source(ApplicationDerivedRelationSource),
    Compose { left: usize, right: usize },
    Reverse { input: usize },
    WithoutSelfEdges { input: usize },
}

/// Finite endpoint-set meaning. Every supporting physical dependency survives
/// endpoint deduplication; syntax itself carries no read authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationDerivedRelationExpression {
    pub nodes: Vec<ApplicationDerivedRelationNode>,
}

impl ApplicationDerivedRelationExpression {
    pub(super) fn source(source: ApplicationDerivedRelationSource) -> Self {
        Self {
            nodes: vec![ApplicationDerivedRelationNode::Source(source)],
        }
    }

    pub(super) fn compose(mut self, other: Self) -> Self {
        let left = self.nodes.len() - 1;
        let offset = self.nodes.len();
        for node in other.nodes {
            self.nodes.push(match node {
                ApplicationDerivedRelationNode::Source(source) => {
                    ApplicationDerivedRelationNode::Source(source)
                }
                ApplicationDerivedRelationNode::Compose { left, right } => {
                    ApplicationDerivedRelationNode::Compose {
                        left: left + offset,
                        right: right + offset,
                    }
                }
                ApplicationDerivedRelationNode::Reverse { input } => {
                    ApplicationDerivedRelationNode::Reverse {
                        input: input + offset,
                    }
                }
                ApplicationDerivedRelationNode::WithoutSelfEdges { input } => {
                    ApplicationDerivedRelationNode::WithoutSelfEdges {
                        input: input + offset,
                    }
                }
            });
        }
        let right = self.nodes.len() - 1;
        self.nodes
            .push(ApplicationDerivedRelationNode::Compose { left, right });
        self
    }

    pub(super) fn reverse(mut self) -> Self {
        let input = self.nodes.len() - 1;
        self.nodes
            .push(ApplicationDerivedRelationNode::Reverse { input });
        self
    }

    pub(super) fn without_self_edges(mut self) -> Self {
        let input = self.nodes.len() - 1;
        self.nodes
            .push(ApplicationDerivedRelationNode::WithoutSelfEdges { input });
        self
    }
}
