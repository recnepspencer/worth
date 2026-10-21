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
                context.checkpoint()?;
                context.checkpoint()?;
            }
            OverflowMeter::PreparationBytes => {
                context.checkpoint()?;
                context.checkpoint()?;
            }
        }
        panic!("an overflowing preflight charge must stop before a companion effect exists");
    }
}

#[test]
fn preflight_counter_overflow_denies_without_moving_the_native_head() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for meter in [OverflowMeter::Work, OverflowMeter::PreparationBytes] {
        let runtime = runtime_with_test_schema();
        let before = test_owner_main_basis(&runtime).expect("main branch has owner basis");
        let pending = runtime
            .publication_companion_port()
            .begin_required_registration()
            .expect("required registration starts");
        let _registration = pending
            .activate(Arc::new(OverflowCompanion(meter)))
            .expect("overflow fixture is active");
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                batch_create("overflow-stops-before-movement"),
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .expect("candidate stages");
        let candidate = runtime
            .prepare_branch_transaction(execution, transaction)
            .expect("candidate prepares");
        let outcome = runtime
            .publication_port()
            .compare_and_publish(execution, candidate);
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
