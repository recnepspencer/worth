use super::*;

#[derive(Debug, Clone, Copy)]
enum OverflowMeter {
    Work,
    PreparationBytes,
}

#[derive(Debug)]
struct OverflowCompanion(OverflowMeter);

impl RelationalPublicationCompanion for OverflowCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        match self.0 {
            OverflowMeter::Work => {
                context.claim_work(u64::MAX)?;
                context.claim_work(1)?;
            }
            OverflowMeter::PreparationBytes => {
                context.claim_bytes(u64::MAX)?;
                context.claim_bytes(1)?;
            }
        }
        panic!("an overflowing preflight charge must stop before a companion effect exists");
    }
}

#[test]
fn preflight_counter_overflow_denies_without_moving_the_native_head() {
    for meter in [OverflowMeter::Work, OverflowMeter::PreparationBytes] {
        let runtime = runtime_with_test_schema();
        let before = test_owner_main_basis(&runtime).expect("main branch has owner basis");
        let pending = runtime
            .publication_companion_port()
            .begin_required_registration()
            .expect("required registration starts");
        let _registration = pending
            .activate(
                Arc::new(OverflowCompanion(meter)),
                CompanionPreflightBudget {
                    maximum_work_visits: u64::MAX,
                    maximum_preparation_bytes: u64::MAX,
                },
            )
            .expect("overflow fixture is active");
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                batch_create("overflow-stops-before-movement"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("candidate stages");
        let candidate = runtime
            .prepare_branch_transaction(
                transaction,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("candidate prepares");
        let outcome = runtime.publication_port().compare_and_publish(candidate);
        let expected = match meter {
            OverflowMeter::Work => CompanionPreflightStop::WorkCounterOverflow,
            OverflowMeter::PreparationBytes => {
                CompanionPreflightStop::PreparationMemoryCounterOverflow
            }
        };
        assert!(matches!(
            outcome,
            RelationalPublicationOutcome::Deferred(
                RelationalPublicationDeferred::CompanionPreflight(stop)
            ) if stop == expected
        ));
        assert_eq!(
            test_owner_main_basis(&runtime)
                .expect("denial retains the old main branch")
                .descriptor(),
            before.descriptor(),
        );
        assert_eq!(
            runtime.history.pending_canonical_publication_route_count(),
            0
        );
    }
}
