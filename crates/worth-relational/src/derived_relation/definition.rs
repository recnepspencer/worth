//! Descriptive recipes and the separate registry-issued executable handle.
use std::sync::Arc;

use crate::graph_member::NativeMemberDomain;
use crate::identity::data::KindId;
use crate::projected_member_domain::ProjectedMemberDomainIdentity;
use crate::schema::data::{SchemaId, SchemaVersionId};
use serde::{Deserialize, Serialize};

/// A recipe's declared identity. This value grants no read authority.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DerivedRelationIdentity {
    pub schema: SchemaId,
    pub version: SchemaVersionId,
    pub name: String,
}

/// Finite postorder grammar; the last node is the expression's root.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum DerivedRelationNode {
    Stored {
        relation: KindId,
        from: KindId,
        to: KindId,
    },
    Derived(DerivedRelationIdentity),
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
        domain: ProjectedMemberDomainIdentity,
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

/// Portable Native meaning, never a registered binding or prepared read.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DerivedRelationDefinitionParts {
    pub identity: DerivedRelationIdentity,
    pub from: NativeMemberDomain,
    pub to: NativeMemberDomain,
    pub nodes: Vec<DerivedRelationNode>,
    /// Absent for the existing statically proved unbounded contract. The
    /// optional tail changes no legacy physical recipe's serialized meaning.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub laws: Option<super::DerivedRelationGraphLaws>,
}

impl DerivedRelationDefinitionParts {
    pub(crate) fn owned_capacity_bytes(&self) -> usize {
        self.laws
            .as_ref()
            .map_or(0, super::DerivedRelationGraphLaws::owned_bytes)
            + self.from.owned_bytes()
            + self.to.owned_bytes()
            + self.identity.schema.0.capacity()
            + self.identity.name.capacity()
            + self.nodes.capacity() * std::mem::size_of::<DerivedRelationNode>()
            + self
                .nodes
                .iter()
                .map(|node| match node {
                    DerivedRelationNode::Derived(identity) => {
                        identity.schema.0.capacity() + identity.name.capacity()
                    }
                    DerivedRelationNode::CarrierLift { domain, case } => {
                        domain.owned_bytes() + case.capacity()
                    }
                    _ => 0,
                })
                .sum::<usize>()
    }
}

/// An exact entry issued by one registry. Registry clones share this owner;
/// independently registering equal descriptive Parts creates another owner.
#[derive(Clone, Debug)]
pub struct DerivedRelationHandle {
    pub(crate) entry: Arc<RegisteredDerivedRelation>,
}
impl PartialEq for DerivedRelationHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.entry, &other.entry)
    }
}
impl Eq for DerivedRelationHandle {}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct RegisteredDerivedRelation {
    pub(crate) parts: DerivedRelationDefinitionParts,
    pub(crate) leaves: Vec<(KindId, KindId, KindId)>,
    pub(crate) domains: Vec<ProjectedMemberDomainIdentity>,
    pub(crate) carriers: Vec<KindId>,
    pub(crate) endpoints: Vec<(NativeMemberDomain, NativeMemberDomain)>,
}

impl DerivedRelationHandle {
    pub fn parts(&self) -> &DerivedRelationDefinitionParts {
        &self.entry.parts
    }

    /// Every physical source, including sources of named recipes.
    pub fn physical_sources(&self) -> &[(KindId, KindId, KindId)] {
        &self.entry.leaves
    }

    /// Every finite carrier domain, including named and Boolean branches.
    pub fn projected_domains(&self) -> &[ProjectedMemberDomainIdentity] {
        &self.entry.domains
    }
}

/// Why descriptive registration or an exact source could not be admitted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DerivedRelationDenial {
    InvalidIdentity,
    DuplicateDefinition(DerivedRelationIdentity),
    MissingDefinition(DerivedRelationIdentity),
    CyclicDefinitions,
    InvalidExpression,
    InvalidPhysicalSource(KindId),
    DiscontinuousEndpoints,
    ForeignDefinition,
    ForeignObservation,
    OutsideDeclaredAccess,
    RecordUnavailable,
    WorkBudgetExceeded,
    InvalidMemberDomain,
    InvalidMemberCase,
    InvalidGraphLaws,
}
