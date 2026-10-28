//! Recovery entry admission opens one request before authorization and lookup.
use super::*;
use worth_query_host::facade::application_entry::WorthQueryApplicationRecoveryRequestDenial as RecoveryDenial;
use worth_query_host::facade::primary_graph::WorthQueryManagedApplicationRecoveryDenial as NativeDenial;

fn zero_work() {
    let policy = support::observation::CHECKPOINT_EXECUTION_POLICY
        .accounting()
        .expect("this fixture declares accounting");
    bound(Some(worth_foundational::ExecutionBudget::new(
        NonZeroUsize::MIN,
        policy.charged_memory_bytes(),
        Some(0),
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
macro_rules! ordinary {
    ($request:expr, $source:expr, $key:expr) => {
        $request
            .advanced()
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

mod cancellation;
