//! Shared authored inputs and composition of the actual Home recovery stages.
//! This fixture builds no runtime beside the native owner returned by Home.
//! Its fixed genuine Query binding proves descriptor admission only. The
//! independently supplied native definition owns its native schema; its cases
//! do not prove Query-program continuation or expose an application service.
//! Host composition must retain its actual installed schema/program/roster;
//! descriptor digests cannot recreate that authority.
use std::path::PathBuf;
use worth_execution::{ExecutionRequest, MapKernelStop, WorkCeilingDenial};
use worth_store_application::{
    admit_home, readmit_relational, recover_home, ApplicationHomeOpenRefusal,
    ApplicationHomeRequest, ReadmittedApplicationHome,
};

pub fn request(
    location: impl Into<PathBuf>,
    native: worth_relational::facade::runtime::RelationalRuntimeBuilder,
) -> ApplicationHomeRequest {
    let binding = worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding();
    let build = worth_foundational::facade::ApplicationBuildIdentity::parse(
        "11111111-1111-4111-8111-111111111111",
    )
    .unwrap();
    ApplicationHomeRequest::new(location.into(), native.resolve(), binding, build)
}

pub fn open(
    request: ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, ApplicationHomeOpenRefusal> {
    open_through(execution, || recover_home(request, execution))
}

fn open_through(
    execution: &ExecutionRequest<'_, '_>,
    recover: impl FnOnce() -> Result<
        worth_proof::StoreRecovered<worth_store_application::RecoveredPhysicalHome>,
        ApplicationHomeOpenRefusal,
    >,
) -> Result<ReadmittedApplicationHome, ApplicationHomeOpenRefusal> {
    let mut opened = None;
    let scope = execution.in_scope(|_| {
        opened = Some((|| {
            let recovered = recover()?;
            let admitted = admit_home(recovered, execution)?;
            readmit_relational(admitted, execution).map(|proof| proof.consume(|home| home))
        })());
    });
    match (scope, opened) {
        (Ok(()), Some(result)) => result,
        (Err(WorkCeilingDenial::Stopped(MapKernelStop::NestedStopped)), Some(Err(cause))) => {
            Err(cause)
        }
        (Err(cause), _) => Err(ApplicationHomeOpenRefusal::Execution(cause)),
        (Ok(()), None) => unreachable!("completed original-request fixture opening"),
    }
}

pub fn open_with_media(
    request: ApplicationHomeRequest,
    media: worth_store::physical_runtime::FilesystemMediaAdmission,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, ApplicationHomeOpenRefusal> {
    open(request.with_media_for_test(media), execution)
}

pub fn open_with_descriptor_observer(
    request: ApplicationHomeRequest,
    media: worth_store::physical_runtime::FilesystemMediaAdmission,
    observer: impl FnOnce(&worth_store::physical_runtime::ServingPhysicalRuntime),
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, ApplicationHomeOpenRefusal> {
    open_through(execution, || {
        request.recover_with_descriptor_observer_for_test(media, observer, execution)
    })
}

pub fn open_with_readmission_observer(
    request: ApplicationHomeRequest,
    observer: impl Fn(
            worth_relational::facade::publication::PatchStreamPosition,
            worth_store_relational::RelationalStoreColdReadCounters,
        ) + Send
        + Sync
        + 'static,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, ApplicationHomeOpenRefusal> {
    open(
        request.observe_relational_reads_for_test(observer),
        execution,
    )
}

pub fn open_with_recovery_yieldpoint(
    request: ApplicationHomeRequest,
    point: worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, ApplicationHomeOpenRefusal> {
    open(request.with_recovery_yieldpoint_for_test(point), execution)
}
