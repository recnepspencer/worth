//! A selected Ready and its admitted lifecycle successor name one member.
use super::*;

impl PerformedMembers {
    /// Only admission of the actual successor may join another demand key.
    /// The sealed result is required by the selected publication handoff.
    pub(in crate::domain_computation::primary_graph::application_contribution::producer) fn attach_successor(
        &mut self,
        readiness: FreshReadiness,
        successor: &WorthQueryOutputDemandKey,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PublicationReadiness, WorthQueryOutputDemandDenial> {
        let entry = &mut self.entries[readiness.member];
        admission
            .charge_external_work(entry.comparison_work()?)
            .map_err(admission_denial)?;
        if !entry.names(successor) {
            // Registry succession permits Initial to Preserve (including another
            // binding), not a sequence of arbitrary binding replacements.
            // A reselect of the admitted successor already names this entry.
            let bytes = std::mem::size_of::<WorthQueryOutputDemandKey>()
                .checked_add(successor.producer_identity().len())
                .ok_or_else(capacity_denial)?;
            admission
                .charge_external_work(u64::try_from(bytes).map_err(|_| work_denial())?)
                .map_err(admission_denial)?;
            admission
                .admit_read_scratch(u64::try_from(bytes).map_err(|_| capacity_denial())?)
                .map_err(admission_denial)?;
            entry.successor = Some(Box::new(successor.clone()));
        }
        Ok(PublicationReadiness {
            member: readiness.member,
        })
    }
}

impl PerformedMember {
    pub(super) fn comparison_work(&self) -> Result<u64, WorthQueryOutputDemandDenial> {
        let primary = key_comparison_work(&self.key)?;
        match &self.successor {
            Some(successor) => primary
                .checked_add(key_comparison_work(successor)?)
                .ok_or_else(work_denial),
            None => Ok(primary),
        }
    }
    pub(super) fn names(&self, key: &WorthQueryOutputDemandKey) -> bool {
        same_key(&self.key, key)
            || self
                .successor
                .as_ref()
                .is_some_and(|successor| same_key(successor, key))
    }
}

fn key_comparison_work(
    key: &WorthQueryOutputDemandKey,
) -> Result<u64, WorthQueryOutputDemandDenial> {
    key.producer_identity()
        .len()
        .checked_add(key.applicability().profile_kind().len())
        .and_then(|bytes| bytes.checked_add(3))
        .and_then(|work| u64::try_from(work).ok())
        .ok_or_else(work_denial)
}
fn same_key(left: &WorthQueryOutputDemandKey, right: &WorthQueryOutputDemandKey) -> bool {
    left.family_identity() == right.family_identity()
        && left.producer_identity() == right.producer_identity()
        && left.applicability() == right.applicability()
        && left.source_epoch().same_occurrence(right.source_epoch())
}
