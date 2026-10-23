use super::*;

/// Preflight work and preparation-byte counters belong to the companion owner
/// (Query) since ee07da2b0e removed Relational's preflight quota bookkeeping;
/// Relational's contract is that the owner's counter-overflow stop survives
/// verbatim as a deferral before any companion effect or native head movement.
#[derive(Debug, Clone, Copy)]
enum OverflowMeter {
    Work,
    PreparationBytes,
}

impl OverflowMeter {
    const fn stop(self) -> CompanionPreflightStop {
        match self {
            Self::Work => CompanionPreflightStop::WorkCounterOverflow,
            Self::PreparationBytes => CompanionPreflightStop::PreparationMemoryCounterOverflow,
        }
    }
}

#[derive(Debug)]
struct OverflowCompanion {
    meter: OverflowMeter,
    prepared: Arc<AtomicUsize>,
}

impl RelationalPublicationCompanion for OverflowCompanion {
    fn prepare(
        &self,
        context: &mut PublicationCompanionPreflight<'_>,
    ) -> Result<PreparedPublicationCompanionEffect, CompanionPreflightStop> {
        // The live preflight admits the owner's work before its counter overflows.
        context.checkpoint()?;
        self.prepared.fetch_add(1, Ordering::SeqCst);
        Err(self.meter.stop())
    }
}

#[test]
fn preflight_counter_overflow_denies_without_moving_the_native_head() {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host_request);

    for meter in [OverflowMeter::Work, OverflowMeter::PreparationBytes] {
        let runtime = runtime_with_test_schema();
        let prepared = Arc::new(AtomicUsize::new(0));
        let before = test_owner_main_basis(&runtime).expect("main branch has owner basis");
        let pending = runtime
            .publication_companion_port()
            .begin_required_registration()
            .expect("required registration starts");
        let _registration = pending
            .activate(Arc::new(OverflowCompanion {
                meter,
                prepared: Arc::clone(&prepared),
            }))
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
        let expected = meter.stop();
        assert_eq!(
            prepared.load(Ordering::SeqCst),
            1,
            "the companion owner must run its preflight exactly once"
        );
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
