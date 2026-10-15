//! Preauthenticated capability admission observes the unwarmed adoption successor.
use super::fixture::{adoption_context_world, request_scope, AUTHORITY, ESTATE, EXECUTOR};
use bank_domain::{
    estate::EstateAction,
    schema::{RecognizeEstateExecutorCapability, RecognizeEstateExecutorOperation},
};
use worth_query_host::facade::{
    application_entry::WorthQueryBranchAdoptionPublicationOutcome,
    primary_graph::{
        WorthQueryOperationAuthorizationDenialKind, WorthQueryOperationAuthorizationRefusalKind,
    },
};

#[test]
fn retained_principal_reads_non_scope_context_on_adoption_successor() {
    let fixture = adoption_context_world("unwarmed-context-successor");
    let principal = fixture.authenticate();
    let runtime = &fixture.runtime;
    let scope = request_scope();
    let target = runtime
        .supported_program_revision::<crate::BankApplicationP1>()
        .unwrap();
    let request = runtime.request(&principal, &scope);
    let programs = request.on_branch(runtime.current_branch()).programs();
    let requirements = programs.compare(&target).unwrap();
    let prepared = programs.adopt(&requirements).unwrap().prepare(128).unwrap();
    assert!(matches!(
        runtime
            .request(&principal, &scope)
            .on_branch(runtime.current_branch())
            .programs()
            .publish(prepared),
        WorthQueryBranchAdoptionPublicationOutcome::Performed(_)
    ));
    let application = runtime.application_runtime();
    let capability = application
        .installed_schema()
        .capability(
            RecognizeEstateExecutorCapability::reference(),
            RecognizeEstateExecutorOperation::reference(),
        )
        .unwrap();
    // Reuse the authentic pre-adoption principal proof. This admission does not
    // resolve a principal or prepare all primary generations on the successor.
    let denial = crate::test_host_policy::with_phase(application, |phase| {
        runtime
            .select_current_product(phase)
            .unwrap()
            .admit_capability_access(
                phase,
                principal.query(),
                &capability,
                EstateAction::RecognizeExecutor {
                    estate: ESTATE,
                    executor: EXECUTOR,
                    authority: AUTHORITY,
                },
                &scope,
            )
    })
    .err()
    .expect("the actual LegalAuthority context must reject self-held authority");
    assert_eq!(
        denial.kind(),
        WorthQueryOperationAuthorizationDenialKind::Refused(
            WorthQueryOperationAuthorizationRefusalKind::SeparationOfDutyRuleMatched
        )
    );
}
