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
    /// Newest-first physical event chain. Retirement precedes introduction at
    /// each event; these indices must name three earlier literal Stored leaves.
    OrderedEvents {
        predecessor: usize,
        introduced: usize,
        retired: usize,
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
    pub(crate) fn clone_allocation_bytes(
        &self,
        control: crate::schema::SchemaCompilationControl<'_, '_>,
    ) -> Result<Option<usize>, crate::execution::RelationalExecutionDenialCause> {
        let mut stopped = None;
        let bytes = self.measure_clone_bytes(&mut || match control.checkpoint() {
            Ok(()) => Some(()),
            Err(cause) => {
                stopped = Some(cause);
                None
            }
        });
        match stopped {
            Some(cause) => Err(cause),
            None => Ok(bytes),
        }
    }
    fn measure_clone_bytes(&self, checkpoint: &mut dyn FnMut() -> Option<()>) -> Option<usize> {
        let mut bytes = self
            .identity
            .schema
            .0
            .len()
            .checked_add(self.identity.name.len())?
            .checked_add(self.from.name_bytes()?)?
            .checked_add(self.to.name_bytes()?)?
            .checked_add(
                self.nodes
                    .len()
                    .checked_mul(size_of::<DerivedRelationNode>())?,
            )?;
        for node in &self.nodes {
            checkpoint()?;
            let names = match node {
                DerivedRelationNode::Derived(identity) => {
                    identity.schema.0.len().checked_add(identity.name.len())?
                }
                DerivedRelationNode::CarrierLift { domain, case } => {
                    domain.name_bytes()?.checked_add(case.len())?
                }
                DerivedRelationNode::Stored { .. }
                | DerivedRelationNode::Compose { .. }
                | DerivedRelationNode::Reverse { .. }
                | DerivedRelationNode::WithoutSelfEdges { .. }
                | DerivedRelationNode::Union { .. }
                | DerivedRelationNode::Intersection { .. }
                | DerivedRelationNode::OrderedEvents { .. } => 0,
            };
            bytes = bytes.checked_add(names)?;
        }
        if let Some(laws) = &self.laws {
            bytes = bytes.checked_add(laws.clone_allocation_bytes(checkpoint)?)?;
        }
        Some(bytes)
    }

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
    /// Memoization follows this exact registered path owner across clones and
    /// registry composition. It contributes no descriptive schema meaning.
    pub(crate) managed_source_binding: crate::validation::candidate_source::SourceBindingCache,
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
    InvalidEventChain,
    OrderedPredecessorMembershipRequired,
    OrderedPredecessorMustBeToOne,
    SourceProgramIdentityExhausted,
}
