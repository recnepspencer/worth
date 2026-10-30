//! Opaque public transport and complete installed-program readmission.
use super::{
    host::{self, Host, Program},
    schema::Schema,
};
use worth_query_decl::facade::{application_program::*, application_schema::*};
use worth_query_host::facade::application_installation::*;

pub fn capture(host: &Host) -> Box<[u8]> {
    let started = std::time::Instant::now();
    let checkpoint = host
        .runtime()
        .with_host_advancement(|phase| {
            host.runtime()
                .capture_application_checkpoint(
                    &phase,
                    WorthQueryCheckpointCapturePolicy::SystemAllocation,
                )
                .expect("the native cell application captures")
        })
        .expect("explicit host checkpoint maintenance opens");
    println!(
        "cell cold capture_us={} opaque_checkpoint_payload_bytes={}",
        started.elapsed().as_micros(),
        checkpoint.bytes().len()
    );
    checkpoint.bytes().to_vec().into_boxed_slice()
}

pub fn restore(bytes: Box<[u8]>, retained_steps: u64) -> Host {
    let started = std::time::Instant::now();
    let host = in_memory_program_from_checkpoint(
        ApplicationProgramAuthoring::<Schema, Program>::begin()
            .validated_program()
            .unwrap(),
        Schema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("16161616-1616-1616-1616-161616161616").unwrap(),
        host::limits(retained_steps),
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes),
    )
    .expect("the complete native cell checkpoint readmits through its owner");
    println!("cell cold restore_us={}", started.elapsed().as_micros());
    host
}
