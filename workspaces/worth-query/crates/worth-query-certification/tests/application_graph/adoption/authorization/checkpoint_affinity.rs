//! The original native malformed checkpoint still belongs to the current P0/schema.
use super::*;
use crate::document_retention_model::{
    host::restore_on_first_program,
    programs::{validated_first_program, validated_second_program},
};
use worth_query_host::facade::application_installation::{
    WorthQueryApplicationCheckpoint, WorthQueryApplicationProgramRoster,
};
#[test]
fn captured_native_missing_input_checkpoint_retains_final_p0_schema_and_program_affinity() {
    let bytes = include_bytes!("../../workflow_definition_readmission/missing_input.checkpoint");
    assert_eq!(bytes.len(), 299_162);
    // Native restore validates installed schema and roster affinity. The malformed
    // input remains for the joined workflow owner's cold readmission court.
    let host = restore_on_first_program(
        WorthQueryApplicationCheckpoint::from_untrusted_bytes(bytes.to_vec()),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the authentic checkpoint must match the final canonical P0 and schema");
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let inspection = host
        .runtime()
        .request(&principal, &scope)
        .programs()
        .inspect()
        .unwrap();
    assert_eq!(inspection.revision(), validated_first_program().revision());
    assert_eq!(read_retention(host.runtime(), host.current_world()), 7);
}
