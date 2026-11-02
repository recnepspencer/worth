//! Recovery call sites reject genuinely installed foreign seals before owner work.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryExternalRedispatchDenial, WorthQueryManagedApplicationRecoveryDenial,
};

#[test]
fn foreign_installed_seal_is_stale_at_redispatch_before_work() {
    let world = installed_authorization_world(true);
    let (handle, mut admission, _) = super::super::recovery_custody::committed_handle(&world);
    let authority = world
        .application
        .admit_recovery_effect_authority(&handle, &admission)
        .unwrap();
    let donor = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    admission.forge_installed_seal_for_test(donor.installed_view());
    world.application.with_host_advancement(|phase| {
        let _observation=crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        fact_work();
        let before=reads();
        let consults=observe_consults_for_test(|_|{});
        let result=world.application.redispatch_admitted_external_effect(&phase,&handle,&authority,&admission);
        let Err(WorthQueryExternalRedispatchDenial::AdmissionAuthorization(kind))=result else { panic!("redispatch must check its own installed seal: {result:?}"); };
        assert_eq!(kind,refused(AuthorizationRefusal::StaleInstalledOperation));
        assert_eq!(reads(),before);
        assert_eq!(fact_work().source_admission_visits,0);
        assert_eq!(consult_count_for_test(&consults),0);
        assert!(charged_prefixes_for_test(&consults).is_empty());
    }).unwrap();
}

#[test]
fn foreign_installed_seal_is_stale_at_managed_recovery_before_work() {
    let world = installed_authorization_world(true);
    let request = live_scope();
    let (principal, account) = party(&world, &request);
    let program = world
        .application
        .with_host_advancement(|phase| {
            crate::domain_computation::primary_graph::tests::application_attempt::admitted_program(
                &phase, &world, &principal, &account, &request, "closed",
            )
        })
        .unwrap();
    let mut admission = admit!(world, &principal, &account, TouchAccountOperation, &request);
    let recovery =
        super::super::unwind_custody::retained_recovery_after_application_unwind(&world, || {
            world.application.compare_and_commit_application(
                program,
                idempotency(188, 189),
                crate::facade::runtime::ExecutionAllocationPolicy::SystemAllocation,
            )
        });
    let donor = world
        .application
        .installed_schema()
        .installed_operation(ProgramRequiredOperation::reference())
        .unwrap();
    admission.forge_installed_seal_for_test(donor.installed_view());
    world.application.with_host_advancement(|phase| {
        let _observation=crate::facade::primary_graph::WorthQueryExecutionObservation::install_on_this_thread();
        fact_work();
        let before=reads();
        let consults=observe_consults_for_test(|_|{});
        let result=world.application.recover_admitted_unpublished_application_in_advancement(
            &phase,&recovery,&admission,idempotency(188,189));
        let Err(WorthQueryManagedApplicationRecoveryDenial::Authorization(denial))=result else { panic!("managed recovery must check its own installed seal"); };
        assert_eq!(denial.kind(),refused(AuthorizationRefusal::StaleInstalledOperation));
        assert_eq!(reads(),before);
        assert_eq!(fact_work().source_admission_visits,0);
        assert_eq!(consult_count_for_test(&consults),0);
        assert!(charged_prefixes_for_test(&consults).is_empty());
    }).unwrap();
    world
        .application
        .release_product_publication_recovery(recovery, 0)
        .expect("the original refused recovery is still owned and releasable");
}
