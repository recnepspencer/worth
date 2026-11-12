//! Source registration which cannot advance an earlier cutoff proof by replay.

use std::sync::Arc;

use worth_relational::facade::mvcc::CompanionCellEditStop;
use worth_relational::facade::runtime::PositionedRelationalSnapshot;

use super::PreparedSettlementRegistration;
use super::SettlementRegistration;
use super::SettlementRegistrationCleanup;
use super::SettlementRegistrationStop;
use super::SourceInvalidationOwner;
use super::StoppedSettlementRegistration;
use super::{RegistrationOutputBasis, RegistrationReadBasis};
use crate::domain_computation::primary_graph::output_lineage::input_cutoff::StableEqualityConsequence;
use crate::domain_computation::primary_graph::output_lineage::RecordedSettlementIdentity;

/// Bound to the exact source root checked by the cutoff, and to the new
/// lineage address whose marks this source image will install.
pub struct PreparedCurrentSettlementRegistration<'selected> {
    prepared: PreparedSettlementRegistration,
    identity: Arc<RecordedSettlementIdentity>,
    selected: &'selected PositionedRelationalSnapshot,
}

pub struct CurrentSettlementRegistrationCleanup {
    _prepared: SettlementRegistrationCleanup,
    _identity: Arc<RecordedSettlementIdentity>,
}

pub struct StoppedCurrentSettlementRegistration<'selected> {
    stopped: StoppedSettlementRegistration,
    identity: Arc<RecordedSettlementIdentity>,
    selected: &'selected PositionedRelationalSnapshot,
}

impl<'selected> PreparedCurrentSettlementRegistration<'selected> {
    pub(in crate::domain_computation::primary_graph) fn identity(
        &self,
    ) -> &Arc<RecordedSettlementIdentity> {
        &self.identity
    }

    pub(in crate::domain_computation::primary_graph) fn selected_source(
        &self,
    ) -> &'selected PositionedRelationalSnapshot {
        self.selected
    }

    pub(in crate::domain_computation::primary_graph) fn install(
        self,
    ) -> Result<CurrentSettlementRegistrationCleanup, StoppedCurrentSettlementRegistration<'selected>>
    {
        match self.prepared.install() {
            Ok(prepared) => Ok(CurrentSettlementRegistrationCleanup {
                _prepared: prepared,
                _identity: self.identity,
            }),
            Err(stopped) => Err(StoppedCurrentSettlementRegistration {
                stopped,
                identity: self.identity,
                selected: self.selected,
            }),
        }
    }
}

impl<'selected> StoppedCurrentSettlementRegistration<'selected> {
    pub(in crate::domain_computation::primary_graph) fn into_parts(
        self,
    ) -> (
        CompanionCellEditStop,
        PreparedCurrentSettlementRegistration<'selected>,
    ) {
        let (reason, prepared) = self.stopped.into_parts();
        (
            reason,
            PreparedCurrentSettlementRegistration {
                prepared,
                identity: self.identity,
                selected: self.selected,
            },
        )
    }
}

impl SourceInvalidationOwner {
    pub(in crate::domain_computation::primary_graph) fn prepare_current_settlement<'selected>(
        &self,
        registration: SettlementRegistration,
        selected: &'selected PositionedRelationalSnapshot,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        if registration.read_basis != *selected {
            return Err(SettlementRegistrationStop::Foreign);
        }
        let identity = Arc::clone(&registration.identity);
        let prepared = self.prepare_settlement_at_basis(
            registration,
            RegistrationReadBasis::CurrentOnly,
            RegistrationOutputBasis::Ordinary,
            None,
        )?;
        Ok(PreparedCurrentSettlementRegistration {
            prepared,
            identity,
            selected,
        })
    }

    /// Only the sealed stable-publication relation may discharge an old
    /// consumed identity in the same prepared current source image.
    pub(in crate::domain_computation::primary_graph) fn prepare_current_stable_settlement<
        'selected,
    >(
        &self,
        registration: SettlementRegistration,
        selected: &'selected PositionedRelationalSnapshot,
        equality: StableEqualityConsequence<'selected>,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        if registration.read_basis != *selected
            || !std::ptr::eq(selected, equality.selected())
            || !Arc::ptr_eq(&registration.identity, equality.successor())
        {
            return Err(SettlementRegistrationStop::Foreign);
        }
        let identity = Arc::clone(&registration.identity);
        let prepared = self.prepare_settlement_at_basis(
            registration,
            RegistrationReadBasis::CurrentOnly,
            RegistrationOutputBasis::VerifiedCutoff(&equality),
            equality.partition(),
        )?;
        Ok(PreparedCurrentSettlementRegistration {
            prepared,
            identity,
            selected,
        })
    }

    /// Actual executed no-effect evidence supplies fresh source and live output
    /// facts. Unlike cutoff reuse, it does not require an old settlement mark.
    pub(in crate::domain_computation::primary_graph) fn prepare_current_executed_settlement<
        'selected,
    >(
        &self,
        registration: SettlementRegistration,
        selected: &'selected PositionedRelationalSnapshot,
        equality: StableEqualityConsequence<'selected>,
        adoption: &worth_relational::facade::mvcc::RelationalCurrentOutputAdoption,
    ) -> Result<PreparedCurrentSettlementRegistration<'selected>, SettlementRegistrationStop> {
        let basis = adoption.basis();
        if registration.read_basis != *selected
            || !std::ptr::eq(selected, equality.selected())
            || !Arc::ptr_eq(&registration.identity, equality.successor())
            || selected.runtime_instance_id() != basis.runtime_instance_id()
            || selected.branch_id() != basis.branch_id()
            || selected.root_id() != basis.root_identity()
            || registration.identity.source().scope !=
                crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(adoption.meaning().source_scope)
            || !registration.facts.source().is_executed_adoption()
        {
            return Err(SettlementRegistrationStop::Foreign);
        }
        let identity = Arc::clone(&registration.identity);
        let prepared = self.prepare_settlement_at_basis(
            registration,
            RegistrationReadBasis::CurrentOnly,
            RegistrationOutputBasis::ExecutedAdoption(&equality),
            equality.partition(),
        )?;
        Ok(PreparedCurrentSettlementRegistration {
            prepared,
            identity,
            selected,
        })
    }
}

mod court_doors_for_test;
