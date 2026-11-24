//! Crate-boundary internal ports for `WorthQueryProductBranchLease` and
//! `WorthQueryProductObservationLease`. Never exported on
//! `worth-query-execution::facade` or Host.

use std::sync::Arc;

use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_runtime_bridge::facade::RelationalBridgeObservationLease;
use worth_runtime_world::facade::ProductBranchObservation;
use worth_signal::facade::branch::AdmittedSignalBranchBasis;

use crate::domain_computation::execution_runtime::product_world::{
    WorthQueryProductPublicationBinding, WorthQuerySelectedProgramInspection,
};

use super::super::WorthQueryProductBranchAdmissionDenial;
use super::{WorthQueryProductBranchLease, WorthQueryProductObservationLease};

pub trait ProductObservationLeaseInternalPort: Sized {
    fn from_observation(observation: ProductBranchObservation) -> Self;
    fn suspended_security(
        observation: ProductBranchObservation,
        current_security_guard: ProductBranchObservation,
    ) -> Self;
    fn retained_clone(&self) -> Self;
    fn observation(&self) -> &ProductBranchObservation;
    fn current_security_guard(&self) -> Option<&ProductBranchObservation>;
    fn relational_basis(&self) -> &AdmittedRelationalBranchBasis;
}

impl ProductObservationLeaseInternalPort for WorthQueryProductObservationLease {
    fn from_observation(observation: ProductBranchObservation) -> Self {
        Self::new(observation)
    }

    fn suspended_security(
        observation: ProductBranchObservation,
        current_security_guard: ProductBranchObservation,
    ) -> Self {
        Self::suspended_security(observation, current_security_guard)
    }

    fn retained_clone(&self) -> Self {
        Self::retained_clone(self)
    }

    fn observation(&self) -> &ProductBranchObservation {
        Self::observation(self)
    }

    fn current_security_guard(&self) -> Option<&ProductBranchObservation> {
        Self::current_security_guard(self)
    }

    fn relational_basis(&self) -> &AdmittedRelationalBranchBasis {
        Self::relational_basis(self)
    }
}

pub trait ProductBranchLeaseInternalPort: Sized {
    fn retain_selected_program_source(
        &mut self,
        selected: &WorthQuerySelectedProgramInspection,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial>;
    fn carry_selected_program_source_from(
        &mut self,
        prior: &Self,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial>;
    fn read_lease(&self) -> WorthQueryProductObservationLease;
    fn into_read_lease(self) -> WorthQueryProductObservationLease;
    fn read_lease_ref(&self) -> &WorthQueryProductObservationLease;
    fn retained_clone(&self) -> Self;
    fn shares_retained_mutation_source(&self, other: &Self) -> bool;
    fn observation(&self) -> &ProductBranchObservation;
    fn relational_basis(&self) -> &AdmittedRelationalBranchBasis;
    fn bridge_source(&self) -> Arc<RelationalBridgeObservationLease>;
    fn bridge_source_observation(&self) -> &RelationalBridgeObservationLease;
    fn signal_basis(&self) -> &AdmittedSignalBranchBasis;
    fn publication_binding(&self) -> WorthQueryProductPublicationBinding;
}

impl ProductBranchLeaseInternalPort for WorthQueryProductBranchLease {
    fn retain_selected_program_source(
        &mut self,
        selected: &WorthQuerySelectedProgramInspection,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial> {
        Self::retain_selected_program_source(self, selected)
    }

    fn carry_selected_program_source_from(
        &mut self,
        prior: &Self,
    ) -> Result<(), WorthQueryProductBranchAdmissionDenial> {
        Self::carry_selected_program_source_from(self, prior)
    }

    fn read_lease(&self) -> WorthQueryProductObservationLease {
        Self::read_lease(self)
    }

    fn into_read_lease(self) -> WorthQueryProductObservationLease {
        Self::into_read_lease(self)
    }

    fn read_lease_ref(&self) -> &WorthQueryProductObservationLease {
        Self::read_lease_ref(self)
    }

    fn retained_clone(&self) -> Self {
        Self::retained_clone(self)
    }

    fn shares_retained_mutation_source(&self, other: &Self) -> bool {
        Self::shares_retained_mutation_source(self, other)
    }

    fn observation(&self) -> &ProductBranchObservation {
        Self::observation(self)
    }

    fn relational_basis(&self) -> &AdmittedRelationalBranchBasis {
        Self::relational_basis(self)
    }

    fn bridge_source(&self) -> Arc<RelationalBridgeObservationLease> {
        Self::bridge_source(self)
    }

    fn bridge_source_observation(&self) -> &RelationalBridgeObservationLease {
        Self::bridge_source_observation(self)
    }

    fn signal_basis(&self) -> &AdmittedSignalBranchBasis {
        Self::signal_basis(self)
    }

    fn publication_binding(&self) -> WorthQueryProductPublicationBinding {
        Self::publication_binding(self)
    }
}
