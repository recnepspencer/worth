//! A selected Ready and its admitted lifecycle successor name one member.
use super::*;

impl PerformedMembers {
    /// Only admission of the actual successor may join another demand key.
    /// The sealed result is required by the selected publication handoff.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn attach_successor(
        &mut self,
        readiness: FreshReadiness,
        successor: &WorthQueryOutputDemandKey,
    ) -> Result<PublicationReadiness, WorthQueryOutputDemandDenial> {
        let entry = &mut self.entries[readiness.member];

        if !entry.names(successor) {
            // Registry succession permits Initial to Preserve (including another
            // binding), not a sequence of arbitrary binding replacements.
            // A reselect of the admitted successor already names this entry.

            entry.successor = Some(Box::new(successor.clone()));
        }
        Ok(PublicationReadiness {
            member: readiness.member,
        })
    }
}

impl PerformedMember {
    pub(super) fn names(&self, key: &WorthQueryOutputDemandKey) -> bool {
        same_key(&self.key, key)
            || self
                .successor
                .as_ref()
                .is_some_and(|successor| same_key(successor, key))
    }
}

fn same_key(left: &WorthQueryOutputDemandKey, right: &WorthQueryOutputDemandKey) -> bool {
    left.family_identity() == right.family_identity()
        && left.producer_identity() == right.producer_identity()
        && left.applicability() == right.applicability()
        && left.source_epoch().same_occurrence(right.source_epoch())
}
