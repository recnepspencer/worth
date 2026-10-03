use std::sync::Arc;

use super::PreparedStableLineagePublication;
use crate::domain_computation::primary_graph::output_lineage::invalidation::{
    collect_consumed_output_upstream, InvalidationEditAdmission,
    PreparedCurrentSettlementRegistration, SettlementRegistration, SettlementRegistrationStop,
    SourceInvalidationOwner,
};
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

impl<'lane, 'selected> PreparedStableLineagePublication<'lane, 'selected> {
    /// Prepares the current native source image for the already sealed alias.
    /// This shares the ordinary publication's consumed-output collection and
    /// creates no separate source or settlement authority.
    pub(in crate::domain_computation::primary_graph) fn prepare_current_registration(
        &self,
        owner: &SourceInvalidationOwner,
        work_membership: Option<Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>>,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        let selected = self.address.verified.selected;
        let branch_bytes = u64::try_from(selected.branch_id().0.len()).map_err(|_| {
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow
        })?;
        admission.charge_external_work(branch_bytes.checked_add(3).ok_or(
            worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
        )?)?;
        admission.admit_read_scratch(branch_bytes)?;
        let upstream = collect_consumed_output_upstream(self.consumed_outputs(), admission)?;
        // The selected candidate may itself be an alias. This sealed relation
        // names that exact predecessor, never just the performed origin.
        let equality = StableEqualityConsequence {
            predecessor: Arc::clone(self.address.verified.candidate.settlement_identity()),
            successor: Arc::clone(&self.address.identity),
            selected,
        };
        owner.prepare_current_stable_settlement(
            SettlementRegistration {
                work_membership,
                identity: Arc::clone(&self.address.identity),
                facts: Arc::clone(&self.facts),
                output_facts: None,
                read_basis: selected.clone(),
                requirement: None,
                upstream,
            },
            selected,
            equality,
            admission,
        )
    }
}

/// Minted only by a verified stable publication at its exact selected source.
pub(in crate::domain_computation::primary_graph) struct StableEqualityConsequence<'selected> {
    predecessor: Arc<RecordedSettlementIdentity>,
    successor: Arc<RecordedSettlementIdentity>,
    selected: &'selected worth_relational::facade::runtime::PositionedRelationalSnapshot,
}

impl<'selected> StableEqualityConsequence<'selected> {
    #[cfg(test)]
    pub(in crate::domain_computation::primary_graph) fn native_actor_fixture(
        predecessor: Arc<RecordedSettlementIdentity>,
        successor: Arc<RecordedSettlementIdentity>,
        selected: &'selected worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Self {
        Self {
            predecessor,
            successor,
            selected,
        }
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn predecessor(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.predecessor
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn successor(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.successor
    }

    pub(in crate::domain_computation::primary_graph::output_lineage) fn selected(
        &self,
    ) -> &'selected worth_relational::facade::runtime::PositionedRelationalSnapshot {
        self.selected
    }
}
