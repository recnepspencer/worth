//! Shared authored inputs and composition of the actual Home recovery stages.
//! This fixture builds no runtime beside the native owner returned by Home.
//! Its fixed genuine Query binding proves descriptor admission only. The
//! independently supplied native definition owns its native schema; its cases
//! do not prove Query-program continuation or expose an application service.
//! Host composition must retain its actual installed schema/program/roster;
//! descriptor digests cannot recreate that authority.
use std::path::PathBuf;
use worth_execution::{ExecutionRequest, WorkCeilingDenial};
use worth_store_application::{
    admit_home, readmit_relational, recover_home, ApplicationHomeOpenRefusal,
    ApplicationHomeRequest, ReadmittedApplicationHome,
};

/// The fixture retains the actual most advanced source owner on refusal.
pub enum HomeOpeningRefused {
    BeforeReadmission(ApplicationHomeOpenRefusal),
    Readmission(worth_store_application::RelationalHomeOpenRefused),
    UnrecoveredClose(worth_store_application::UnrecoveredApplicationHomeCloseRefused),
    Close(worth_store_application::ApplicationHomeCloseOutcome),
    Admitted {
        home: ReadmittedApplicationHome,
        cause: WorkCeilingDenial,
    },
}
impl HomeOpeningRefused {
    /// Borrow the actual source cause while this error keeps its real owner.
    pub fn source_cause(&self) -> Option<&ApplicationHomeOpenRefusal> {
        match self {
            Self::BeforeReadmission(cause) => Some(cause),
            Self::Readmission(worth_store_application::RelationalHomeOpenRefused::Readmission(
                refused,
            )) => Some(refused.reason()),
            _ => None,
        }
    }
}
impl std::fmt::Debug for HomeOpeningRefused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BeforeReadmission(cause) => {
                f.debug_tuple("BeforeReadmission").field(cause).finish()
            }
            Self::Readmission(home) => f.debug_tuple("Readmission").field(home).finish(),
            Self::UnrecoveredClose(home) => f.debug_tuple("UnrecoveredClose").field(home).finish(),
            Self::Close(worth_store_application::ApplicationHomeCloseOutcome::Closed {
                ..
            }) => f.write_str("Closed"),
            Self::Close(worth_store_application::ApplicationHomeCloseOutcome::CloseRefused {
                reason,
                ..
            }) => f.debug_tuple("CloseRefused").field(reason).finish(),
            Self::Admitted { home, cause } => f
                .debug_struct("Admitted")
                .field("home", &home.home_identity())
                .field("cause", cause)
                .finish(),
        }
    }
}

pub fn request(
    location: impl Into<PathBuf>,
    native: worth_relational::facade::runtime::RelationalRuntimeBuilder,
) -> ApplicationHomeRequest {
    request_with_build(
        location,
        native,
        worth_foundational::facade::ApplicationBuildIdentity::parse(
            "11111111-1111-4111-8111-111111111111",
        )
        .unwrap(),
    )
}
/// The genuine Home binding is unchanged; only implementation identity varies.
pub fn request_with_build(
    location: impl Into<PathBuf>,
    native: worth_relational::facade::runtime::RelationalRuntimeBuilder,
    build: worth_foundational::facade::ApplicationBuildIdentity,
) -> ApplicationHomeRequest {
    let binding=worth_query_testkit_installation::facade::installation::application_home_binding::admitted_home_binding();
    ApplicationHomeRequest::new(location.into(), native.resolve(), binding, build)
}

pub fn open(
    request: ApplicationHomeRequest,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
    open_through(execution, || recover_home(request, execution))
}

fn open_through(
    execution: &ExecutionRequest<'_, '_>,
    recover: impl FnOnce() -> Result<
        worth_proof::StoreRecovered<worth_store_application::RecoveredPhysicalHome>,
        ApplicationHomeOpenRefusal,
    >,
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
    let mut opened = None;
    let scope = execution.in_scope(|_| {
        opened = Some((|| {
            let recovered = recover().map_err(HomeOpeningRefused::BeforeReadmission)?;
            let admitted =
                admit_home(recovered, execution).map_err(HomeOpeningRefused::BeforeReadmission)?;
            readmit_relational(admitted, execution)
                .map(|proof| proof.consume(|home| home))
                .map_err(HomeOpeningRefused::Readmission)
        })());
    });
    match (scope, opened) {
        (Ok(()), Some(result)) => result,
        (Err(_), Some(Err(refused))) => Err(refused),
        (Err(cause), Some(Ok(home))) => Err(HomeOpeningRefused::Admitted { home, cause }),
        (Err(cause), None) => Err(HomeOpeningRefused::BeforeReadmission(
            ApplicationHomeOpenRefusal::Execution(cause),
        )),
        (Ok(()), None) => unreachable!("completed original-request fixture opening"),
    }
}

pub fn open_with_media(
    request: ApplicationHomeRequest,
    media: worth_store::physical_runtime::FilesystemMediaAdmission,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
    open(request.with_media_for_test(media), execution)
}

pub fn open_with_descriptor_observer(
    request: ApplicationHomeRequest,
    media: worth_store::physical_runtime::FilesystemMediaAdmission,
    observer: impl FnOnce(&worth_store::physical_runtime::ServingPhysicalRuntime),
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
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
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
    open(
        request.observe_relational_reads_for_test(observer),
        execution,
    )
}

pub fn open_with_recovery_yieldpoint(
    request: ApplicationHomeRequest,
    point: worth_store::physical_runtime::PhysicalRecoveryProcessYieldpoint,
    execution: &ExecutionRequest<'_, '_>,
) -> Result<ReadmittedApplicationHome, HomeOpeningRefused> {
    open(request.with_recovery_yieldpoint_for_test(point), execution)
}
