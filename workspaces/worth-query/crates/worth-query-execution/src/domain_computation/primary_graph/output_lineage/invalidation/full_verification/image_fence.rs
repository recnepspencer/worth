//! Same-Native-position actor edits may change equality meaning during a probe.

use super::*;

impl FullVerificationImage<'_> {
    pub(in crate::domain_computation::primary_graph::output_lineage::invalidation) fn fence_semantic_image(
        &self,
        observed: &OrdSet<Arc<RecordedSettlementIdentity>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<(), FullVerificationStop> {
        let cell = self
            .owner
            .cell_for_read(self.selected, admission)?
            .ok_or(FullVerificationStop::ActorImageChanged)?;
        admission.work(3)?;
        let image = cell.read_image();
        admission.ordered_read(image.payload().past.len())?;
        let current = SnapshotAlignedMarkState::select_image(image, self.selected)
            .map_err(FullVerificationStop::Alignment)?
            .into_full_verification_state();
        admission.work(1)?;
        if !self.state.equal_links.ptr_eq(&current.equal_links) {
            return Err(FullVerificationStop::ActorImageChanged);
        }
        for identity in observed {
            admission.work(1)?;
            admission.ordered_read(self.state.settlements.len())?;
            admission.ordered_read(current.settlements.len())?;
            // A retired row an equality link still names was walked through,
            // not read: it is absent from both images unless one changed.
            let (old, fresh) = match (
                self.state.settlements.get(identity),
                current.settlements.get(identity),
            ) {
                (Some(old), Some(fresh)) => (old, fresh),
                (None, None) => continue,
                _ => return Err(FullVerificationStop::ActorImageChanged),
            };
            admission.work(6)?;
            let same_output = match (&old.output_facts, &fresh.output_facts) {
                (Some(old), Some(fresh)) => Arc::ptr_eq(&old.facts, &fresh.facts),
                (None, None) => true,
                _ => false,
            };
            // Own certified dirty-clear changes only marks. A registration or
            // Stable publication changes these semantic bindings and requires retry.
            if !old.facts.same_binding(&fresh.facts)
                || !Arc::ptr_eq(&old.read_basis, &fresh.read_basis)
                || !old.consumed_upstream.ptr_eq(&fresh.consumed_upstream)
                || old.output_coverage != fresh.output_coverage
                || old.verification_requirement != fresh.verification_requirement
                || !same_output
            {
                return Err(FullVerificationStop::ActorImageChanged);
            }
        }
        Ok(())
    }
}

pub(super) fn observe_row(
    observed: &mut OrdSet<Arc<RecordedSettlementIdentity>>,
    identity: &Arc<RecordedSettlementIdentity>,
    admission: &mut InvalidationEditAdmission,
) -> Result<(), FullVerificationStop> {
    admission.admit_visited_settlement(observed.len())?;
    observed.insert(Arc::clone(identity));
    Ok(())
}
