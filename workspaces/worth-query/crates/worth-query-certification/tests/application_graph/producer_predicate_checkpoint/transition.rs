//! A real accepted output must block unsupported checkpoint migration.
use super::*;
use crate::document_retention_model::{host::host_limits, schema::DocumentRetentionSchema};
use worth_query_host::facade::application_installation as installation;

#[test]
fn checkpoint_transition_refuses_real_accepted_output_before_authoring() {
    let program = validated_program();
    let predecessor =
        installation::WorthQueryCheckpointProgramPredecessor::new(&program.revision().to_string())
            .unwrap();
    let host = publish(
        program,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap();
    let runtime = host.runtime();
    let scope = request_scope();
    let principal = authenticate_operator(runtime.installed_schema(), &scope);
    let request = runtime.request(&principal, &scope);
    let mut handle = request
        .demand(RetentionAssessmentDemand::new(DOCUMENT_IDENTITY))
        .start_in_program::<_, AssessmentRoot>(&host)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        handle.settle(&request).unwrap()
    else {
        panic!("actual producer must settle");
    };
    assert!(settled.application_commit_receipt().is_some());
    let (source, sections) = runtime
        .capture_application_checkpoint_with_sections()
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 1);
    let mut called = false;
    let denial = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        validated_program(),
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
        DocumentRetentionSchema::declaration().unwrap(),
        ((),),
        host_limits(),
        source.clone(),
        predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(512, 32, 8192).unwrap(),
        |_, _| {
            called = true;
            Ok(())
        },
    )
    .err()
    .expect("unmapped accepted output must not be silently discarded or recertified");
    assert!(
        !called,
        "accepted-output refusal must precede authoring and native effects"
    );
    assert!(format!("{denial:?}").contains("accepted-output migration support"));
    let restored = restore(
        validated_program(),
        source,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .unwrap();
    assert_eq!(
        restored
            .runtime()
            .capture_application_checkpoint_with_sections()
            .unwrap()
            .1
            .accepted_output_count(),
        1,
        "ordinary restore preserves the original accepted output"
    );
}
