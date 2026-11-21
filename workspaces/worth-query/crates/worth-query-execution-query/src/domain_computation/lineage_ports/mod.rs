//! Domain-computation-level port implementations bridging `worth-query-execution`
//! to `worth-query-execution-lineage` lower ports.

mod query;

use worth_query_execution_lineage::facade::ports::{
    LineageHomeRetirementDenial, WorthQueryReadCleanupRuntime,
};
use worth_query_installation::facade::ApplicationSchema;

use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

impl<Schema: ApplicationSchema> WorthQueryReadCleanupRuntime
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn close_product_publication_recovery_in_advancement(
        &self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        recovery: crate::domain_computation::WorthQueryProductUnpublishedRecovery,
    ) -> Result<
        crate::domain_computation::execution_runtime::product_world::WorthQueryProductBranchOwnerCleanupReceipt,
        crate::domain_computation::WorthQueryProductUnpublishedRecoveryReleaseFailure,
    >{
        WorthQueryPrimaryGraphApplicationRuntime::close_product_publication_recovery_in_advancement(
            self, phase, recovery,
        )
    }
}

impl From<LineageHomeRetirementDenial> for WorthQueryHomeRetirementDenial {
    fn from(denial: LineageHomeRetirementDenial) -> Self {
        match denial {
            LineageHomeRetirementDenial::Busy(owner) => Self::Busy(owner),
            LineageHomeRetirementDenial::Unsupported(owner) => Self::Unsupported(owner),
            LineageHomeRetirementDenial::Handler { binding, cause } => {
                Self::Handler { binding, cause }
            }
            LineageHomeRetirementDenial::NativeAlias(cause) => Self::NativeAlias(cause),
            LineageHomeRetirementDenial::NativeObservation(cause) => Self::NativeObservation(cause),
            LineageHomeRetirementDenial::BridgeBasis(cause) => Self::BridgeBasis(cause),
            LineageHomeRetirementDenial::CanonicalSubscription(cause) => {
                Self::CanonicalSubscription(cause)
            }
            LineageHomeRetirementDenial::Bridge(cause) => Self::Bridge(cause),
            LineageHomeRetirementDenial::Publication(cause) => Self::Publication(cause),
            LineageHomeRetirementDenial::Execution(cause) => Self::Execution(cause),
        }
    }
}
