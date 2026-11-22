//! Port implementations bridging `worth-query-execution` primary-graph types to
//! `worth-query-execution-runtime` lower ports.

use std::any::Any;
use std::sync::{Arc, Mutex};
use worth_query_execution_lineage::facade::ports::ApplicationOutputLineageInternalPort as _;

use worth_query_execution_runtime::facade::execution_runtime::product_world::WorthQueryProductRuntime;
use worth_query_execution_runtime::facade::ports::{
    ForkLineagePort, InspectionTargetPort, ProductRuntimeRef, RuntimeHomeRetirementDenial,
};
use worth_query_installation::facade::{
    ApplicationSchema, ApplicationSchemaBindingIdentity,
    WorthQueryInstalledApplicationInspectionContract,
};
use worth_runtime_world::facade::ProductBranchObservation;

use crate::domain_computation::primary_graph::causal_inspection::application_inspection::WorthQueryApplicationInspectionTarget;
use crate::domain_computation::primary_graph::output_lineage::WorthQueryApplicationOutputLineage;
use crate::domain_computation::primary_graph::{
    WorthQueryHomeRetirementDenial, WorthQueryPrimaryGraphApplicationRuntime,
};

impl InspectionTargetPort for WorthQueryApplicationInspectionTarget {
    fn contract(&self) -> &WorthQueryInstalledApplicationInspectionContract {
        WorthQueryApplicationInspectionTarget::contract(self)
    }

    fn schema_binding(&self) -> &ApplicationSchemaBindingIdentity {
        WorthQueryApplicationInspectionTarget::schema_binding(self)
    }

    fn runtime_authority(&self) -> u64 {
        WorthQueryApplicationInspectionTarget::runtime_authority(self)
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

impl<Schema: ApplicationSchema> ProductRuntimeRef
    for WorthQueryPrimaryGraphApplicationRuntime<Schema>
{
    fn product_runtime(&self) -> &WorthQueryProductRuntime {
        &self.runtime.product_runtime
    }
}

pub struct OutputLineageForkAdapter(pub Arc<Mutex<WorthQueryApplicationOutputLineage>>);

impl ForkLineagePort for OutputLineageForkAdapter {
    fn register_fork(
        &self,
        source: &ProductBranchObservation,
        destination: &ProductBranchObservation,
    ) {
        self.0
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .register_fork(source, destination);
    }
}

pub use worth_query_execution_attempt::facade::domain_computation::runtime_ports::PrimaryGraphProviderArc;

impl From<RuntimeHomeRetirementDenial> for WorthQueryHomeRetirementDenial {
    fn from(value: RuntimeHomeRetirementDenial) -> Self {
        match value {
            RuntimeHomeRetirementDenial::Busy(owner) => Self::Busy(owner),
            RuntimeHomeRetirementDenial::Unsupported(owner) => Self::Unsupported(owner),
            RuntimeHomeRetirementDenial::NativeAlias(cause) => Self::NativeAlias(cause),
            RuntimeHomeRetirementDenial::NativeObservation(cause) => Self::NativeObservation(cause),
            RuntimeHomeRetirementDenial::BridgeBasis(cause) => Self::BridgeBasis(cause),
            RuntimeHomeRetirementDenial::CanonicalSubscription(cause) => {
                Self::CanonicalSubscription(cause)
            }
            RuntimeHomeRetirementDenial::Bridge(cause) => Self::Bridge(cause),
            RuntimeHomeRetirementDenial::Publication(cause) => Self::Publication(cause),
            RuntimeHomeRetirementDenial::Execution(cause) => Self::Execution(cause),
        }
    }
}
