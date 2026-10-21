//! Native inspection binds a Current certificate to the actual row it read.
use super::*;
use crate::domain_computation::primary_graph::application_contribution::CurrentBasis;
impl RequiredWorkMembership {
    pub(in crate::domain_computation::primary_graph) fn certify_current_basis(
        self: &Arc<Self>,
        basis: CurrentBasis,
    ) -> Arc<Self> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current_basis = Some(basis);
        Arc::clone(self)
    }
    pub(in crate::domain_computation::primary_graph) fn certified_at(
        &self,
        basis: &CurrentBasis,
    ) -> bool {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .current_basis
            .as_ref()
            == Some(basis)
    }
}
impl SelectedReadyReadmission {
    pub(in crate::domain_computation::primary_graph) fn certify_current_record(
        &self,
        basis: CurrentBasis,
    ) -> Arc<RequiredWorkMembership> {
        self.membership.certify_current_basis(basis)
    }
}

impl WorthQueryOutputDemandRegistry {
    /// The cause stays on the claimed predecessor. Its Current proof belongs
    /// to the live Ready that now answers that occurrence, including the
    /// installed Initial-to-Preserve succession. Pin it before certification;
    /// a foreign record or basis still cannot acknowledge either cause.
    pub(in super::super) fn pin_current_required_successor(
        &self,
        work: &mut SelectedRequiredWork,
    ) -> Result<(), WorthQueryOutputDemandDenial> {
        let state = self
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);

        if !state
            .records
            .get(work.key())
            .is_some_and(super::super::refreshed_rejoin::superseded)
        {
            return Ok(());
        }
        let successor = super::super::refreshed_rejoin::newest_matching_row(
            &state.records,
            work.key(),
            |key, row| {
                matches!(
                    work.key().replacement_order(key),
                    Some(std::cmp::Ordering::Less)
                ) && matches!(&row.state, super::super::DemandState::Output(output)
                        if matches!(output.advancement(), super::super::WorthQueryOutputAdvancement::Idle)
                            && row.has_cached_ready())
            },
        );
        if let Some(successor) = successor {
            work.certification_membership = Arc::clone(
                state.records[&successor]
                    .work_membership
                    .as_ref()
                    .expect("an admitted successor owns its required record"),
            );
        }
        Ok(())
    }
}
