//! current output reports an obsolete source after authoritative retirement.
use super::*;

#[test]
fn current_output_reports_an_obsolete_source_after_authoritative_retirement() {
    let world = installed_authorization_world(true);
    let obsolete = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .invariant
                .project(
                    &phase,
                    |reader| {
                        reader
                            .resolve_entity(AccountStatus::reference(), "open".to_owned())
                            .unwrap()
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
                .into_parts()
                .0
        })
        .expect("the fixture read owns its installed host call");
    world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            retire_open_account(&phase, &world)
        })
        .expect("the fixture read owns its installed host call");

    let request = live_scope();
    let external = world.authenticate("alice", Duration::from_secs(60), &request);
    let selected = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world.selected_product(&(phase).test_host_request())
        })
        .expect("the fixture read owns its installed host call");
    let principal = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
                .resolve_authenticated_principal(
                    &phase,
                    &world.binding,
                    &external,
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let surviving_scope = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
                .resolve_entity(
                    &phase,
                    AccountStatus::reference(),
                    "unrelated".to_owned(),
                    &request,
                    WorthQueryPrincipalResolutionMode::Ordinary,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    let operation = world
        .application
        .installed_schema()
        .installed_operation(TouchAccountOperation::reference())
        .unwrap();
    let admission = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            selected
                .authorize_operation(
                    &phase,
                    &principal,
                    &surviving_scope,
                    &operation,
                    Default::default(),
                    &request,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");

    let completed = world
        .application
        .with_host_advancement(|phase| {
            let host_custody = phase.test_host_request();
            world
                .invariant
                .project_admitted_operation(
                    &phase,
                    &admission,
                    |reader, _| {
                        reader.current_output::<TestCurrentOutputFamily, Account>(&obsolete)
                    },
                    worth_execution::ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap()
        })
        .expect("the fixture read owns its installed host call");
    assert!(matches!(
        completed.output(),
        Ok(WorthQueryCurrentOutputSelection::ObsoleteSource)
    ));
}
