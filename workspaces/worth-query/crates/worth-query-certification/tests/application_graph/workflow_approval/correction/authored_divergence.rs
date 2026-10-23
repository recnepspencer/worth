//! The workflow authority projection retains the handler's business dependencies.
use super::*;
use worth_query_host::facade::primary_graph::WorthQueryCorrectionDerivationDenial;

#[test]
fn correction_rejects_authored_dependency_changed_after_approved_source() {
    let (application, instance, _, source) = approved_source();
    assert_eq!(
        settle(set_retention(
            application.program_runtime(),
            instance.branch(),
            9,
            9_192
        )),
        RetentionVerdict::Performed(9)
    );
    let scope = request_scope();
    let principal = authenticate_operator(application.runtime().installed_schema(), &scope);
    let request = application
        .runtime()
        .request(&principal, &scope)
        .on_branch(instance.branch());
    let limits = WorthQueryCorrectionHistoryLimits::new(None, std::num::NonZeroUsize::MIN);
    let selected = request.corrections().select(&source, limits).unwrap();
    assert!(matches!(
        request
            .corrections()
            .reverse(&selected)
            .idempotency("9193")
            .limits(limits)
            .prepare(),
        Err(WorthQueryApplicationCorrectionDenial::Derivation(
            WorthQueryCorrectionDerivationDenial::SourceDiverged
        ))
    ));
    assert_eq!(read_retention(application.runtime(), instance.branch()), 9);
    assert!(matches!(advance_instance(&application, instance, 9_194),
        Ok(WorkflowProgressOutcome::Completed(performed)) if performed.node_path() == "applied"));
}
