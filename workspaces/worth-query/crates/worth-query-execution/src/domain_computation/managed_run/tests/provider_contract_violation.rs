use super::*;

#[derive(Clone, Copy)]
enum HostilePort {
    Effect,
    Output,
    OutputThenFailure,
    Scratch,
    Retained,
    Artifact,
    Checkpoint,
    NoProgress,
}

struct HostileProvider(HostilePort);

struct HostileExecution(HostilePort);

impl WorthQueryGraphProviderExecution for HostileExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        match self.0 {
            HostilePort::Effect => {
                let _ = step.apply_effect(|| Ok(()));
            }
            HostilePort::Output => {
                step.perform_work_unit(|| Ok(()))?;
                let _ = step.emit_projection_chunk(graph_material_rows(9));
            }
            HostilePort::OutputThenFailure => {
                step.perform_work_unit(|| Ok(()))?;
                step.emit_projection_chunk(graph_material())
                    .map_err(step_failure)?;
                return Err(WorthQueryGraphProviderFailure::new(
                    "provider failed after producing unpublished output",
                ));
            }
            HostilePort::Scratch => {
                step.perform_work_unit(|| Ok(()))?;
                let _ = step.with_scratch_bytes(9, |_| Ok(()));
            }
            HostilePort::Retained => {
                step.perform_work_unit(|| Ok(()))?;
                let _ = step.retain_bytes(4_097);
            }
            HostilePort::Artifact => {
                let _ = step.produce_artifact(
                    WorthQueryArtifactProductionEvidence::new(
                        "hostile-artifact",
                        "missing-artifact-authority",
                    ),
                    HostileArtifact,
                );
            }
            HostilePort::Checkpoint => {
                step.perform_work_unit(|| Ok(()))?;
                step.record_checkpoint_available().map_err(step_failure)?;
                let _ = step.record_checkpoint_available();
            }
            HostilePort::NoProgress => {}
        }
        WorthQueryGraphProviderStepDisposition::complete("hostile")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<ManagedGraph> for HostileProvider {
    type Execution = HostileExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        crate::domain_computation::provider_session::execution_resource_support("hostile-port", 8)
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        admit_provider_execution(start, HostileExecution(self.0))
    }
}

#[test]
fn ignored_governed_denials_and_zero_progress_completion_cannot_advance() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        for (port, expected_denial) in [
            (
                HostilePort::Effect,
                WorthQueryGraphProviderStepDenialKind::UnexpectedEffect,
            ),
            (
                HostilePort::Output,
                WorthQueryGraphProviderStepDenialKind::ChunkWidthExceeded,
            ),
            (
                HostilePort::Scratch,
                WorthQueryGraphProviderStepDenialKind::ScratchBudgetExceeded,
            ),
            (
                HostilePort::Retained,
                WorthQueryGraphProviderStepDenialKind::RetainedBudgetExceeded,
            ),
            (
                HostilePort::Artifact,
                WorthQueryGraphProviderStepDenialKind::ArtifactAdmissionDenied,
            ),
            (
                HostilePort::Checkpoint,
                WorthQueryGraphProviderStepDenialKind::MultipleCheckpoints,
            ),
            (
                HostilePort::NoProgress,
                WorthQueryGraphProviderStepDenialKind::NoProgress,
            ),
        ] {
            let (access, kind) = if matches!(port, HostilePort::Output) {
                (
                    WorthQueryOperationGraphAccess::Project,
                    WorthQueryGraphProviderCallKind::Project,
                )
            } else {
                (
                    WorthQueryOperationGraphAccess::Observe,
                    WorthQueryGraphProviderCallKind::Observe,
                )
            };
            let (running, graph) = managed_graph_run_with_provider(access, HostileProvider(port));
            let active = running
                .begin_graph_execution(
                    execution,
                    &graph,
                    WorthQueryManagedGraphCallRequest::new(kind, "hostile-port"),
                )
                .expect("hostile fixture should reach its governed step");
            let terminal = match active.advance(execution) {
                WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
                _ => panic!("hostile governed port advanced the managed lane"),
            };
            assert_eq!(terminal.provider_work().abandoned_call_count(), 1);
            let failure = terminal
                .provider_work()
                .last_step_failure()
                .expect("failed provider step should retain its exact cause");
            assert_eq!(
                failure.invocation(),
                WorthQueryGraphProviderStepInvocationDisposition::Returned
            );
            assert_eq!(failure.governed_denial_kind(), Some(expected_denial));
            terminal
                .cleanup()
                .expect("failed hostile step should clean up");
        }
    });
}

struct HostileArtifact;

impl WorthQueryArtifactProviderResource for HostileArtifact {
    const PROVIDER_FAMILY: &'static str = "WORTH.tests.hostile-artifact";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"hostile-artifact".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        1
    }

    fn dispose(&mut self) {}
}

#[test]
fn failed_provider_output_is_released_without_queue_publication() {
    crate::domain_computation::primary_graph::with_test_advancement(|active_phase| {
        let phase = &active_phase;
        let execution = phase;

        let (running, graph) = managed_graph_run_with_provider(
            WorthQueryOperationGraphAccess::Project,
            HostileProvider(HostilePort::OutputThenFailure),
        );
        let active = running
            .begin_graph_execution(
                execution,
                &graph,
                WorthQueryManagedGraphCallRequest::new(
                    WorthQueryGraphProviderCallKind::Project,
                    "output-then-failure",
                ),
            )
            .expect("hostile output provider should reach its governed step");
        let terminal = match active.advance(execution) {
            WorthQueryDirectGraphStepOutcome::Failed(terminal) => terminal,
            _ => panic!("failed provider output became a promotable chunk"),
        };
        assert_eq!(terminal.provider_work().queue_state_mutation_count(), 0);
        assert_eq!(terminal.provider_work().retained_bytes(), 0);
        assert!(terminal.provider_work().peak_retained_bytes() > 0);
        let failure = terminal.provider_work().last_step_failure().unwrap();
        assert_eq!(
            failure.invocation(),
            WorthQueryGraphProviderStepInvocationDisposition::Rejected
        );
        assert_eq!(
            failure.provider_failure_detail(),
            Some("provider failed after producing unpublished output")
        );
        terminal
            .cleanup()
            .expect("failed unpublished output preserves cleanup authority");
    });
}

fn graph_material() -> WorthQueryGraphReadMaterial {
    graph_material_rows(1)
}

fn graph_material_rows(row_count: usize) -> WorthQueryGraphReadMaterial {
    let path = CanonicalFieldPath::single(FieldKey::new("id").expect("valid field key"));
    WorthQueryGraphReadMaterial::new((0..row_count).map(|index| {
        WorthQueryGraphReadRow::from_native_fields(
            format!("managed-entity-{index}"),
            [(
                path.clone(),
                AspectValue::String(InternedString::from(format!("entity-{index}"))),
            )]
            .into_iter()
            .collect(),
        )
        .expect("managed graph row should construct")
    }))
}

fn step_failure(
    denial: crate::domain_computation::WorthQueryGraphProviderStepDenial,
) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}
