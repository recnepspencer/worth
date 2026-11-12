use std::sync::Arc;

use super::PreparedStableLineagePublication;
use crate::domain_computation::primary_graph::output_lineage::invalidation::collect_consumed_output_upstream;
use crate::domain_computation::primary_graph::output_lineage::invalidation::PreparedCurrentSettlementRegistration;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistration;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop;
use crate::domain_computation::primary_graph::output_lineage::invalidation::SourceInvalidationOwner;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

impl<'lane, 'selected> PreparedStableLineagePublication<'lane, 'selected> {
    pub(in crate::domain_computation::primary_graph) fn prepare_current_registration(
        &self,
        owner: &SourceInvalidationOwner,
        work_membership: Option<Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>>,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        self.prepare_registration(owner, work_membership, None)
    }

    pub(in crate::domain_computation::primary_graph) fn prepare_executed_registration(
        &self,
        owner: &SourceInvalidationOwner,
        adoption: &worth_relational::facade::mvcc::RelationalCurrentOutputAdoption,
        work_membership: Option<Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>>,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        self.prepare_registration(owner, work_membership, Some(adoption))
    }

    /// Both postures retain the same consumed facts and output availability.
    /// Only actual Native execution can establish a row without an old mark.
    fn prepare_registration(
        &self,
        owner: &SourceInvalidationOwner,
        work_membership: Option<Arc<crate::domain_computation::primary_graph::application_output_demand::RequiredWorkMembership>>,
        adoption: Option<&worth_relational::facade::mvcc::RelationalCurrentOutputAdoption>,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        let selected = self.address.selected;

        let upstream = collect_consumed_output_upstream(self.consumed_outputs())?;
        // The selected candidate may itself be an alias. This sealed relation
        // names that exact predecessor, never just the performed origin.
        let equality = StableEqualityConsequence {
            partition: Some(self.address.partition),
            predecessor: Arc::clone(self.address.candidate.settlement_identity()),
            successor: Arc::clone(&self.address.identity),
            selected,
        };
        let registration = SettlementRegistration {
                computation: self.recorded.completed_handler_facts.as_ref().and_then(|_| {
                    let row = self.recorded.mutable.lock()
                        .unwrap_or_else(std::sync::PoisonError::into_inner);
                    matches!(row.computation,
                        super::super::super::retained_computation::RecordedComputation::Retained(_))
                        .then(super::super::super::invalidation::ComputationRegistration::retained)
                }),
                work_membership,
                identity: Arc::clone(&self.address.identity),
                facts: crate::domain_computation::primary_graph::output_lineage::RetainedSourceFacts::retain(self.recorded.computation_source, self.facts.clone()),
                output_facts: None,
                read_basis: selected.clone(),
                stale_at_read_basis: im::OrdSet::new(),
                requirement: None,
                upstream,
            };
        match adoption {
            Some(adoption) => owner.prepare_current_executed_settlement(
                registration,
                selected,
                equality,
                adoption,
            ),
            None => owner.prepare_current_stable_settlement(registration, selected, equality),
        }
    }
}

/// Minted only by a verified stable publication at its exact selected source.
pub struct StableEqualityConsequence<'selected> {
    partition: Option<[u8; 32]>,
    predecessor: Arc<RecordedSettlementIdentity>,
    successor: Arc<RecordedSettlementIdentity>,
    selected: &'selected worth_relational::facade::runtime::PositionedRelationalSnapshot,
}

impl<'selected> StableEqualityConsequence<'selected> {
    pub(in crate::domain_computation::primary_graph) fn native_actor_fixture(
        predecessor: Arc<RecordedSettlementIdentity>,
        successor: Arc<RecordedSettlementIdentity>,
        selected: &'selected worth_relational::facade::runtime::PositionedRelationalSnapshot,
    ) -> Self {
        Self {
            partition: None,
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

    pub(in crate::domain_computation::primary_graph::output_lineage) fn partition(
        &self,
    ) -> Option<[u8; 32]> {
        self.partition
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

mod court_doors_for_test;
