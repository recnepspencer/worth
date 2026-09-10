use std::collections::BTreeSet;
use std::sync::Arc;

use crate::basis_lifecycle::BasisOperationLane;
use crate::runtime::WorthQueryWorkspace;
use worth_proof::TransitionOutcome;

use super::workflow_progression_state::WorthQueryWorkflowAdvanceStep;
use super::{
    WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowAdvanceDenialKind,
    WorthQueryWorkflowParallelAdmissionCall, WorthQueryWorkflowParallelAdmissionReceipt,
    WorthQueryWorkflowParallelFrontierStage, WorthQueryWorkflowRun, WorthQueryWorkflowValue,
};

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub fn advance_admitted_frontier(
        mut self,
        stages: impl IntoIterator<Item = (String, WorthQueryWorkflowValue)>,
        workspace: &mut WorthQueryWorkspace,
    ) -> super::workflow_progression::WorthQueryWorkflowAdvanceOutcome<D, O, F, L> {
        match self.advance_frontier_once(stages, workspace) {
            Ok(WorthQueryWorkflowAdvanceStep::Advanced) => TransitionOutcome::Success(self),
            Ok(WorthQueryWorkflowAdvanceStep::Deferred(conditional)) => {
                TransitionOutcome::Deferred(
                    crate::domain_installation::WorthQueryDeferredWorkflowStage {
                        run: self,
                        conditional,
                    },
                )
            }
            Err(denial) => self.outcome_from_denial(denial),
        }
    }

    fn advance_frontier_once(
        &mut self,
        stages: impl IntoIterator<Item = (String, WorthQueryWorkflowValue)>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        let stages = self.canonical_parallel_stages(stages)?;
        self.validate_parallel_runtime_authority(workspace)?;
        let frontier = self.prepare_parallel_frontier(&stages)?;
        self.admit_parallel_frontier(frontier)?;
        self.execute_parallel_stages(stages, workspace)
    }

    fn canonical_parallel_stages(
        &self,
        stages: impl IntoIterator<Item = (String, WorthQueryWorkflowValue)>,
    ) -> Result<Vec<(String, WorthQueryWorkflowValue)>, WorthQueryWorkflowAdvanceDenial> {
        let mut stages = stages.into_iter().collect::<Vec<_>>();
        stages.sort_by(|left, right| left.0.cmp(&right.0));
        let duplicated = stages
            .windows(2)
            .any(|pair| pair[0].0.as_str() == pair[1].0.as_str());
        if stages.len() < 2 || duplicated {
            return Err(self.denial(WorthQueryWorkflowAdvanceDenialKind::ParallelFrontierShape));
        }
        if !self.bound.definition().semantics().lowering.deterministic {
            return Err(self.denial(WorthQueryWorkflowAdvanceDenialKind::NonDeterministicLowering));
        }
        Ok(stages)
    }

    fn validate_parallel_runtime_authority(
        &mut self,
        workspace: &WorthQueryWorkspace,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        self.counters.runtime_authority_checks += 1;
        let witness =
            crate::domain_installation::WorthQueryInstalledDomainAuthorityWitness::from_authority(
                Arc::clone(self.bound.operation().domain_authority()),
            );
        workspace
            .validate_installed_domain_witness::<D>(&witness)
            .map_err(|denial| {
                self.denial(WorthQueryWorkflowAdvanceDenialKind::RuntimeAuthority(
                    denial.kind(),
                ))
            })
    }

    fn prepare_parallel_frontier(
        &mut self,
        stages: &[(String, WorthQueryWorkflowValue)],
    ) -> Result<Vec<WorthQueryWorkflowParallelFrontierStage>, WorthQueryWorkflowAdvanceDenial> {
        let mut frontier = Vec::with_capacity(stages.len());
        for (stage_identity, input) in stages {
            frontier.push(self.prepare_parallel_stage(stage_identity, input)?);
        }
        Ok(frontier)
    }

    fn prepare_parallel_stage(
        &mut self,
        stage_identity: &str,
        input: &WorthQueryWorkflowValue,
    ) -> Result<WorthQueryWorkflowParallelFrontierStage, WorthQueryWorkflowAdvanceDenial> {
        self.counters.stage_admission_checks += 1;
        self.counters.stage_index_lookups += 1;
        let stage = self
            .graph
            .stage(stage_identity)
            .cloned()
            .ok_or_else(|| self.denial(WorthQueryWorkflowAdvanceDenialKind::UnknownStage))?;
        if self.completed.contains(stage_identity) {
            return Err(self.denial(WorthQueryWorkflowAdvanceDenialKind::StageAlreadyCompleted));
        }
        let predecessor_receipts = self.parallel_predecessor_receipts(&stage)?;
        if !input.satisfies(&stage.semantics().input) {
            return Err(self.denial(WorthQueryWorkflowAdvanceDenialKind::InputContract));
        }
        self.validate_frontier_requirements(&stage)?;
        Ok(WorthQueryWorkflowParallelFrontierStage::new(
            stage_identity.to_string(),
            predecessor_receipts,
            stage.semantics().graph_read_roles.clone(),
            stage.semantics().touch_roles.clone(),
            stage.semantics().effect_roles.clone(),
        ))
    }

    fn parallel_predecessor_receipts(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    ) -> Result<Vec<String>, WorthQueryWorkflowAdvanceDenial> {
        let mut identities = Vec::with_capacity(stage.predecessors().len());
        for predecessor in stage.predecessors() {
            self.counters.predecessor_checks += 1;
            if !self.completed.contains(predecessor) {
                return Err(self.denial(
                    WorthQueryWorkflowAdvanceDenialKind::PredecessorIncomplete(predecessor.clone()),
                ));
            }
            let identity = self
                .receipt_index
                .get(predecessor)
                .map(|index| self.receipts[*index].identity())
                .ok_or_else(|| {
                    self.denial(
                        WorthQueryWorkflowAdvanceDenialKind::PredecessorAuthorityMissing(
                            predecessor.clone(),
                        ),
                    )
                })?;
            identities.push(identity.to_string());
        }
        identities.sort();
        Ok(identities)
    }

    fn admit_parallel_frontier(
        &mut self,
        frontier: Vec<WorthQueryWorkflowParallelFrontierStage>,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        let call = WorthQueryWorkflowParallelAdmissionCall::new(
            super::WorthQueryWorkflowParallelAdmissionCallParts {
                operation_identity: self.bound.definition().canonical_identity().into(),
                binding_identity: self.bound.binding_identity().into(),
                run_identity: self.identity.clone(),
                basis_identity: self.bound.basis().capability_digest().into(),
                frontier,
                execution_resources: self.operation_resource_evidence.clone(),
                resource_envelope: self.resources.operation().shared_envelope(),
            },
        );
        let provider = match &self.parallel_posture {
            crate::domain_installation::operating_world::WorthQueryBoundWorkflowParallelPosture::Parallel(provider) => provider,
            crate::domain_installation::operating_world::WorthQueryBoundWorkflowParallelPosture::Sequential => {
                unreachable!("an admitted parallel frontier carries its installed provider")
            }
        };
        self.counters.parallel_admission_checks += 1;
        let receipt = provider.admit(&call).map_err(|failure| {
            self.denial(WorthQueryWorkflowAdvanceDenialKind::ParallelProvider(
                failure.detail().into(),
            ))
        })?;
        if let Some(reason) = receipt.serial_fallback_reason() {
            return Err(
                self.denial(WorthQueryWorkflowAdvanceDenialKind::ParallelNotAdmitted(
                    reason,
                )),
            );
        }
        self.active_parallel_admission = Some(Arc::new(
            WorthQueryWorkflowParallelAdmissionReceipt::mint(&call, receipt),
        ));
        Ok(())
    }

    fn execute_parallel_stages(
        &mut self,
        stages: Vec<(String, WorthQueryWorkflowValue)>,
        workspace: &mut WorthQueryWorkspace,
    ) -> Result<WorthQueryWorkflowAdvanceStep, WorthQueryWorkflowAdvanceDenial> {
        for (stage_identity, input) in stages {
            if let WorthQueryWorkflowAdvanceStep::Deferred(conditional) =
                self.advance_once(&stage_identity, input, workspace)?
            {
                self.active_parallel_admission = None;
                return Ok(WorthQueryWorkflowAdvanceStep::Deferred(conditional));
            }
        }
        self.active_parallel_admission = None;
        Ok(WorthQueryWorkflowAdvanceStep::Advanced)
    }

    fn validate_frontier_requirements(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        for required in stage.required_capabilities() {
            self.counters.required_capability_checks += 1;
            if !self
                .bound
                .operation()
                .domain_authority()
                .required_capabilities()
                .iter()
                .any(|installed| installed.satisfies_operation_requirement(*required))
            {
                return Err(
                    self.denial(WorthQueryWorkflowAdvanceDenialKind::RequiredCapability(
                        required.as_str().into(),
                    )),
                );
            }
        }
        let bound_roles = self.bound.required_domain_roles().collect::<BTreeSet<_>>();
        for role in &stage.semantics().required_domain_roles {
            self.counters.required_domain_checks += 1;
            if !bound_roles.contains(role.as_str()) {
                return Err(
                    self.denial(WorthQueryWorkflowAdvanceDenialKind::RequiredDomain(
                        role.as_str().into(),
                    )),
                );
            }
        }
        Ok(())
    }

    fn denial(&self, kind: WorthQueryWorkflowAdvanceDenialKind) -> WorthQueryWorkflowAdvanceDenial {
        WorthQueryWorkflowAdvanceDenial::new(kind, self.counters)
    }
}
