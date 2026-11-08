//! Opaque public transport and complete installed-program readmission.
use super::{
    host::{self, Host, Program},
    schema::Schema,
};
use worth_query_decl::facade::application_program::*;
use worth_query_host::facade::application_installation::*;

pub fn capture(host: &Host) -> WorthQueryApplicationCheckpoint {
    host.runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .expect("the native cell application captures")
        })
        .expect("explicit host checkpoint maintenance opens")
}

pub fn restore(checkpoint: WorthQueryApplicationCheckpoint, retained_steps: u64) -> Host {
    WorthQueryInterimCheckpointRestore::program(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("16161616-1616-1616-1616-161616161616").unwrap(),
        host::limits(retained_steps),
        checkpoint,
    )
    .expect("the complete native cell checkpoint readmits through its owner")
}
