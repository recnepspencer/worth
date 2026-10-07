use worth_query::facade::domain;

use super::native_consumer::execute_native_consumer;
use super::provider::ArtifactProbe;
use crate::suite::installed_operation_fixture::{GeometryDomain, ReadFamily, WorkflowRead};

#[derive(Clone)]
pub struct ArtifactWorkflowExecutor {
    probe: ArtifactProbe,
}

impl ArtifactWorkflowExecutor {
    pub fn new(probe: ArtifactProbe) -> Self {
        Self { probe }
    }

    fn execute_producer(
        &self,
        input: domain::WorthQueryWorkflowValue,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let mode = match input {
            domain::WorthQueryWorkflowValue::Text(mode) => mode,
            _ => return Err(failure("producer expected a text scenario")),
        };
        let admission = self.production_admission_for_mode(&mode, context)?;
        let handle = self.register_producer_resource(&mode, admission, workspace)?;
        self.finish_producer(mode, handle, context, workspace)
    }

    fn production_admission_for_mode(
        &self,
        mode: &str,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
    ) -> Result<
        domain::WorthQueryArtifactProductionAdmission,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let admission = if mode == "reuse-retained-admission" {
            self.probe
                .take_retained_admission()
                .ok_or_else(|| failure("no retained artifact admission was available"))?
        } else {
            production_admission(context)?
        };
        if mode == "retain-admission" {
            self.probe.retain_admission(admission);
            return Err(failure(
                "artifact production admission retained for sabotage",
            ));
        }
        Ok(admission)
    }

    fn register_producer_resource(
        &self,
        mode: &str,
        admission: domain::WorthQueryArtifactProductionAdmission,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryMoveOnlyArtifactHandle,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        if mode == "panic-during-projection" {
            let _ = workspace.register_artifact(admission, self.probe.panic_during_projection());
            unreachable!("provider projection panic must unwind registration");
        }
        if mode == "reject-provider" {
            let denial = workspace
                .register_artifact(admission, self.probe.foreign(b"foreign"))
                .expect_err("foreign provider must be rejected");
            self.probe.observe_denial(denial.kind());
            return Err(failure("foreign provider rejected"));
        }
        if mode == "reuse-retained-admission" {
            let denial = workspace
                .register_artifact(admission, self.probe.candidate(b"retained-admission"))
                .expect_err("retained admission from another run must be rejected");
            self.probe.observe_denial(denial.kind());
            return Err(failure("retained artifact admission rejected"));
        }
        let resource = if mode.starts_with("native-") {
            self.probe.native_candidate(b"canonical-candidates", mode)
        } else {
            self.probe.candidate(b"canonical-candidates")
        };
        workspace
            .register_artifact(admission, resource)
            .map_err(artifact_failure)
    }

    fn finish_producer(
        &self,
        mode: String,
        handle: domain::WorthQueryMoveOnlyArtifactHandle,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        if mode == "panic-during-replacement" {
            let _ = workspace.replace_artifact(
                handle,
                production_admission(context)?,
                self.probe.panic_during_projection(),
            );
            unreachable!("replacement projection panic must unwind replacement");
        }
        if mode == "cancel" {
            let disposed = handle.cancel();
            if disposed.disposition() == domain::WorthQueryArtifactDisposition::Cancelled
                && matches!(
                    disposed.provider_release(),
                    domain::WorthQueryArtifactProviderReleasePosture::Complete(_)
                )
            {
                self.probe.observe_cancellation();
            }
            return Err(failure("artifact cancelled by producer"));
        }
        if mode == "panic-after-production" {
            panic!("declared artifact handoff panic");
        }
        if mode == "fail-after-production" {
            return Err(failure("declared failure after artifact production"));
        }
        if mode == "retain-observer-lease" {
            let lease = handle
                .retain("foreign-runtime-observer")
                .map_err(artifact_failure)?;
            self.probe.escape_lease(lease);
        }
        if consumer_sabotage_mode(&mode) {
            self.probe.arm_consumer(mode.clone());
        }
        let handle = self.replace_if_requested(&mode, handle, context, workspace)?;
        Ok(domain::WorthQueryWorkflowStageMaterial::new(
            domain::WorthQueryWorkflowValue::installed_artifact(handle),
        ))
    }

    fn replace_if_requested(
        &self,
        mode: &str,
        handle: domain::WorthQueryMoveOnlyArtifactHandle,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryMoveOnlyArtifactHandle,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        if mode != "replace" {
            return Ok(handle);
        }
        let replacement = workspace
            .replace_artifact(
                handle,
                production_admission(context)?,
                self.probe.candidate(b"canonical-candidates"),
            )
            .map_err(|stop| artifact_failure(stop.into_parts().0))?;
        if replacement.prior().disposition() == domain::WorthQueryArtifactDisposition::Replaced
            && matches!(
                replacement.prior().provider_release(),
                domain::WorthQueryArtifactProviderReleasePosture::Complete(_)
            )
        {
            self.probe.observe_replacement();
        }
        Ok(replacement.into_replacement())
    }

    fn execute_consumer(
        &self,
        input: domain::WorthQueryWorkflowValue,
        context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
        workspace: &mut domain::WorthQueryWorkflowStageWorkspace<'_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let transferred = input
            .into_transferred_artifact()
            .map_err(|_| failure("consumer expected a transferred artifact"))?;
        self.probe.observe_lifecycle(transferred.owner_snapshot());
        let consumer_mode = self.probe.take_consumer_mode();
        if let Some(mode) = consumer_mode
            .as_deref()
            .filter(|mode| mode.starts_with("native-"))
        {
            return execute_native_consumer(mode, transferred, workspace, &self.probe);
        }
        let view = transferred
            .borrow(format!("{}-projection", context.stage().identity()))
            .map_err(artifact_failure)?;
        self.probe.observe_lifecycle(transferred.owner_snapshot());
        if view.semantic_projection().bytes() != b"canonical-candidates" {
            return Err(failure("consumer observed a non-canonical projection"));
        }
        self.probe.observe_borrow();
        drop(view);
        self.probe.observe_lifecycle(transferred.owner_snapshot());
        match consumer_mode.as_deref() {
            Some("fail-after-transfer") => {
                return Err(failure("declared failure after artifact transfer"));
            }
            Some("fail-after-lease-transfer") => {
                return Err(failure("declared failure after artifact lease transfer"));
            }
            Some("panic-after-transfer") => {
                panic!("declared artifact transfer panic");
            }
            Some("escape-after-transfer") => {
                self.probe.escape_handle(transferred);
                return Err(failure("transferred artifact escaped the stage executor"));
            }
            Some(mode) => return Err(failure(format!("unknown artifact consumer mode: {mode}"))),
            None => {}
        }
        Ok(
            domain::WorthQueryWorkflowStageMaterial::new(domain::WorthQueryWorkflowValue::Text(
                context.stage().identity().to_owned(),
            ))
            .with_result_state(domain::WorthQueryOperationResultState::Ready),
        )
    }
}

