//! Sealed test doors over this module's private members.
use super::*;

impl SettlementRegistration {
    #[doc(hidden)]
    pub fn computation_mut_for_test(
        &mut self,
    ) -> &mut Option<super::super::ComputationRegistration> {
        &mut self.computation
    }
}

impl SettlementRegistration {
    #[doc(hidden)]
    pub fn read_basis_mut_for_test(&mut self) -> &mut PositionedRelationalSnapshot {
        &mut self.read_basis
    }
}

impl SettlementRegistration {
    #[doc(hidden)]
    pub fn stale_at_read_basis_mut_for_test(&mut self) -> &mut OrdSet<usize> {
        &mut self.stale_at_read_basis
    }
}

impl SettlementRegistration {
    #[doc(hidden)]
    pub fn new_for_test(
        computation: Option<super::super::ComputationRegistration>,
        work_membership: Option<Arc<RequiredWorkMembership>>,
        identity: Arc<RecordedSettlementIdentity>,
        facts: super::super::super::RetainedSourceFacts,
        output_facts: Option<RegisteredOutputFacts>,
        read_basis: PositionedRelationalSnapshot,
        stale_at_read_basis: OrdSet<usize>,
        requirement: Option<FullVerificationReason>,
        upstream: OrdSet<Arc<RecordedSettlementIdentity>>,
    ) -> Self {
        Self {
            computation,
            work_membership,
            identity,
            facts,
            output_facts,
            read_basis,
            stale_at_read_basis,
            requirement,
            upstream,
        }
    }
}
