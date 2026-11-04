use crate::domain_computation::managed_run::tests::*;

pub struct MultiChunkProvider {
    pub advances: Arc<AtomicUsize>,
}

pub struct MultiChunkExecution {
    pub advances: Arc<AtomicUsize>,
}

impl WorthQueryGraphProviderExecution for MultiChunkExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        let ordinal = self.advances.fetch_add(1, Ordering::Relaxed);
        step.perform_work_unit(|| Ok(()))?;
        step.emit_projection_chunk(graph_material())
            .map_err(step_failure)?;
        if ordinal == 0 {
            Ok(WorthQueryGraphProviderStepDisposition::continue_work())
        } else {
            WorthQueryGraphProviderStepDisposition::complete("multi-chunk")
                .map_err(WorthQueryGraphProviderFailure::new)
        }
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for MultiChunkProvider {
    type Execution = MultiChunkExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support("multi-chunk", 8)
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(
            start,
            MultiChunkExecution {
                advances: Arc::clone(&self.advances),
            },
        )
    }
}

#[test]
fn each_streamed_chunk_requires_consumption_before_the_next_provider_step() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let active = start_projection(
            execution,
            MultiChunkProvider {
                advances: Arc::clone(&advances),
            },
            resource_request,
        );
        let first = expect_chunk({
            let installed_fixture_phase = active.fixture_phase_for_test(execution);
            active.advance(&installed_fixture_phase)
        });
        assert_eq!(first.queue_depth(), 1);
        assert_eq!(first.queue_capacity(), 8);
        assert_eq!(advances.load(Ordering::Relaxed), 1);

        let active = match first.acknowledge() {
            WorthQueryDirectGraphStepOutcome::Continue(active) => active,
            _ => panic!("consuming the first chunk did not release the next step"),
        };
        let second = expect_chunk({
            let installed_fixture_phase = active.fixture_phase_for_test(execution);
            active.advance(&installed_fixture_phase)
        });
        assert_eq!(advances.load(Ordering::Relaxed), 2);
        let completion = match second.acknowledge() {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("consuming the final chunk did not complete"),
        };
        let stream = completion
            .receipt()
            .graph_read_stream_evidence()
            .expect("managed projection should seal stream evidence");
        assert_eq!(stream.chunk_count(), 2);
        assert_eq!(stream.row_count(), 2);
        assert_eq!(stream.product().row_count(), 2);
        assert_eq!(stream.product().rows().count(), 2);
        assert_eq!(
            completion.receipt().work_report().output_retained_bytes(),
            stream.retained_bytes()
        );
        assert_eq!(
            completion.receipt().work_report().retained_bytes(),
            stream.retained_bytes()
        );
    });
}

#[test]
fn stalled_consumer_cancellation_releases_the_chunk_before_terminal_cleanup() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let pending = expect_chunk({
            let fixture_state = start_projection(
                execution,
                MultiChunkProvider {
                    advances: Arc::clone(&advances),
                },
                resource_request,
            );
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.advance(&installed_fixture_phase)
        });
        pending
        .request_cancellation(
            worth_runtime_bridge::facade::BridgeManagedExecutionCancellationReason::HostRequested,
        )
        .expect("pending chunk should retain the exact Signal request");
        let terminal = match pending.acknowledge() {
            WorthQueryDirectGraphStepOutcome::Cancelled(terminal) => terminal,
            _ => panic!("cancelled stalled consumer did not derive cancellation"),
        };
        assert_eq!(advances.load(Ordering::Relaxed), 1);
        assert_eq!(terminal.provider_work().interrupted_call_count(), 1);
        assert_eq!(terminal.provider_work().queue_state_mutation_count(), 2);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        let cleanup = terminal
            .cleanup()
            .expect("cancelled stream should clean up");
        assert!(cleanup.inspection().resources_released());
    });
}

#[test]
fn foreign_consumer_failure_preserves_queue_occupancy_for_owner_cleanup() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let pending = expect_chunk({
            let fixture_state = start_projection(
                execution,
                MultiChunkProvider {
                    advances: Arc::new(AtomicUsize::new(0)),
                },
                resource_request,
            );
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.advance(&installed_fixture_phase)
        });
        let terminal = std::thread::spawn(move || match pending.acknowledge() {
            WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("foreign consumer should fail its Signal safe-point observation"),
        })
        .join()
        .expect("foreign consumer should return the terminal recovery authority");
        assert_eq!(terminal.provider_work().queue_state_mutation_count(), 1);

        let cleanup = terminal
            .cleanup()
            .expect("Signal owner should release the retained queue occupancy");
        assert_eq!(
            cleanup
                .inspection()
                .provider_work()
                .queue_state_mutation_count(),
            2
        );
        assert_eq!(
            cleanup.inspection().disposition(),
            WorthQueryManagedRunCleanupDisposition::RecoveryRequired
        );
        assert_eq!(cleanup.inspection().released_reservation_count(), 2);
    });
}

