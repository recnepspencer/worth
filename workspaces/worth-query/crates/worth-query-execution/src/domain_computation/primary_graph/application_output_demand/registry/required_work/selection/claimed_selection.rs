//! Claim now, select only at the owner's inspection of the current basis.
use super::*;
impl WorthQueryOutputDemandRegistry {
    /// Claim one required row for the already admitted Product and its exact
    /// native branch. A foreign-cell hint stays queued; a later hint for this
    /// cell cannot be hidden behind it.
    pub(in crate::domain_computation::primary_graph) fn next_required_claim_for_selected(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        source_owner: &SourceInvalidationOwner,
    ) -> Result<Option<RequiredWorkClaim>, WorthQueryOutputDemandDenial> {
        self.maintain_selected_required_cleanup(product, selected, source_owner)?;
        let queue = {
            let state = self
                .state
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            state.required_work_queue.clone()
        };
        let Some(queue) = queue else {
            return Ok(None);
        };

        let queued = queue.queued_count();
        for _ in 0..queued {
            let work = match queue.pop() {
                RequiredWorkPop::Empty => return Ok(None),
                RequiredWorkPop::Stale => continue,
                RequiredWorkPop::Claimed(work) => work,
            };
            let membership = {
                let state = self
                    .state
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);

                let occurrence = match state.records.get(work.key()) {
                    Some(record)
                        if (record.is_required()
                            || super::super::super::refreshed_rejoin::awaited_by_stale_owner(
                                &state.records,
                                work.key(),
                            ))
                            && record
                                .work_membership
                                .as_ref()
                                .is_some_and(|member| Arc::ptr_eq(member, &work.membership))
                            && state.required_keys.contains(work.key()) =>
                    {
                        Some(record.product_occurrence)
                    }
                    _ => None,
                };
                if occurrence.is_none() {
                    work.membership.set_required(false);
                }
                occurrence
            };
            let Some(occurrence) = membership else {
                continue;
            };
            if occurrence != product.observation().lifecycle_incarnation() {
                continue;
            }
            return Ok(Some(work));
        }
        Ok(None)
    }
    /// A wave claims each record once, but later publications can release
    /// terminal custody. Maintain that custody on the same paid selected door
    /// without popping another required record.
    pub(in crate::domain_computation::primary_graph) fn maintain_selected_required_cleanup(
        &self,
        product: &crate::basis::WorthQueryProductObservationLease,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
        source_owner: &SourceInvalidationOwner,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let basis = product.relational_basis_descriptor();

        if basis.runtime_instance_id() != selected.runtime_instance_id()
            || basis.branch_id() != selected.branch_id()
            || basis.root_identity() != selected.root_id()
        {
            return Err(WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                "selected required-work Product differs from its native root",
            ));
        }

        self.drain_terminal_cleanup_admitted(source_owner)
    }
    pub(in crate::domain_computation::primary_graph) fn select_required_claim(
        &self,
        claim: &mut RequiredWorkClaim,
        selected: &worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Result<Option<SelectedRequiredWork>, WorthQueryOutputDemandDenial> {
        let Some(mut work) = claim.select() else {
            return Ok(None);
        };
        if work.for_selected_branch(selected.branch_id())? {
            self.pin_current_required_successor(&mut work)?;
            Ok(Some(work))
        } else {
            Ok(None)
        }
    }
}
