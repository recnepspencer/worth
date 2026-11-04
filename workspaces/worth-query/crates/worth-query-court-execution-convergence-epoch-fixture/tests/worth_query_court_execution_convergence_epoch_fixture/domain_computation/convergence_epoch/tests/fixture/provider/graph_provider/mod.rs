use worth_foundational::facade::AspectValue;
use worth_foundational::facade::CanonicalFieldPath;
use worth_foundational::facade::FieldKey;
use worth_foundational::facade::InternedString;

use worth_query_execution::facade::domain_computation::WorthQueryArtifactProductionEvidence;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderResource;
use worth_query_execution::facade::domain_computation::WorthQueryCooperativeGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryGraphParticipationProvider;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCall;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderCheckpoint;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecution;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderExecutionStart;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderFailure;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderRestoreMemory;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderRetainedMemory;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStep;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDenial;
use worth_query_execution::facade::domain_computation::WorthQueryGraphProviderStepDisposition;
use worth_query_execution::facade::domain_computation::WorthQueryGraphReadMaterial;
use worth_query_execution::facade::domain_computation::WorthQueryGraphReadRow;

use crate::domain_computation::convergence_epoch::tests::fixture::provider::convergence_provider::ConvergentProvider;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::convergence_provider::FixtureCleanupArtifact;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::disposition::FixtureDisposition;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::resource_support::execution_support;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::resource_support::execution_support_with_broader_stage_queue_contract;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::yield_recovery::FixtureYieldRecoveryArtifact;
use crate::domain_computation::convergence_epoch::tests::fixture::provider::yield_recovery::FixtureYieldRecoveryProbe;

pub struct FixtureGraph;

pub struct CompletedGraphExecution {
    pub step_ordinal: u8,
    pub disposition: FixtureDisposition,
    pub retained: Option<WorthQueryGraphProviderRetainedMemory>,
    pub cleanup_artifact: Option<FixtureCleanupArtifact>,
    pub yield_recovery_probe: Option<FixtureYieldRecoveryProbe>,
}

impl WorthQueryGraphProviderExecution for CompletedGraphExecution {
    fn advance(
        &mut self,
        step: &mut WorthQueryGraphProviderStep,
    ) -> Result<WorthQueryGraphProviderStepDisposition, WorthQueryGraphProviderFailure> {
        step.perform_work_unit(|| Ok(()))?;
        if let Some(artifact) = self.cleanup_artifact.take() {
            produce_cleanup_artifact(step, artifact);
        }
        if let Some(width) = self.disposition.projection_width() {
            step.emit_projection_chunk(projection_rows(width))
                .map_err(step_failure)?;
        }
        if matches!(
            self.disposition,
            FixtureDisposition::YieldThenCheckpointUnavailable
                | FixtureDisposition::YieldThenConverged
                | FixtureDisposition::YieldThenRestorePanic
                | FixtureDisposition::YieldThenCheckpointDropPanic
                | FixtureDisposition::YieldThenSuspensionFailure
        ) && self.step_ordinal == 0
        {
            self.step_ordinal = 1;
            self.retained = Some(step.retain_bytes(1).map_err(step_failure)?);
            if !matches!(
                self.disposition,
                FixtureDisposition::YieldThenCheckpointUnavailable
            ) {
                step.record_checkpoint_available().map_err(step_failure)?;
            }
            return Ok(WorthQueryGraphProviderStepDisposition::continue_work());
        }
        WorthQueryGraphProviderStepDisposition::complete("convergence-provider-receipt")
            .map_err(WorthQueryGraphProviderFailure::new)
    }

    fn suspend(
        &mut self,
    ) -> Result<Box<dyn WorthQueryGraphProviderCheckpoint>, WorthQueryGraphProviderFailure> {
        if let Some(probe) = &self.yield_recovery_probe {
            probe.attempted_suspension();
        }
        if matches!(
            self.disposition,
            FixtureDisposition::YieldThenSuspensionFailure
        ) {
            return Err(WorthQueryGraphProviderFailure::new(
                "fixture provider rejected convergence checkpoint suspension",
            ));
        }
        matches!(
            self.disposition,
            FixtureDisposition::YieldThenCheckpointUnavailable
                | FixtureDisposition::YieldThenConverged
                | FixtureDisposition::YieldThenRestorePanic
                | FixtureDisposition::YieldThenCheckpointDropPanic
        )
        .then(|| {
            Box::new(ConvergenceCheckpoint {
                retained: self
                    .retained
                    .take()
                    .expect("yielding convergence execution retains governed memory"),
                disposition: self.disposition,
            }) as Box<dyn WorthQueryGraphProviderCheckpoint>
        })
        .ok_or_else(|| WorthQueryGraphProviderFailure::new("checkpoint not installed"))
    }

    fn dispose(&mut self) -> Result<(), WorthQueryGraphProviderFailure> {
        Ok(())
    }
}

impl WorthQueryGraphParticipationProvider<FixtureGraph> for ConvergentProvider {
    type Execution = CompletedGraphExecution;

