//! Sealed test doors over this module's private members.
use super::*;

impl ErasedClockObservationOutcome {
    #[doc(hidden)]
    pub fn typed_for_test<Clock>(
        self,
        installation: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation,
        source_read_basis: Option<
            crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis,
        >,
    ) -> super::super::WorthQueryConditionalClockObservationOutcome<Clock> {
        Self::typed::<Clock>(self, installation, source_read_basis)
    }
}

impl ErasedClockObservationReceipt {
    #[doc(hidden)]
    pub fn typed_for_test<Clock>(
        self,
        granular_invalidation_installation: crate::domain_computation::primary_graph::WorthQueryGranularInvalidationInstallation,
        granular_source_read_basis: Option<
            crate::domain_computation::primary_graph::WorthQueryGranularSourceReadBasis,
        >,
    ) -> super::super::WorthQueryConditionalClockObservationReceipt<Clock> {
        Self::typed::<Clock>(
            self,
            granular_invalidation_installation,
            granular_source_read_basis,
        )
    }
}
