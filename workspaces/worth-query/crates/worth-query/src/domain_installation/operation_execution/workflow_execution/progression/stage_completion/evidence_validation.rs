//! Validation rules applied inside the sealed workflow stage-completion owner.

use crate::basis_lifecycle::BasisOperationLane;
use crate::domain_installation::WorthQueryOperationGraphParticipation;

mod attachment;
pub(in crate::domain_installation::operation_execution) use attachment::WorthQueryWorkflowDomainEvidenceAttachment;

use super::super::workflow_stage_receipt::WorthQueryAdmittedWorkflowStageEvidence;
use super::{
    WorthQueryWorkflowAdvanceDenial, WorthQueryWorkflowAdvanceDenialKind,
    WorthQueryWorkflowInvariantOutcome, WorthQueryWorkflowRun, WorthQueryWorkflowRunCounters,
    WorthQueryWorkflowSemanticValue, WorthQueryWorkflowStageCompletion,
};

struct WorthQueryValidatedWorkflowEvidenceCompletion<'a, D, O, F, L>
where
    L: BasisOperationLane,
{
    run: &'a mut WorthQueryWorkflowRun<D, O, F, L>,
    stage: &'a worth_query_installation::facade::WorthQueryPortableWorkflowStage,
    input: WorthQueryWorkflowStageCompletion,
    invariant_outcomes: Vec<WorthQueryWorkflowInvariantOutcome>,
    output_semantics: WorthQueryWorkflowSemanticValue,
    stage_counters: WorthQueryWorkflowRunCounters,
}

impl<'a, D: 'static, O: 'static, F: 'static, L: BasisOperationLane>
    WorthQueryValidatedWorkflowEvidenceCompletion<'a, D, O, F, L>
{
    fn finish(
        mut self,
    ) -> Result<WorthQueryAdmittedWorkflowStageEvidence, WorthQueryWorkflowAdvanceDenial> {
        let output_occurrence_identity = self
            .input
            .material
            .output
            .domain_evidence_occurrence_identity();
        let attachment = WorthQueryWorkflowDomainEvidenceAttachment::from_completion(
            self.run,
            self.stage,
            &self.input,
            output_occurrence_identity,
        );
        let material = self.input.material.domain_evidence.take();
        let domain_evidence = super::super::admit_workflow_completion_content(
            attachment,
            material,
            &mut self.run.domain_evidence_ledger,
        )
        .map_err(|denial| {
            let kind = WorthQueryWorkflowAdvanceDenialKind::DomainEvidence(denial.kind());
            self.input.denial(self.run.counters, kind)
        })?;
        Ok(WorthQueryAdmittedWorkflowStageEvidence {
            input: self.input.semantic_input,
            output: self.input.material.output,
            output_semantics: self.output_semantics,
            result_state: self.input.material.result_state,
            warnings: self.input.material.warnings,
            graph_receipts: self.input.graph_receipts,
            primary_read_evidence: self.input.material.primary_graph_reads,
            effect_evidence: self.input.material.effects,
            invariant_outcomes: self.invariant_outcomes,
            counters: self.stage_counters,
            execution_snapshot: self.input.execution_snapshot,
            conditional: self.input.conditional,
            lineage: self.input.material.lineage,
            domain_evidence,
            resource_evidence: self.input.resource_evidence,
        })
    }
}

