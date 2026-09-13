use super::{admission, configuration, markers, operation_program, Invocation};
use worth_store::physical_runtime::PhysicalRecordInitialization;
use worth_store::physical_runtime::{
    AdmittedPhysicalRecordFormat, AdmittedRecordAccessPolicy, AdmittedRecordPlacementPolicy,
    ServingPhysicalRuntime,
};

pub(super) struct InitializedWriter {
    pub(super) operation_program: operation_program::C8OperationProgram,
    pub(super) serving: ServingPhysicalRuntime,
    pub(super) format: AdmittedPhysicalRecordFormat,
    pub(super) placement: AdmittedRecordPlacementPolicy,
}

pub(super) fn initialize(invocation: &Invocation) -> Result<InitializedWriter, String> {
    std::fs::create_dir_all(&invocation.root)
        .map_err(|error| format!("create C8 writer root: {error}"))?;
    let operation_program = operation_program::read(&invocation.operation_program)?;
    let (format, placement, access) = configuration::record_configuration();
    let serving = start_serving(
        &invocation.root,
        format,
        placement,
        access,
        invocation.writer_durability_profile,
    )?;
    markers::write_runtime_identity(
        &invocation.start_marker,
        serving.runtime_identity().get().to_string(),
    )?;
    Ok(InitializedWriter {
        operation_program,
        serving,
        format,
        placement,
    })
}

fn start_serving(
    root: &std::path::Path,
    format: AdmittedPhysicalRecordFormat,
    placement: AdmittedRecordPlacementPolicy,
    access: AdmittedRecordAccessPolicy,
    profile: super::durability_profile::WriterDurabilityProfile,
) -> Result<ServingPhysicalRuntime, String> {
    let media = admission::admit_media(root)?;
    let durability = admission::admit_durability(&media, profile)?;
    admission::require_serving(
        media.initialize_record_store(PhysicalRecordInitialization::new(
            format, placement, access, durability,
        )),
        "C8 production writer record-store initialization",
    )
}
