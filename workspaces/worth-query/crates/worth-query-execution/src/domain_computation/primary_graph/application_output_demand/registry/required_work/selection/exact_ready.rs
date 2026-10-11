//! Exact Ready custody and comparisons for selected required work.

use super::*;

impl SelectedReadyReadmission {
    /// The registry row this Ready belongs to.
    pub(in crate::domain_computation::primary_graph) fn key(&self) -> &WorthQueryOutputDemandKey {
        self.membership.key()
    }

    /// Authenticate the exact pinned Ready, source and membership against the
    /// row selected under the registry guard. No receipt traversal is needed.
    pub(in crate::domain_computation::primary_graph::application_output_demand::registry) fn matches_ready_record(
        &self,
        record: &super::super::super::DemandRecord,
    ) -> bool {
        record
            .work_membership
            .as_ref()
            .is_some_and(|member| Arc::ptr_eq(member, &self.membership))
            && record
                .readmission_source
                .as_ref()
                .is_some_and(|source| Arc::ptr_eq(source, &self.readmission))
            && matches!(&record.state,
                DemandState::Output(output)
                    if matches!(output.advancement, WorthQueryOutputAdvancement::Idle)
                        && matches!(&output.checkpoint,
                            Some(WorthQueryOutputCheckpoint::Ready(current))
                                if current.same_cell(&self.completion)))
    }

    /// Conservative physical-row cycle check for a pending prerequisite walk.
    /// A reopened Ready row may retain this token; reporting a repeat only
    /// defers the walk and never certifies a settlement as current.
    pub(in crate::domain_computation::primary_graph) fn same_record(
        &self,
        other: &Self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(1)
            .map_err(|_| work_denial())?;
        Ok(Arc::ptr_eq(&self.membership, &other.membership))
    }

    /// Compare the selected required row with the caller's actual interest.
    /// The demand key owns the comparison charge for all declared strings.
    pub(in crate::domain_computation::primary_graph) fn matches_interest(
        &self,
        interest: &super::super::super::WorthQueryOutputDemandInterest,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        let comparison_work = self
            .membership
            .key
            .comparison_work()
            .ok_or_else(work_denial)?;
        admission
            .charge_external_work(u64::try_from(comparison_work).map_err(|_| work_denial())?)
            .map_err(|_| work_denial())?;
        Ok(self.membership.key.as_ref() == &interest.key)
    }

    pub(in crate::domain_computation::primary_graph) fn completion(&self) -> &ReadyCompletion {
        &self.completion
    }

    pub(in crate::domain_computation::primary_graph) fn source(
        &self,
    ) -> &(dyn std::any::Any + Send + Sync) {
        self.readmission.source.as_ref()
    }

    pub(in crate::domain_computation::primary_graph) fn limits(
        &self,
    ) -> crate::domain_computation::execution_runtime::WorthQueryOutputDemandLimits {
        self.readmission.limits
    }

    pub(in crate::domain_computation::primary_graph) fn producer_identity(&self) -> &str {
        &self.membership.key.producer
    }

    pub(in crate::domain_computation::primary_graph) fn retained_program_basis(
        &self,
    ) -> Option<&Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>>
    {
        self.readmission.retained_program_basis.as_ref()
    }

    /// A physical row may be reopened with another Ready cell. Member
    /// identity alone cannot authorize moving its successor into the caller.
    pub(in crate::domain_computation::primary_graph) fn same_ready_cell(
        &self,
        other: &Self,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, WorthQueryOutputDemandDenial> {
        admission
            .charge_external_work(3)
            .map_err(|_| work_denial())?;
        Ok(Arc::ptr_eq(&self.membership, &other.membership)
            && Arc::ptr_eq(&self.readmission, &other.readmission)
            && self.completion.same_cell(&other.completion))
    }
}
