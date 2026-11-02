use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligation,
    WorthQueryInstalledGraphObligationSetIdentity,
};

use crate::domain_computation::execution_resource_admission::{
    WorthQueryCapacityReservedExecutionResourcePlan, WorthQueryExecutionCapacityReleaseReceipt,
    WorthQueryReservedGraphProviderCapacity,
};
use crate::graph_read_access::WorthQueryGraphReadPlanReview;

use super::{
    WorthQueryGraphWorkAdmissionDenial, WorthQueryGraphWorkIntent,
    WorthQuerySelectedGraphObligations,
};

mod identity;
pub use identity::WorthQueryGraphWorkPlanIdentity;

pub(super) enum WorthQueryGraphWorkAdmissionMechanics {
    ApplicationInspectionRead {
        capacity: WorthQueryReservedGraphProviderCapacity,
    },
    ManagedIndexRead {
        capacity: WorthQueryReservedGraphProviderCapacity,
    },
    ResourceRead {
        capacity: WorthQueryReservedGraphProviderCapacity,
    },
    ApplicationQuery {
        review: WorthQueryGraphReadPlanReview,
        capacity: WorthQueryReservedGraphProviderCapacity,
    },
    ApplicationOperationRead {
        capacity: WorthQueryReservedGraphProviderCapacity,
    },
    ApplicationOperation {
        capacity: Option<WorthQueryCapacityReservedExecutionResourcePlan>,
    },
}

/// Move-only, capacity-reserved admission authority.
///
/// ```compile_fail
/// use worth_query_admission::facade::graph_obligation::WorthQueryAdmittedGraphWorkPlan;
/// let forged = WorthQueryAdmittedGraphWorkPlan { identity: todo!(), selected: todo!(), mechanics: todo!() };
/// ```
pub struct WorthQueryAdmittedGraphWorkPlan {
    identity: WorthQueryGraphWorkPlanIdentity,
    selected: WorthQuerySelectedGraphObligations,
    mechanics: WorthQueryGraphWorkAdmissionMechanics,
}

impl WorthQueryAdmittedGraphWorkPlan {
    pub(super) fn seal(
        selected: WorthQuerySelectedGraphObligations,
        mechanics: WorthQueryGraphWorkAdmissionMechanics,
    ) -> Result<Self, WorthQueryGraphWorkAdmissionDenial> {
        Ok(Self {
            identity: WorthQueryGraphWorkPlanIdentity::mint()?,
            selected,
            mechanics,
        })
    }

    pub const fn identity(&self) -> WorthQueryGraphWorkPlanIdentity {
        self.identity
    }

