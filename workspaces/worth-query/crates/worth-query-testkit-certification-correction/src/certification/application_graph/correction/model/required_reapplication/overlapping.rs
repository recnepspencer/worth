//! Different admitted writers cannot choose a winner for one field.
use super::*;
pub(super) fn run(worker: bool) {
    let application = if worker {
        host::install_with_output_limits(host::worker_limits())
    } else {
        host::install_with_outputs()
    };
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let request = application.request(&principal, &scope);
    let before = head(&application);
    let before_source = host::read(&application);
    let calculated = read_calculation(&application);
    let result = request
        .advanced()
        .mutate(Edit {
            key: "numeric".into(),
            value: Value::Calculated(12),
        })
        .without_source()
        .idempotency(&"overlapping-author-and-producer".to_owned())
        .execute_performed::<OutputProgram, Root>(
            &application,
            ExecutionAllocationPolicy::SystemAllocation,
        )
        .unwrap();
    let denial = match result {
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(
            WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
                denial,
            )),
        ) => denial,
        WorthQueryApplicationPerformedMutationOutcome::NotPerformed(other) => {
            panic!("overlapping admitted field writes must refuse; actual={other:?}")
        }
        WorthQueryApplicationPerformedMutationOutcome::Performed(source) => panic!(
            "overlapping admitted field writes must refuse; actually performed {:?}",
            source.receipt().commit_reference()
        ),
        _ => panic!("overlapping admitted field writes must refuse; unfinished output custody"),
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::ProviderRejected
    );
    assert!(
        denial
            .detail()
            .is_some_and(|detail| detail.contains("conflicting entity intent for slot")),
        "the conflict must name the actual shared entity: {denial:?}"
    );
    assert_eq!(
        head(&application),
        before,
        "overlapping writes have no effect"
    );
    assert_eq!(host::read(&application), before_source);
    assert_eq!(read_calculation(&application), calculated);
}
