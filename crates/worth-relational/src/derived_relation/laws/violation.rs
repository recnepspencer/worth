//! Exact derived-law failures name logical endpoints and real physical support.
use crate::graph_member::NativeMemberKey;
use crate::identity::data::{EntityId, RelationId};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum DerivedRelationLaw {
    SelfEdges,
    CrossContext,
    SourceMinimum,
    SourceMaximum,
    TargetMinimum,
    TargetMaximum,
    PairMinimum,
    PairMaximum,
    Acyclic,
    Symmetric,
    Orbit,
    SamePartition,
    MinimumReachability,
}
impl DerivedRelationLaw {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SelfEdges => "self-edges",
            Self::CrossContext => "cross-context",
            Self::SourceMinimum => "source-minimum",
            Self::SourceMaximum => "source-maximum",
            Self::TargetMinimum => "target-minimum",
            Self::TargetMaximum => "target-maximum",
            Self::PairMinimum => "pair-minimum",
            Self::PairMaximum => "pair-maximum",
            Self::Acyclic => "acyclic",
            Self::Symmetric => "symmetric",
            Self::Orbit => "orbit",
            Self::SamePartition => "same-partition",
            Self::MinimumReachability => "minimum-reachability",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DerivedRelationLawViolation {
    pub law: DerivedRelationLaw,
    pub member: NativeMemberKey,
    pub counterpart: Option<NativeMemberKey>,
    pub count: u64,
    pub limit: u64,
    pub physical_entities: Vec<EntityId>,
    pub physical_relations: Vec<RelationId>,
}
impl DerivedRelationLawViolation {
    pub(crate) fn owned_bytes(&self) -> u64 {
        (self.member.owned_bytes().unwrap_or(usize::MAX) as u64)
            .saturating_add(
                self.counterpart
                    .as_ref()
                    .and_then(NativeMemberKey::owned_bytes)
                    .unwrap_or(0) as u64,
            )
            .saturating_add(
                self.physical_entities
                    .capacity()
                    .saturating_mul(size_of::<EntityId>()) as u64,
            )
            .saturating_add(
                self.physical_relations
                    .capacity()
                    .saturating_mul(size_of::<RelationId>()) as u64,
            )
    }
}
