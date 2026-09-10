use super::*;

struct TwoStepProvider;

struct TwoStepExecution {
    phase: u8,
    retained: Option<WorthQueryGraphProviderRetainedMemory>,
}

impl WorthQueryGraphProviderExecution for TwoStepExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        match self.phase {
            0 => {
                self.phase = 1;
                for _ in 0..3 {
                    step.perform_work_unit(|| Ok(()))?;
                }
                step.with_scratch_bytes(8, |_| Ok(()))?;
                self.retained = Some(step.retain_bytes(4).map_err(step_failure)?);
                Ok(WorthQueryGraphProviderStepDisposition::continue_work())
            }
            1 => {
                self.phase = 2;
                drop(self.retained.take());
                for _ in 0..2 {
                    step.perform_work_unit(|| Ok(()))?;
                }
                step.emit_projection_chunk(graph_material())
                    .map_err(step_failure)?;
                WorthQueryGraphProviderStepDisposition::complete("two-step-provider")
                    .map_err(WorthQueryGraphProviderFailure::new)
            }
            _ => Err(WorthQueryGraphProviderFailure::new(
                "completed provider advanced again",
            )),
        }
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for TwoStepProvider {
    type Execution = TwoStepExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "managed-two-step",
            8,
        )
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
            TwoStepExecution {
                phase: 0,
                retained: None,
            },
        )
    }
}

struct FailingProvider;

struct FailingExecution {
    retained: Option<WorthQueryGraphProviderRetainedMemory>,
}

impl WorthQueryGraphProviderExecution for FailingExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        for _ in 0..2 {
            step.perform_work_unit(|| Ok(()))?;
        }
        self.retained = Some(step.retain_bytes(4).map_err(step_failure)?);
        Err(WorthQueryGraphProviderFailure::new(
            "provider failed after governed work",
        ))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for FailingProvider {
    type Execution = FailingExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support(
            "managed-failing",
            8,
        )
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(start, FailingExecution { retained: None })
    }
}

#[test]
fn bounded_provider_steps_stream_exact_terminal_work_evidence() {
    let (running, graph) =
        managed_graph_run_with_provider(WorthQueryOperationGraphAccess::Project, TwoStepProvider);
    let active = running
        .begin_graph_execution(
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Project,
                "managed-project",
            ),
        )
        .expect("installed provider anchor should start exact execution");
    let active = match active.advance() {
        WorthQueryDirectGraphStepOutcome::Continue(active) => active,
        _ => panic!("first bounded step did not continue"),
    };
    let pending = match active.advance() {
        WorthQueryDirectGraphStepOutcome::ChunkReady(pending) => pending,
        _ => panic!("second bounded step did not expose its bounded result chunk"),
    };
    assert_eq!(pending.chunk().rows().len(), 1);
    let completion = match pending.acknowledge() {
        WorthQueryDirectGraphStepOutcome::Completed(completion) => completion,
        _ => panic!("acknowledged final chunk did not complete"),
    };
    let stream = completion
        .receipt()
        .graph_read_stream_evidence()
        .expect("managed projection completion should carry stream evidence");
    assert_eq!(stream.chunk_count(), 1);
    assert_eq!(stream.row_count(), 1);
    let product = completion
        .receipt()
        .graph_read_product()
        .expect("acknowledged projection chunks must become the terminal read product");
    assert_eq!(product.row_count(), 1);
    assert_eq!(
        product
            .rows()
            .next()
            .expect("the terminal product must retain its emitted row")
            .entity_identity(),
        "managed-entity-0"
    );
    assert!(product.retained_bytes() > 0);
    assert!(
        stream.retained_bytes() > product.retained_bytes(),
        "stream custody must account for both materialization and its evidence envelope"
    );
    assert_eq!(completion.receipt().work_report().completed_work_units(), 5);
    assert_eq!(completion.receipt().work_report().scratch_bytes(), 8);
    assert_eq!(
        completion.receipt().work_report().retained_bytes(),
        stream.retained_bytes()
    );
    assert_eq!(
        completion.receipt().work_report().output_retained_bytes(),
        stream.retained_bytes()
    );

    let terminal = completion
        .into_running()
        .completed()
        .expect("Query-settled provider work should permit run completion");
    let work = terminal.provider_work().clone();
    assert_eq!(
        work.session_disposition(),
        WorthQueryManagedProviderSessionDisposition::ReceiptsAdmitted
    );
    assert_eq!(work.issued_call_count(), 1);
    assert_eq!(work.admitted_receipt_count(), 1);
    assert_eq!(work.completed_work_units(), 5);
    assert_eq!(work.peak_scratch_bytes(), 8);
    assert_eq!(work.retained_bytes(), 0);
    assert!(work.peak_retained_bytes() >= 4);
    assert_eq!(work.output_capacity_classification_count(), 2);
    let last_safe_point = work
        .last_safe_point()
        .expect("terminal work retains the exact last Signal safe point");
    assert_eq!(
        last_safe_point.signal_state(),
        worth_runtime_bridge::facade::BridgeExecutionSafePointSignalState::Active
    );
    assert_eq!(last_safe_point.queue_depth(), 0);
    assert_eq!(last_safe_point.queue_capacity(), 8);
    let cleanup = terminal.cleanup().expect("settled provider work cleans up");
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::CleanupComplete
    );
}

#[test]
fn provider_failure_preserves_governed_work_and_recovery_authority() {
    let (running, graph) =
        managed_graph_run_with_provider(WorthQueryOperationGraphAccess::Observe, FailingProvider);
    let active = running
        .begin_graph_execution(
            &graph,
            WorthQueryManagedGraphCallRequest::new(
                WorthQueryGraphProviderCallKind::Observe,
                "failing-observe",
            ),
        )
        .expect("failing provider should start before its bounded work");
    let terminal = match active.advance() {
        WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
        _ => panic!("provider failure did not terminalize the managed run"),
    };
    assert_eq!(terminal.provider_work().completed_work_units(), 2);
    assert_eq!(terminal.provider_work().retained_bytes(), 0);
    assert_eq!(terminal.provider_work().peak_retained_bytes(), 4);
    assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
    let cleanup = terminal
        .cleanup()
        .expect("failed provider run retains cleanup authority");
    assert_eq!(
        cleanup.inspection().disposition(),
        WorthQueryManagedRunCleanupDisposition::RecoveryRequired
    );
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
