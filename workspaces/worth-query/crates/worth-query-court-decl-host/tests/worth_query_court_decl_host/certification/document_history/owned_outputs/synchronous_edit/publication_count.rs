//! Count real Native publications even when another branch's head is selected.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_relational::facade::inspection::observe_maintenance_work;

fn run(worker: bool) {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = if worker {
        world::install_worker_only()
    } else {
        world::install()
    };
    let head = host
        .with_host_advancement(|phase| {
            let selected = host.on_branch(host.current_world()).select(&phase).unwrap();
            host.branches()
                .fork_expected(selected)
                .unwrap()
                .components(|parts| parts.fork_relational().reuse_exact_signal_basis())
                .create(&phase)
                .unwrap()
                .into_owned_head()
        })
        .unwrap();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    let (_, publications) = observe_maintenance_work(1, || {
        let first = request
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 2,
            })
            .without_source()
            .idempotency(&"count-main-publication".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            first,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        let main = courts::native_head(&host);
        let second = request
            .on_branch(head.branch())
            .advanced()
            .mutate(Intent {
                key: "a".into(),
                value: 3,
            })
            .without_source()
            .idempotency(&"count-side-publication".to_owned())
            .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        assert!(matches!(
            second,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ));
        assert_ne!(
            first.receipt().unwrap().commit_reference(),
            second.receipt().unwrap().commit_reference()
        );
        assert_eq!(
            courts::native_head(&host),
            main,
            "head ancestry cannot see the side-branch publication"
        );
        assert_eq!(world::model(&host, host.current_world()).value, 2);
        assert_eq!(world::model(&host, head.branch()).value, 3);
        let main_outputs = world::outputs(&host, host.current_world());
        let side_outputs = world::outputs(&host, head.branch());
        assert_eq!((main_outputs.0.value, main_outputs.1.value), (20, 1002));
        assert_eq!((side_outputs.0.value, side_outputs.1.value), (30, 1003));
    });
    assert_eq!(publications.published_rows.len(), 1);
    assert_eq!(
        publications.omitted_publications, 1,
        "a bounded recorder still counts the second branch's commit"
    );
    assert_eq!(
        publications.published_rows.len() + publications.omitted_publications,
        2
    );
}

#[test]
fn publication_counter_includes_an_edit_on_another_branch() {
    run(false);
}

#[test]
fn worker_only_publication_counter_includes_an_edit_on_another_branch() {
    run(true);
}
