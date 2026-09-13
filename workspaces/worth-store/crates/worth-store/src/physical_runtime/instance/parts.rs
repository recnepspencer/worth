use crate::physical_runtime::{
    lifecycle::LifecycleTerminationGuard,
    record_serving::{
        AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, RecordPublicationDirector,
        RecordServingOwner, RecordWorkAdmission,
    },
    runtime::PhysicalRuntimeCore,
    work::PhysicalWorkAdmissionAuthority,
};

use super::{PhysicalResidencyOwner, PhysicalSchedulerAdmissionOwner, PhysicalStoreWorkRuntime};

/// Exhaustive construction packet for the owners installed in record-serving.
///
/// This remains private to the physical composition root. Adding a managed
/// owner therefore makes construction and terminal destructuring fail until
/// every lifecycle boundary handles it.
pub(in crate::physical_runtime) struct PhysicalStoreInstanceParts {
    pub(in crate::physical_runtime) termination: LifecycleTerminationGuard,
    pub(in crate::physical_runtime) work_admission: PhysicalWorkAdmissionAuthority,
    pub(in crate::physical_runtime) work_runtime: std::sync::Arc<PhysicalStoreWorkRuntime>,
    pub(in crate::physical_runtime) scheduler_admission: PhysicalSchedulerAdmissionOwner,
    pub(in crate::physical_runtime) record_owner: RecordServingOwner,
    pub(in crate::physical_runtime) record_work: std::sync::Arc<RecordWorkAdmission>,
    pub(in crate::physical_runtime) core: PhysicalRuntimeCore,
    pub(in crate::physical_runtime) format: AdmittedPhysicalRecordFormat,
    pub(in crate::physical_runtime) access: AdmittedRecordAccessPolicy,
    pub(in crate::physical_runtime) publication: std::sync::Arc<RecordPublicationDirector>,
    pub(in crate::physical_runtime) checkpoint:
        std::sync::Arc<crate::physical_runtime::durability::PhysicalCheckpointRuntimeOwner>,
    pub(in crate::physical_runtime) root_protocol_counters:
        crate::physical_runtime::RootProtocolRouteCounters,
    pub(in crate::physical_runtime) residency: PhysicalResidencyOwner,
    pub(in crate::physical_runtime) durability:
        crate::physical_runtime::durability::ReopenedPhysicalDurabilityRuntimeOwner,
}