impl<D: 'static, O: 'static, F: 'static, L: BasisOperationLane> WorthQueryWorkflowRun<D, O, F, L> {
    pub(super) fn validate_stage_evidence(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        mut input: WorthQueryWorkflowStageCompletion,
    ) -> Result<WorthQueryAdmittedWorkflowStageEvidence, WorthQueryWorkflowAdvanceDenial> {
        self.validate_stage_lineage(stage, &input)?;
        self.validate_primary_reads(stage, &mut input)?;
        self.validate_effects(stage, &mut input)?;
        let invariant_outcomes = self.validate_invariants(stage, &input)?;
        self.validate_result_contracts(stage, &input)?;
        let output_semantics = input.material.output.semantic_value();
        let stage_counters = self.counters.delta_since(input.counters_before);
        self.validate_cost_contract(stage, stage_counters, &input)?;
        WorthQueryValidatedWorkflowEvidenceCompletion {
            run: self,
            stage,
            input,
            invariant_outcomes,
            output_semantics,
            stage_counters,
        }
        .finish()
    }

    fn validate_stage_lineage(
        &self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        input: &WorthQueryWorkflowStageCompletion,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        let valid = super::super::workflow_lineage_validation::valid_stage_lineage(
            &input.material.lineage,
            self.bound.definition().semantics().lineage,
            &input.conditional,
            self.bound.definition().canonical_identity(),
            &self.identity,
            stage.identity(),
            &input.material.effects,
        );
        valid.then_some(()).ok_or_else(|| {
            input.denial(
                self.counters,
                WorthQueryWorkflowAdvanceDenialKind::LineageEvidence,
            )
        })
    }

    fn validate_primary_reads(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        input: &mut WorthQueryWorkflowStageCompletion,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        input
            .material
            .primary_graph_reads
            .sort_by(|left, right| left.role().cmp(right.role()));
        let mut expected = self
            .bound
            .definition()
            .semantics()
            .graph_reads
            .domain_roles()
            .iter()
            .filter(|read| {
                stage.semantics().graph_read_roles.contains(&read.role)
                    && read.participation
                        == WorthQueryOperationGraphParticipation::PrimaryLogicalGraph
            })
            .map(|read| read.role.as_str())
            .collect::<Vec<_>>();
        expected.sort();
        let evidence_roles = input
            .material
            .primary_graph_reads
            .iter()
            .map(|evidence| evidence.role())
            .collect::<Vec<_>>();
        let valid = evidence_roles == expected
            && input.material.primary_graph_reads.iter().all(|evidence| {
                evidence.validates(
                    &self.bound.definition().semantics().canonical_query,
                    self.bound.basis().normalized().family(),
                    &input.execution_snapshot,
                    self.bound
                        .operation()
                        .domain_authority()
                        .runtime_authority(),
                )
            });
        if !valid {
            return Err(input.denial(
                self.counters,
                WorthQueryWorkflowAdvanceDenialKind::PrimaryReadEvidence,
            ));
        }
        self.counters.graph_read_contacts += input.material.primary_graph_reads.len();
        Ok(())
    }

    fn validate_effects(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        input: &mut WorthQueryWorkflowStageCompletion,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        input.material.effects.sort_by_key(|effect| effect.family());
        let mut expected = stage.semantics().effect_roles.clone();
        expected.sort();
        let evidence = input
            .material
            .effects
            .iter()
            .map(|effect| effect.family())
            .collect::<Vec<_>>();
        let valid = evidence == expected
            && input.material.effects.iter().all(|effect| {
                effect.binds_workflow(
                    &input.effect_workflow_binding,
                    self.bound.basis().normalized().family(),
                )
            });
        if !valid {
            return Err(input.denial(
                self.counters,
                WorthQueryWorkflowAdvanceDenialKind::EffectEvidence,
            ));
        }
        self.counters.effect_receipt_checks += input.material.effects.len();
        Ok(())
    }

    fn validate_invariants(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        input: &WorthQueryWorkflowStageCompletion,
    ) -> Result<Vec<WorthQueryWorkflowInvariantOutcome>, WorthQueryWorkflowAdvanceDenial> {
        if !stage.semantics().invariant_roles.is_empty() && input.material.effects.is_empty() {
            return Err(input.denial(
                self.counters,
                WorthQueryWorkflowAdvanceDenialKind::InvariantEvidence,
            ));
        }
        let outcomes = stage
            .semantics()
            .invariant_roles
            .iter()
            .map(|role| {
                let installed_identity = self
                    .bound
                    .operation()
                    .domain_authority()
                    .installed_invariant_identity(role)
                    .ok_or_else(|| {
                        input.denial(
                            self.counters,
                            WorthQueryWorkflowAdvanceDenialKind::InvariantEvidence,
                        )
                    })?;
                Ok(WorthQueryWorkflowInvariantOutcome::from_query_commits(
                    role,
                    installed_identity,
                    &input.material.effects,
                ))
            })
            .collect::<Result<Vec<_>, WorthQueryWorkflowAdvanceDenial>>()?;
        self.counters.invariant_checks += outcomes.len();
        Ok(outcomes)
    }

    fn validate_result_contracts(
        &mut self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        input: &WorthQueryWorkflowStageCompletion,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        self.counters.output_contract_checks += 1;
        if !input.material.output.satisfies(&stage.semantics().output) {
            return Err(input.denial(
                self.counters,
                WorthQueryWorkflowAdvanceDenialKind::OutputContract,
            ));
        }
        if let Err(denial) = self.validate_artifact_output(stage, &input.material.output) {
            return Err(input.denial(
                self.counters,
                WorthQueryWorkflowAdvanceDenialKind::ArtifactCarriage(denial),
            ));
        }
        self.counters.terminal_contract_checks += 1;
        let state = input.material.result_state;
        if stage.is_terminal() != state.is_some()
            || state.is_some_and(|state| !stage.semantics().terminal_result_states.contains(&state))
        {
            return Err(input.denial(
                self.counters,
                WorthQueryWorkflowAdvanceDenialKind::TerminalContract,
            ));
        }
        Ok(())
    }

    fn validate_artifact_output(
        &self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        output: &super::WorthQueryWorkflowValue,
    ) -> Result<(), crate::domain_installation::WorthQueryArtifactDenial> {
        let worth_query_installation::facade::WorthQueryWorkflowValueContract::InstalledArtifact(_) =
            &stage.semantics().output
        else {
            return Ok(());
        };
        let super::WorthQueryWorkflowValue::InstalledArtifact(handle) = output else {
            return Err(crate::domain_installation::WorthQueryArtifactDenial::new(
                crate::domain_installation::WorthQueryArtifactDenialKind::StageMismatch,
                None,
                "artifact workflow output must be an owned managed handle",
            ));
        };
        let admission = self
            .managed_run()
            .artifacts()
            .output_validation_admission(stage.identity())?;
        handle.validate_output(&admission)
    }

    fn validate_cost_contract(
        &self,
        stage: &worth_query_installation::facade::WorthQueryPortableWorkflowStage,
        stage_counters: WorthQueryWorkflowRunCounters,
        input: &WorthQueryWorkflowStageCompletion,
    ) -> Result<(), WorthQueryWorkflowAdvanceDenial> {
        let mut declared = stage.semantics().cost_roles.clone();
        declared.sort();
        (declared == stage_counters.observed_cost_roles())
            .then_some(())
            .ok_or_else(|| {
                input.denial(
                    self.counters,
                    WorthQueryWorkflowAdvanceDenialKind::CostContract,
                )
            })
    }
}
