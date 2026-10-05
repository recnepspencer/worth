use std::sync::atomic::{AtomicU64, Ordering};

use worth_query_installation::facade::{
    ApplicationSchemaBindingIdentity, WorthQueryInstalledGraphObligation,
    WorthQueryInstalledGraphObligationSetIdentity,
};

use crate::domain_computation::execution_resource_admission::{
    WorthQueryCapacityReservedExecutionResourcePlan, WorthQueryExecutionCapacityReleaseReceipt,
    WorthQueryReservedGraphProviderCapacity,
};
use crate::graph_read_access::WorthQueryGraphReadPlanReview;

use super::{WorthQueryGraphWorkIntent, WorthQuerySelectedGraphObligations};

static NEXT_GRAPH_WORK_PLAN_IDENTITY: AtomicU64 = AtomicU64::new(1);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct WorthQueryGraphWorkPlanIdentity(u64);

impl WorthQueryGraphWorkPlanIdentity {
    pub(super) fn mint() -> Option<Self> {
        NEXT_GRAPH_WORK_PLAN_IDENTITY
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_add(1)
            })
            .ok()
            .map(Self)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

pub(super) enum WorthQueryGraphWorkAdmissionMechanics {
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
    ) -> Option<Self> {
        Some(Self {
            identity: WorthQueryGraphWorkPlanIdentity::mint()?,
            selected,
            mechanics,
        })
    }

    pub const fn identity(&self) -> WorthQueryGraphWorkPlanIdentity {
        self.identity
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
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. }
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
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. }
            | WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { .. } => None,
        }
    }

    pub fn reservation_count(&self) -> usize {
        match &self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationQuery { .. } => 1,
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. } => 1,
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
    pub fn take_operation_capacity(
        &mut self,
    ) -> Option<WorthQueryCapacityReservedExecutionResourcePlan> {
        match &mut self.mechanics {
            WorthQueryGraphWorkAdmissionMechanics::ApplicationQuery { .. } => None,
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperationRead { .. } => None,
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
            WorthQueryGraphWorkAdmissionMechanics::ApplicationOperation { capacity } => capacity
                .expect("an untransferred operation plan retains its reservation")
                .release(),
        }
    }
}