impl domain::WorthQueryDomainWorkflowStageExecutor<GeometryDomain, WorkflowRead, ReadFamily>
    for ArtifactWorkflowExecutor
{
    const LOWERING_FAMILY: &'static str = "artifact-workflow-v1";
    const DETERMINISTIC: bool = true;
    const IDEMPOTENT_STAGE_RETRY: bool = true;
    const EXECUTION_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const RESULT_WIDTH_COST: domain::WorthQueryOperationCostClass =
        domain::WorthQueryOperationCostClass::DeclaredWidth;
    const REPLAY_COMPARATOR_FAMILY: Option<&'static str> = Some("artifact-workflow-exact-v1");

    fn execution_resource_support(&self) -> domain::WorthQueryExecutionResourceSupport {
        super::super::execution_resource_support()
    }

    fn apply(
        &self,
        application: domain::WorthQueryWorkflowStageApplication<'_, '_, '_>,
    ) -> Result<
        domain::WorthQueryWorkflowStageMaterial,
        domain::WorthQueryWorkflowStageExecutorFailure,
    > {
        let (input, _computed, context, workspace) = application.into_parts();
        match context.stage().identity() {
            "produce" => self.execute_producer(input, context, workspace),
            "consume" | "observe-a" | "observe-b" => {
                self.execute_consumer(input, context, workspace)
            }
            _ => Err(failure("unknown artifact workflow stage")),
        }
    }
}

impl domain::WorthQueryDomainReplaySemanticComparator<GeometryDomain, WorkflowRead, ReadFamily>
    for ArtifactWorkflowExecutor
{
    fn compare_replay_semantics(
        &self,
        original: &domain::WorthQueryWorkflowTraceSemantics,
        replay: &domain::WorthQueryWorkflowTraceSemantics,
        noise: domain::WorthQueryReplayNoiseContract,
    ) -> domain::WorthQueryReplayComparison {
        domain::compare_exact_workflow_traces(original, replay, noise)
    }
}

fn production_admission(
    context: &domain::WorthQueryWorkflowStageExecutionContext<'_>,
) -> Result<
    domain::WorthQueryArtifactProductionAdmission,
    domain::WorthQueryWorkflowStageExecutorFailure,
> {
    context
        .admit_artifact_production(domain::WorthQueryArtifactProductionEvidence::new(
            "artifact-workflow-provenance",
            "artifact-workflow-dependency",
        ))
        .map_err(artifact_failure)
}

fn artifact_failure(
    denial: domain::WorthQueryArtifactDenial,
) -> domain::WorthQueryWorkflowStageExecutorFailure {
    failure(format!("{:?}: {}", denial.kind(), denial.detail()))
}

fn failure(detail: impl Into<String>) -> domain::WorthQueryWorkflowStageExecutorFailure {
    domain::WorthQueryWorkflowStageExecutorFailure::new(
        domain::WorthQueryOperationFailureClass::Dependency,
        detail,
    )
}

fn consumer_sabotage_mode(mode: &str) -> bool {
    mode.starts_with("native-")
        || matches!(
            mode,
            "fail-after-transfer"
                | "panic-after-transfer"
                | "escape-after-transfer"
                | "fail-after-lease-transfer"
        )
}
