pub(crate) use worth_query_testkit_certification_application_graph::facade::certification::application_graph::adoption::rostered_restore::*;

#[test]
fn a_restored_host_keeps_each_branch_on_the_program_it_adopted() {
    let directory = unique_directory("rostered-restore");
    let host = open_directory_on_first_program(directory);
    let source = *host.installed_program().revision();
    let target = *host
        .supported_program::<RetentionProgramP1>()
        .expect("P1 is rostered")
        .installed_program()
        .revision();
    {
        let main = host.current_world();
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
    }
    let directory = host.checkpoint_and_close_for_reopen();

    let restored = open_directory_on_first_program(directory);
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