#[test]
fn exact_capacity_chunk_drains_before_completion() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Project,
            WideChunkProvider {
                advances: Arc::clone(&advances),
            },
            resource_request,
        );
        let installed_running_phase = execution.phase_for_direct_for_test(&running);

        let active = running
            .begin_graph_execution(
                &installed_running_phase,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Project,
                    "exact-capacity",
                ),
            )
            .expect("wide provider should start");
        let pending = expect_chunk(active.advance(&installed_running_phase));
        assert_eq!(pending.queue_depth(), 8);
        assert_eq!(pending.queue_capacity(), 8);
        let completion = match pending.acknowledge() {
            WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
            _ => panic!("an admitted exact-capacity chunk could not drain"),
        };
        assert_eq!(advances.load(Ordering::Relaxed), 1);
        assert_eq!(
            completion
                .receipt()
                .graph_read_stream_evidence()
                .expect("exact-capacity projection should seal stream evidence")
                .row_count(),
            8
        );
        let terminal = completion.into_running().completed().unwrap();
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        terminal
            .cleanup()
            .expect("completed stream should clean up");
    });
}

pub struct WideChunkProvider {
    pub advances: Arc<AtomicUsize>,
}

pub struct WideChunkExecution {
    pub advances: Arc<AtomicUsize>,
}

impl WorthQueryGraphProviderExecution for WideChunkExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        self.advances.fetch_add(1, Ordering::Relaxed);
        step.perform_work_unit(|| Ok(()))?;
        step.emit_projection_chunk(graph_material_rows(8))
            .map_err(step_failure)?;
        WorthQueryGraphProviderStepDisposition::complete("wide-chunk")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for WideChunkProvider {
    type Execution = WideChunkExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        worth_query_testkit_execution_primary_graph_application_attempt_2::domain_computation::provider_session::tests::execution_resource_support("wide-chunk", 8)
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(
            start,
            WideChunkExecution {
                advances: Arc::clone(&self.advances),
            },
        )
    }
}

#[test]
fn pending_and_paused_abandonment_release_queue_and_output_retention() {
    worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::with_test_advancement(|active_phase| {
        let bootstrap = active_phase.bootstrap_for_test();
        let resource_request = bootstrap.execution_request();

        let phase = &active_phase;
        let execution = phase;

        let advances = Arc::new(AtomicUsize::new(0));
        let pending = expect_chunk({
            let fixture_state = start_projection(
                execution,
                MultiChunkProvider {
                    advances: Arc::clone(&advances),
                },
                resource_request,
            );
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.advance(&installed_fixture_phase)
        });
        let terminal = match pending.abandon() {
            WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("pending-chunk abandonment did not terminalize"),
        };
        assert_eq!(terminal.provider_work().queue_state_mutation_count(), 2);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        assert!(terminal.provider_work().peak_retained_bytes() > 0);
        assert_eq!(terminal.provider_work().abandoned_call_count(), 1);

        let pending = expect_chunk({
            let fixture_state = start_projection(
                execution,
                MultiChunkProvider {
                    advances: Arc::new(AtomicUsize::new(0)),
                },
                resource_request,
            );
            let installed_fixture_phase = fixture_state.fixture_phase_for_test(execution);
            fixture_state.advance(&installed_fixture_phase)
        });
        let paused = match pending.acknowledge() {
            WorthQueryDirectGraphStepOutcome::Continue(paused) => paused,
            _ => panic!("first chunk acknowledgement did not reach a paused safe point"),
        };
        let terminal = match paused.abandon() {
            WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("paused execution abandonment did not terminalize"),
        };
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
    });
}

pub fn start_projection(
    execution: &worth_query_execution::court_doors_for_test::domain_computation::primary_graph::application_contribution::request_execution::advancement::test_host_policy::TestHostRequest<'_>,

    provider: MultiChunkProvider,
    resource_request: worth_execution::ExecutionRequest<'_, '_>,
) -> worth_query_execution::facade::domain_computation::WorthQueryActiveDirectGraphExecution {
    let (running, graph) = managed_graph_run_with_provider(
        WorthQueryOperationGraphAccess::Project,
        provider,
        resource_request,
    );
    let installed_running_phase = execution.phase_for_direct_for_test(&running);

    running
        .begin_graph_execution(
            &installed_running_phase,
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "streamed-project",
            ),
        )
        .expect("streaming provider should start")
}

pub fn expect_chunk(
    outcome: WorthQueryDirectGraphStepOutcome,
) -> worth_query_execution::facade::domain_computation::WorthQueryPendingDirectGraphChunk {
    match outcome {
        WorthQueryDirectGraphStepOutcome::ChunkReady(pending) => pending,
        _ => panic!("provider did not expose a bounded pending chunk"),
    }
}

pub fn step_failure(
    denial: worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