    fn execution_resource_support(
        &self,
    ) -> worth_query_admission::facade::resource_admission::WorthQueryExecutionResourceSupport {
        if matches!(
            self.disposition(),
            FixtureDisposition::StageQueueContractMismatch
        ) {
            return execution_support_with_broader_stage_queue_contract();
        }
        execution_support(matches!(
            self.disposition(),
            FixtureDisposition::YieldThenCheckpointUnavailable
                | FixtureDisposition::YieldThenConverged
                | FixtureDisposition::YieldThenRestorePanic
                | FixtureDisposition::YieldThenCheckpointDropPanic
                | FixtureDisposition::YieldThenSuspensionFailure
        ))
    }

    fn begin(
        &self,
        _call: &WorthQueryGraphProviderCall,
        start: &mut WorthQueryGraphProviderExecutionStart,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Self::Execution>,
        WorthQueryGraphProviderFailure,
    > {
        let execution = CompletedGraphExecution {
            step_ordinal: 0,
            disposition: self.disposition(),
            retained: None,
            cleanup_artifact: self.cleanup_artifact(),
            yield_recovery_probe: self.yield_recovery_probe(),
        };
        start
            .admit_cooperative_execution(|| execution)
            .map_err(step_failure)
    }
}

pub struct ConvergenceCheckpoint {
    pub retained: WorthQueryGraphProviderRetainedMemory,
    pub disposition: FixtureDisposition,
}

impl WorthQueryGraphProviderCheckpoint for ConvergenceCheckpoint {
    fn retained_bytes(&self) -> u64 {
        u64::try_from(self.retained.len()).unwrap()
    }

    fn restore(
        &self,
        _call: &WorthQueryGraphProviderCall,
        memory: &mut WorthQueryGraphProviderRestoreMemory,
    ) -> Result<
        WorthQueryCooperativeGraphProviderExecution<Box<dyn WorthQueryGraphProviderExecution>>,
        WorthQueryGraphProviderFailure,
    > {
        assert!(
            !matches!(self.disposition, FixtureDisposition::YieldThenRestorePanic),
            "fixture convergence checkpoint restore panic"
        );
        let execution = Box::new(CompletedGraphExecution {
            step_ordinal: 1,
            disposition: FixtureDisposition::YieldThenConverged,
            retained: Some(memory.rebind(&self.retained).map_err(step_failure)?),
            cleanup_artifact: None,
            yield_recovery_probe: None,
        }) as Box<dyn WorthQueryGraphProviderExecution>;
        memory
            .admit_cooperative_execution(|| execution)
            .map_err(step_failure)
    }
}

impl Drop for ConvergenceCheckpoint {
    fn drop(&mut self) {
        assert!(
            !matches!(
                self.disposition,
                FixtureDisposition::YieldThenCheckpointDropPanic
            ),
            "fixture convergence checkpoint drop panic"
        );
    }
}

pub struct CleanupArtifactResource;

impl WorthQueryArtifactProviderResource for CleanupArtifactResource {
    const PROVIDER_FAMILY: &'static str = "worth.convergence.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"convergence-cleanup-pending".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        0
    }

    fn dispose(&mut self) {}
}

pub struct DoublePanickingCleanupArtifactResource {
    pub probe: FixtureYieldRecoveryProbe,
}

impl WorthQueryArtifactProviderResource for DoublePanickingCleanupArtifactResource {
    const PROVIDER_FAMILY: &'static str = "worth.convergence.provider";

    fn canonical_semantic_projection(&self) -> Vec<u8> {
        b"convergence-yield-recovery-double-panic".to_vec()
    }

    fn retained_bytes(&self) -> usize {
        0
    }

    fn dispose(&mut self) {
        self.probe.attempted_disposal();
        panic!("fixture convergence artifact disposal panicked")
    }
}

impl Drop for DoublePanickingCleanupArtifactResource {
    fn drop(&mut self) {
        self.probe.attempted_destructor();
        panic!("fixture convergence artifact destructor panicked")
    }
}

pub fn produce_cleanup_artifact(
    step: &mut WorthQueryGraphProviderStep,
    artifact: FixtureCleanupArtifact,
) {
    let evidence = WorthQueryArtifactProductionEvidence::new(
        "convergence-cleanup-pending",
        "fixture-held-cleanup-lease",
    );
    let handle = match artifact.behavior {
        FixtureYieldRecoveryArtifact::Cooperative => {
            step.produce_artifact(evidence, CleanupArtifactResource)
        }
        FixtureYieldRecoveryArtifact::DoublePanicking => step.produce_artifact(
            evidence,
            DoublePanickingCleanupArtifactResource {
                probe: artifact.probe,
            },
        ),
    }
    .unwrap_or_else(|denial| {
        panic!(
            "convergence cleanup artifact must produce: {}",
            denial.detail()
        )
    });
    artifact
        .sender
        .send(handle)
        .expect("convergence cleanup proof receiver must remain live");
}

pub fn step_failure(denial: WorthQueryGraphProviderStepDenial) -> WorthQueryGraphProviderFailure {
    WorthQueryGraphProviderFailure::new(denial.detail())
}

pub fn projection_rows(row_count: usize) -> WorthQueryGraphReadMaterial {
    let path = CanonicalFieldPath::single(FieldKey::new("state").expect("valid field key"));
    WorthQueryGraphReadMaterial::new((0..row_count).map(|index| {
        WorthQueryGraphReadRow::from_native_fields(
            format!("candidate-{index}"),
            [(
                path.clone(),
                AspectValue::String(InternedString::from(format!("state-{index}"))),
            )]
            .into_iter()
            .collect(),
        )
        .expect("fixture graph row must construct")
    }))
}
