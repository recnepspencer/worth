//! Fresh recovery entry admission preserves original custody and one request.
use super::*;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationMutationOutcome as OrdinaryOutcome,
    WorthQueryApplicationPerformedMutationOutcome as RequiredOutcome,
    WorthQueryApplicationRecoveryRequestDenial as RecoveryDenial,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationUncommitted, WorthQueryManagedApplicationRecoveryDenial as NativeDenial,
};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy as Allocation;

fn zero_work() {
    let policy = support::CHECKPOINT_EXECUTION_POLICY.budget();
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        policy.charged_memory_bytes(),
        0,
    )));
}
fn opening_refused(denial: RecoveryDenial, before: u64) {
    assert!(
        matches!(
            denial,
            RecoveryDenial::Recovery(NativeDenial::ExecutionDenied(Denial::Resource(
                Resource::WorkExhausted
            )))
        ),
        "{denial:?}"
    );
    assert_eq!(reads(), before, "zero work precedes readers");
    assert_eq!(reports().len(), 1, "one refused request per call");
    bound(None);
}
fn one_admitted_request(entry: &str) {
    let opened = reports();
    eprintln!("{entry}: {opened:?}");
    assert_eq!(opened.len(), 1, "one admitted request per call");
    assert!(
        opened[0].is_ok(),
        "the enclosing execution owner completed: {opened:?}"
    );
}
macro_rules! fresh {
    ($request:expr, $source:expr, $key:expr) => {
        $request
            .mutate(PlanarSourceAdjustment {
                scope_key: "anchor-a".into(),
                replacement_y: length(2),
            })
            .expect_source($source.clone())
            .idempotency($key)
    };
}
macro_rules! ordinary {
    ($request:expr, $source:expr, $key:expr) => {
        $request
            .mutate(crate::PlanarEdit(crate::PlanarMutation {
                scope_key: "anchor-a".into(),
                operation: worth_query_consumer_values::PlanarOperation::Adjust(vec![
                    worth_query_consumer_values::PlanarAdjustment {
                        body_key: "anchor-a".into(),
                        replacement_y: length(2),
                    },
                ]),
            }))
            .expect_source($source.clone())
            .idempotency($key)
    };
}
macro_rules! source {
    ($request:expr) => {
        $request
            .query(PlanarRead {
                body_key: "anchor-a".into(),
            })
            .execute()
            .unwrap()
            .observed_sources()[0]
            .clone()
    };
}
#[test]
fn unpublished_recovery_opens_before_authorization_for_all_three_entries() {
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let application = install(None);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let source = source!(request);
        application.fail_next_durable_append_for_test();
        let OrdinaryOutcome::Commit(WorthQueryApplicationUncommitted::ProductUnpublished(partial)) =
            ordinary!(request, source, &9130_u64)
                .execute_in_program(&application, Allocation::SystemAllocation)
                .unwrap()
        else {
            panic!("real durable append refusal retains ordinary recovery")
        };
        let recovery = partial.into_recovery();
        zero_work();
        reports();
        let before = reads();
        opening_refused(
            ordinary!(request, source, &9130_u64)
                .recover_unpublished_in_program(&recovery, &application)
                .err()
                .unwrap(),
            before,
        );
        reports();
        ordinary!(request, source, &9130_u64)
            .recover_unpublished_in_program(&recovery, &application)
            .unwrap();
        assert!(reads() > before, "admitted call reaches its readers");
        one_admitted_request("recover_unpublished_in_program");

        let application = support::install_program::<DiscoveredProgram>(None, Default::default());
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let source = source!(request);
        application.fail_next_durable_append_for_test();
        let Outcome::ProductUnpublished(mut recovery) = fresh!(request, source, &9131_u64)
            .execute_performed_discovered::<DiscoveredProgram, DiscoveredRoot>(
                &application,
                Allocation::SystemAllocation,
            )
            .unwrap()
        else {
            panic!("real durable append refusal retains discovered recovery")
        };
        zero_work();
        reports();
        let before = reads();
        opening_refused(
            fresh!(request, source, &9131_u64)
                .recover_unpublished_discovered_in_program(&mut recovery, &application)
                .err()
                .unwrap(),
            before,
        );
        reports();
        fresh!(request, source, &9131_u64)
            .recover_unpublished_discovered_in_program(&mut recovery, &application)
            .unwrap();
        assert!(reads() > before, "admitted call reaches its readers");
        one_admitted_request("recover_unpublished_discovered_in_program");

        let application = install(None);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let source = source!(request);
        application.fail_next_durable_append_for_test();
        let RequiredOutcome::ProductUnpublished(mut recovery) = fresh!(request, source, &9132_u64)
            .execute_performed::<CheckpointProgram, CheckpointRoot>(
                &application,
                Allocation::SystemAllocation,
            )
            .unwrap()
        else {
            panic!("real durable append refusal retains required recovery")
        };
        zero_work();
        reports();
        let before = reads();
        opening_refused(
            fresh!(request, source, &9132_u64)
                .recover_unpublished_required_in_program(&mut recovery, &application)
                .err()
                .unwrap(),
            before,
        );
        reports();
        fresh!(request, source, &9132_u64)
            .recover_unpublished_required_in_program(&mut recovery, &application)
            .unwrap();
        assert!(reads() > before, "admitted call reaches its readers");
        one_admitted_request("recover_unpublished_required_in_program");
    }
}
#[test]
fn idempotency_opens_before_selected_authorization_and_lookup() {
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let application = install(None);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let source = source!(request);
        zero_work();
        reports();
        let before = reads();
        opening_refused(
            ordinary!(request, source, &9133_u64)
                .resolve_idempotency_in_program(&application)
                .err()
                .unwrap(),
            before,
        );
        reports();
        assert!(matches!(
            ordinary!(request, source, &9133_u64)
                .resolve_idempotency_in_program(&application)
                .unwrap()
                .into_resolution(),
            primary_graph::WorthQueryApplicationIdempotencyResolution::Unseen
        ));
        assert!(reads() > before, "admitted call reaches its readers");
        one_admitted_request("resolve_idempotency_in_program");
    }
}
#[test]
fn discovered_promotion_opens_before_its_receipt_reader_and_returns_custody_on_refusal() {
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let application = support::install_program::<DiscoveredProgram>(None, Default::default());
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let source = source!(request);
        application.fail_next_durable_append_for_test();
        let Outcome::ProductUnpublished(mut recovery) = fresh!(request, source, &9134_u64)
            .execute_performed_discovered::<DiscoveredProgram, DiscoveredRoot>(
                &application,
                Allocation::SystemAllocation,
            )
            .unwrap()
        else {
            panic!("native unpublished custody")
        };
        fresh!(request, source, &9134_u64)
            .recover_unpublished_discovered_in_program(&mut recovery, &application)
            .unwrap();
        let controls = WorthQueryOutputDemandControls::new(
            NonZeroUsize::new(4096).unwrap(),
            NonZeroUsize::new(8192).unwrap(),
        );
        zero_work();
        reports();
        let before = reads();
        let (denial, recovery) = fresh!(request, source, &9134_u64)
            .promote_recovered_discovered_outputs(recovery, &application, controls)
            .err()
            .unwrap();
        opening_refused(denial, before);
        reports();
        fresh!(request, source, &9134_u64)
            .promote_recovered_discovered_outputs(recovery, &application, controls)
            .unwrap_or_else(|(denial, _)| panic!("{denial:?}"));
        assert!(reads() > before, "admitted call reaches its readers");
        one_admitted_request("promote_recovered_discovered_outputs");
    }
}
#[test]
fn required_promotion_opens_before_its_receipt_reader_and_returns_custody_on_refusal() {
    let _guard = checkpoint_recovery_test_guard();
    for placement in [Placement::Serial, Placement::Leased(NonZeroUsize::MIN)] {
        let _restore = Restore(place(placement), bound(None));
        let application = install(None);
        let (scope, principal) = authenticate(&application);
        let request = application.request(&principal, &scope);
        let source = source!(request);
        application.fail_next_durable_append_for_test();
        let RequiredOutcome::ProductUnpublished(mut recovery) = fresh!(request, source, &9135_u64)
            .execute_performed::<CheckpointProgram, CheckpointRoot>(
                &application,
                Allocation::SystemAllocation,
            )
            .unwrap()
        else {
            panic!("native unpublished custody")
        };
        fresh!(request, source, &9135_u64)
            .recover_unpublished_required_in_program(&mut recovery, &application)
            .unwrap();
        zero_work();
        reports();
        let before = reads();
        let (denial, recovery) = fresh!(request, source, &9135_u64)
            .promote_recovered_required_outputs(recovery, &application)
            .err()
            .unwrap();
        opening_refused(denial, before);
        reports();
        fresh!(request, source, &9135_u64)
            .promote_recovered_required_outputs(recovery, &application)
            .unwrap_or_else(|(denial, _)| panic!("{denial:?}"));
        assert!(reads() > before, "admitted call reaches its readers");
        one_admitted_request("promote_recovered_required_outputs");
    }
}

mod cancellation;
