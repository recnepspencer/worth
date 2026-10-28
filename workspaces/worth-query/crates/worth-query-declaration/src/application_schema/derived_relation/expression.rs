//! Portable directed relation composition; indices describe syntax, never records.
use crate::application_schema::ApplicationGraphMemberMeaning;
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
    pub from: ApplicationGraphMemberMeaning,
    pub to: ApplicationGraphMemberMeaning,
}

/// A node refers only to earlier nodes. Transport validation enforces that order.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub enum ApplicationDerivedRelationNode {
    Source(ApplicationDerivedRelationSource),
    Compose {
        left: usize,
        right: usize,
    },
    Reverse {
        input: usize,
    },
    WithoutSelfEdges {
        input: usize,
    },
    CarrierLift {
        domain: String,
        carrier: String,
        case: String,
    },
    Union {
        left: usize,
        right: usize,
    },
    Intersection {
        left: usize,
        right: usize,
    },
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

    pub(super) fn compose(self, other: Self) -> Self {
        self.append(other, |left, right| {
            ApplicationDerivedRelationNode::Compose { left, right }
        })
    }

    pub(super) fn union(self, other: Self) -> Self {
        self.append(other, |left, right| ApplicationDerivedRelationNode::Union {
            left,
            right,
        })
    }
    pub(super) fn intersection(self, other: Self) -> Self {
        self.append(other, |left, right| {
            ApplicationDerivedRelationNode::Intersection { left, right }
        })
    }

    fn append(
        mut self,
        other: Self,
        operation: fn(usize, usize) -> ApplicationDerivedRelationNode,
    ) -> Self {
        let left = self.nodes.len() - 1;
        let offset = self.nodes.len();
        for node in other.nodes {
            self.nodes.push(match node {
                node @ (ApplicationDerivedRelationNode::Source(_)
                | ApplicationDerivedRelationNode::CarrierLift { .. }) => node,
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
                ApplicationDerivedRelationNode::Union { left, right } => {
                    ApplicationDerivedRelationNode::Union {
                        left: left + offset,
                        right: right + offset,
                    }
                }
                ApplicationDerivedRelationNode::Intersection { left, right } => {
                    ApplicationDerivedRelationNode::Intersection {
                        left: left + offset,
                        right: right + offset,
                    }
                }
            });
        }
        let right = self.nodes.len() - 1;
        self.nodes.push(operation(left, right));
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

    /// Whether the exact old tag41 grammar can represent this expression.
    pub fn is_legacy_shape(&self) -> bool {
        self.nodes.iter().all(|node| match node {
            ApplicationDerivedRelationNode::Source(source) => {
                source.from.entity().is_some() && source.to.entity().is_some()
            }
            ApplicationDerivedRelationNode::Compose { .. }
            | ApplicationDerivedRelationNode::Reverse { .. }
            | ApplicationDerivedRelationNode::WithoutSelfEdges { .. } => true,
            ApplicationDerivedRelationNode::CarrierLift { .. }
            | ApplicationDerivedRelationNode::Union { .. }
            | ApplicationDerivedRelationNode::Intersection { .. } => false,
        })
    }
}
