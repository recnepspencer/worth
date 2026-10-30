//! Real native predecessor -> target World -> ordinary target reopen journey.
#[path = "checkpoint_transition/admission.rs"]
pub mod admission;
#[path = "checkpoint_transition/funded_capture.rs"]
pub mod funded_capture;
#[path = "checkpoint_transition/original_request.rs"]
pub mod original_request;
#[path = "checkpoint_transition/predecessor.rs"]
pub mod predecessor;
pub use super::*;
pub use application_installation::{
    WorthQueryCheckpointProgramPredecessor as Predecessor,
    WorthQueryCheckpointTransitionResources as Resources,
};
pub use predecessor::{configuration, source, LegacyTemporalProgram};
pub use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
pub use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;
pub fn transition(
    checkpoint: application_installation::WorthQueryApplicationCheckpoint,
    predecessor: Predecessor,
    resources: Resources,
    author: impl FnOnce(
        &mut application_installation::WorthQueryCheckpointMigrationWriter<'_, TemporalHostSchema>,
        &domain::WorthQueryInstalledApplicationSchema<TemporalHostSchema>,
    ) -> Result<(), primary_graph::WorthQueryPrimaryGraphInstallationDenial>,
) -> Result<
    application_installation::WorthQueryProgramApplicationRuntime<
        TemporalHostSchema,
        TemporalInstallationProgram,
    >,
    application_installation::WorthQueryInMemoryApplicationDenial,
> {
    application_installation::in_memory_rostered_program_from_checkpoint_with_transition(
        validated_program(),
        application_installation::WorthQueryApplicationProgramRoster::new(),
        TemporalHostSchema::declaration().unwrap(),
        configuration(),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        checkpoint::checkpoint_limits(),
        checkpoint,
        predecessor,
        resources,
        CapturePolicy::SystemAllocation,
        author,
    )
}

pub fn source_predecessor() -> Predecessor {
    let program = ApplicationProgramAuthoring::<TemporalHostSchema, LegacyTemporalProgram>::begin()
        .validated_program()
        .unwrap();
    Predecessor::new(&program.revision().to_string()).unwrap()
}