    /// Readonly provenance of the exact capacity reservation held by this plan.
    /// Execution consumes this across the crate boundary when binding a report
    /// to its live inspection session, before consuming the admitted plan.
    #[doc(hidden)]
    pub fn inspection_capacity_identity(&self) -> Option<crate::domain_computation::execution_resource_admission::WorthQueryGraphCapacityReservationIdentity>{
        match &self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { capacity } => {
                Some(capacity.identity())
            }
            _ => None,
        }
    }

    pub fn obligation_identity(&self) -> &WorthQueryInstalledGraphObligationSetIdentity {
        self.selected.identity()
    }

    #[doc(hidden)]
    pub fn binding_identity(&self) -> &ApplicationSchemaBindingIdentity {
        self.selected.binding_identity()
    }

    pub const fn intent(&self) -> WorthQueryGraphWorkIntent {
        self.selected.intent()
    }

    pub fn graph_read_review(&self) -> Option<&WorthQueryGraphReadPlanReview> {
        match &self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationQuery { review, .. } => Some(review),
            WorthQueryGraphWorkAdmissionMechanics::ManagedIndexRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ResourceRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { .. } => None,
        }
    }

    /// Completes one admitted application-query graph-work reservation.
    ///
    /// Kept behind the integration surface so only the execution-owned
    /// session terminal can turn an admitted plan into completion evidence.
    #[doc(hidden)]
    pub fn complete_application_query(
        self,
    ) -> Option<(
        WorthQueryGraphWorkPlanIdentity,
        WorthQueryGraphReadPlanReview,
        WorthQueryExecutionCapacityReleaseReceipt,
    )> {
        let Self {
            identity,
            mechanics,
            ..
        } = self;
        match mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationQuery { review, capacity } => {
                Some((identity, review, capacity.release()))
            }
            WorthQueryGraphWorkAdmissionMechanics::ManagedIndexRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ResourceRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { .. } => None,
        }
    }

    /// Consumes only the genuine ResourceRead reservation at its session terminal.
    #[doc(hidden)]
    pub fn complete_application_resource(
        self,
    ) -> Option<(
        WorthQueryGraphWorkPlanIdentity,
        WorthQueryExecutionCapacityReleaseReceipt,
    )> {
        match self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ResourceRead { capacity } => {
                Some((self.identity, capacity.release()))
            }
            _ => None,
        }
    }

    /// Consume only the genuine metadata-inspection reservation at its terminal.
    #[doc(hidden)]
    pub fn complete_application_inspection(
        self,
    ) -> Option<(
        WorthQueryGraphWorkPlanIdentity,
        WorthQueryExecutionCapacityReleaseReceipt,
    )> {
        match self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { capacity } => {
                Some((self.identity, capacity.release()))
            }
            _ => None,
        }
    }

    /// Consumes only a genuine managed-index reservation at its own terminal.
    #[doc(hidden)]
    pub fn complete_managed_index(
        self,
    ) -> Option<(
        WorthQueryGraphWorkPlanIdentity,
        WorthQueryExecutionCapacityReleaseReceipt,
    )> {
        match self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ManagedIndexRead { capacity } => {
                Some((self.identity, capacity.release()))
            }
            _ => None,
        }
    }

    pub fn reservation_count(&self) -> usize {
        match &self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationQuery { .. } => 1,
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. } => 1,
            WorthQueryGraphWorkAdmissionMechanics::ResourceRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ManagedIndexRead { .. } => 1,
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { capacity } => {
                capacity.as_ref().map_or(
                    0,
                    WorthQueryCapacityReservedExecutionResourcePlan::reservation_count,
                )
            }
        }
    }

    #[doc(hidden)]
    pub fn required_obligations(&self) -> &[WorthQueryInstalledGraphObligation] {
        self.selected.rows()
    }

    #[doc(hidden)]
    pub fn operation_resources(
        &self,
    ) -> Option<&crate::domain_computation::execution_resource_admission::WorthQueryAdmittedExecutionResourcePlan>{
        match &self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { capacity } => capacity
                .as_ref()
                .map(WorthQueryCapacityReservedExecutionResourcePlan::resources),
            _ => None,
        }
    }

    #[doc(hidden)]
    pub fn take_operation_capacity(
        &mut self,
    ) -> Option<WorthQueryCapacityReservedExecutionResourcePlan> {
        match &mut self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationQuery { .. } => None,
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. } => None,
            WorthQueryGraphWorkAdmissionMechanics::ResourceRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ManagedIndexRead { .. } => None,
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { capacity } => {
                capacity.take()
            }
        }
    }

    #[doc(hidden)]
    pub fn release(self) -> WorthQueryExecutionCapacityReleaseReceipt {
        match self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationQuery { capacity, .. } => {
                capacity.release()
            }
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { capacity } => {
                capacity.release()
            }
            WorthQueryGraphWorkAdmissionMechanics::ResourceRead { capacity }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationInspectionRead { capacity }
            | WorthQueryGraphWorkAdmissionMechanics::ManagedIndexRead { capacity } => {
                capacity.release()
            }
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { capacity } => capacity
                .expect("an untransferred operation plan retains its reservation")
                .release(),
        }
    }
}
