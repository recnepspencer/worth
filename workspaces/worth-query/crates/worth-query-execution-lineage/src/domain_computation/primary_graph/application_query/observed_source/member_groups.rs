//! A row member's complete group membership, as a projected group name read
//! it: regrouping a member changes no record its row's footprint holds.

use crate::domain_computation::primary_graph::application_attempt::fact::persistent_name_history::{
    PersistentNameHistoryAnswer, PersistentNameHistoryQuestion, PersistentNameHistoryRead,
};
use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
use worth_relational::facade::lineage::PersistentNameSubjectKind;

/// Every `(group kind, group)` holding one named entity member at the read's
/// basis, ascending, with the member's installed entity kind.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct WorthQueryObservedMemberGroups {
    pub member: [u8; 32],
    pub kind: String,
    pub groups: Vec<(String, [u8; 32])>,
}

impl WorthQueryObservedMemberGroups {
    /// The source fact a consumer re-verifies: a `MemberGroups` name-history
    /// read with this exact answer.
    pub fn fact(&self) -> Fact {
        Fact::PersistentNameHistory {
            read: PersistentNameHistoryRead {
                kind: self.kind.clone(),
                subject: PersistentNameSubjectKind::Entity,
                question: PersistentNameHistoryQuestion::MemberGroups(self.member),
            },
            answer: PersistentNameHistoryAnswer::MemberGroups(self.groups.clone()),
        }
    }

    /// Heap bytes this record (and the fact made from it) owns.
    pub fn owned_bytes(&self) -> u64 {
        let groups = self
            .groups
            .iter()
            .map(|(kind, _)| kind.len().saturating_add(40));
        let bytes = groups.fold(self.kind.len(), usize::saturating_add);
        u64::try_from(bytes).unwrap_or(u64::MAX)
    }
}
