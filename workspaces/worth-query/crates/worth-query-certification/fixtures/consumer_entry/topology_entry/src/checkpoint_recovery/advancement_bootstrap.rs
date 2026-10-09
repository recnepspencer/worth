//! The real installation owns preparation, seed delivery and publication together.
use super::*;
use worth_query_host::facade::{
    application_contribution::WorthQueryManagedComputationResourceDenial as Resource,
    primary_graph::{
        advancement_requests_on_this_thread_for_test as reports,
        installed_source_reads_on_this_thread_for_test as reads,
    },
};

#[test]
fn bootstrap_opens_once_before_its_registry_reader_and_seeding() {
    let _guard = checkpoint_recovery_test_guard();
    reports();
    let before = reads();
    let application = install(None);
    assert!(
        reads() > before,
        "the admitted installation entered the real reader"
    );
    assert_eq!(
        reports().len(),
        1,
        "preparation, seeding and publication borrow one root"
    );
    let checkpoint = application
        .runtime()
        .capture_application_checkpoint()
        .unwrap();
    reports();
    let before = reads();
    drop(install(Some(checkpoint)));
    assert!(
        reads() > before,
        "checkpoint installation enters the same source reader"
    );
    assert_eq!(
        reports().len(),
        1,
        "recovery reads borrow the installation root too"
    );
    for restored in [false, true] {
        for zero_memory in [false, true] {
            let policy = worth_foundational::ExecutionRequestPolicy::new(
                worth_foundational::ExecutionPosture::Serial,
                worth_foundational::DeterminismContract::CanonicalBitwise,
                worth_foundational::ExecutionBudget::new(
                    NonZeroUsize::MIN,
                    if zero_memory { 0 } else { 64 * 1_024 * 1_024 },
                    if zero_memory { 8_000_000 } else { 0 },
                ),
            );
            let limits = support::limits_with_policy(
                32,
                16,
                64,
                support::invalidation(128 * 1_024 * 1_024, 1_000_000, 32),
                support::candidates(),
                policy,
            );
            let checkpoint = restored.then(|| {
                application
                    .runtime()
                    .capture_application_checkpoint()
                    .unwrap()
            });
            let before = reads();
            reports();
            let mut seeds = 0;
            let denial = support::try_install_program_with_limits::<CheckpointProgram>(
                checkpoint,
                Default::default(),
                limits,
                |_| {
                    seeds += 1;
                },
            )
            .err()
            .expect("the declared bootstrap policy refuses");
            let application_installation::WorthQueryInMemoryApplicationDenial::Graph(denial) =
                *denial
            else {
                panic!("the installation retains request admission");
            };
            use worth_query_host::facade::primary_graph::WorthQueryProviderSessionDenialKind;
            let primary_graph::WorthQueryPrimaryGraphInstallationDenialKind::ExecutionDenied {
                kind:
                    WorthQueryProviderSessionDenialKind::ExecutionResource {
                        denial: cause,
                        partition_identity,
                        policy_ancestor,
                    },
            } = denial.kind()
            else {
                panic!("the typed cause reaches the builder");
            };
            assert_eq!(
                cause,
                if zero_memory {
                    Resource::PolicyMemoryLimit
                } else {
                    Resource::WorkExhausted
                }
            );
            assert_eq!(partition_identity, None);
            assert_eq!(policy_ancestor, None);
            assert_eq!(reads(), before);
            assert_eq!(seeds, 0);
            assert_eq!(reports().len(), 1);
        }
    }
}
