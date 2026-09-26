//! A host restores the roster it installed, so each branch in a checkpoint
//! keeps running the program it adopted.

use worth_query_host::facade::application_entry::{
    WorthQueryApplicationRequestExt, WorthQueryBranchAdoptionPublicationOutcome,
};
use worth_query_host::facade::application_installation::WorthQueryApplicationProgramRoster;

use crate::bounded_dimension_model::host::{publish_on_first_program, restore_on_first_program};
use crate::bounded_dimension_model::operator_identity::{authenticate_operator, request_scope};
use crate::bounded_dimension_model::programs::{validated_second_program, DimensionProgramP1};

#[test]
fn a_restored_host_keeps_each_branch_on_the_program_it_adopted() {
    let host = publish_on_first_program();
    let main = host.current_world();
    let source = host.installed_program().revision().clone();
    let target = host
        .supported_program::<DimensionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision()
        .clone();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let programs = host
        .runtime()
        .request(&principal, &scope)
        .on_branch(main)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    match programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(64)
        .expect("the seed satisfies P1")
        .publish()
    {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_) => {}
        _ => panic!("main adopts P1"),
    }
    let checkpoint = host
        .runtime()
        .capture_application_checkpoint()
        .expect("the adopted world captures");
    drop(host);

    let restored = restore_on_first_program(
        checkpoint,
        WorthQueryApplicationProgramRoster::new().support(validated_second_program()),
    )
    .expect("the host restores under the roster it installed");
    assert_eq!(restored.installed_program().revision(), &source);
    let selected = restored
        .runtime()
        .on_branch(restored.current_world())
        .select()
        .expect("the restored world is selectable");
    assert_eq!(
        selected.inspect_selected_program().unwrap().revision(),
        &target,
        "the restored branch still runs the program it adopted",
    );
}
