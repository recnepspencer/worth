//! A host restores the roster it installed, so each branch in a checkpoint
//! keeps running the program it adopted.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;
use worth_query_host::facade::application_installation::WorthQueryCheckpointCapturePolicy as CapturePolicy;

use crate::document_retention_model::host::{publish_on_first_program, restore_on_first_program};
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use crate::document_retention_model::programs::{validated_second_program, RetentionProgramP1};

#[test]
fn a_restored_host_keeps_each_branch_on_the_program_it_adopted() {
    let host = publish_on_first_program();
    let main = host.current_world();
    let source = *host.installed_program().revision();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(main)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    match programs
        .adopt(&requirements)
        .expect("declared adoption capability admits")
        .prepare(64)
        .expect("the seed satisfies P1")
        .publish(host.runtime(), &principal, &scope)
    {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {}
        _ => panic!("main adopts P1"),
    }
    let checkpoint = host
        .runtime()
        .with_host_advancement(|host_phase| {
            host.runtime()
                .capture_application_checkpoint(&host_phase, CapturePolicy::SystemAllocation)
                .expect("the adopted world captures")
        })
        .expect("installed host call opens");
    drop(host);

    let restored = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    assert_eq!(restored.installed_program().revision(), &source);
    let selected = restored
        .with_host_advancement(|host_phase| {
            restored
                .runtime()
                .on_branch(restored.current_world())
                .select(&host_phase)
                .expect("the restored world is selectable")
        })
        .expect("installed host call opens");
    assert_eq!(
        selected.inspect_selected_program().unwrap().revision(),
        &target,
        "the restored branch still runs the program it adopted",
    );
}
