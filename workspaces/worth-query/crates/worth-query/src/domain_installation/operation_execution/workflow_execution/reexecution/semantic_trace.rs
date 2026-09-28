use super::{
    WorthQueryCompletedWorkflowTrace, WorthQueryConditionalTraceMeaning,
    WorthQueryDomainEvidenceReplayMeaning, WorthQueryWorkflowRunCounters,
    WorthQueryWorkflowSemanticValue, WorthQueryWorkflowStageWarning,
};
use crate::basis_lifecycle::BasisOperationLane;
use crate::identity_evolution::InstalledIdentityEvolutionOutcome;
use crate::memory_workspace::{WorthQueryEntityIdentity, WorthQueryMutationDelta};
use crate::runtime::{
    WorthQueryAspectMutationOperation, WorthQueryContinuityMutationEvidence,
    WorthQueryNamingMutationEvidence,
};
use worth_query_installation::facade::{
    WorthQueryOperationEffectFamily, WorthQueryOperationResultState,
};
#[path = "semantic_trace/comparison.rs"]
mod comparison;
pub use comparison::compare_exact_workflow_traces;
pub(crate) use comparison::compare_exact_workflow_traces_counted;

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryMutationTraceMeaning {
    target_entity: Option<WorthQueryEntityIdentity>,
    target_collection: Option<String>,
    deltas: Vec<WorthQueryMutationDelta>,
    declared_aspect_operations: Vec<WorthQueryAspectMutationOperation>,
    declared_aspect_value_digest: Option<String>,
    naming: Option<WorthQueryNamingMutationEvidence>,
    continuity: Option<WorthQueryContinuityMutationEvidence>,
}

impl WorthQueryMutationTraceMeaning {
    pub fn target_entity(&self) -> Option<&WorthQueryEntityIdentity> {
        self.target_entity.as_ref()
    }

    pub fn target_collection(&self) -> Option<&str> {
        self.target_collection.as_deref()
    }

    pub fn deltas(&self) -> &[WorthQueryMutationDelta] {
        &self.deltas
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryEffectTraceMeaning {
    family: WorthQueryOperationEffectFamily,
    mutation: Option<WorthQueryMutationTraceMeaning>,
}

impl WorthQueryEffectTraceMeaning {
    pub const fn family(&self) -> WorthQueryOperationEffectFamily {
        self.family
    }

    pub fn mutation(&self) -> Option<&WorthQueryMutationTraceMeaning> {
        self.mutation.as_ref()
    }
}

#[derive(Clone, Debug)]
pub struct WorthQueryLineageTraceMeaning {
    outcome: InstalledIdentityEvolutionOutcome,
    effect_indices: Vec<usize>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WorthQueryPublicationTraceMeaning {
    projection_role: String,
    stage_identity: String,
    output: WorthQueryWorkflowSemanticValue,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryInvariantTraceMeaning {
    invariant_role: String,
    installed_invariant_identity: String,
    effect_indices: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct WorthQueryWorkflowStageTraceSemantics {
    stage_identity: String,
    predecessor_stage_identities: Vec<String>,
    input: WorthQueryWorkflowSemanticValue,
    output: WorthQueryWorkflowSemanticValue,
    result_state: Option<WorthQueryOperationResultState>,
    warnings: Vec<WorthQueryWorkflowStageWarning>,
    effects: Vec<WorthQueryEffectTraceMeaning>,
    invariants: Vec<WorthQueryInvariantTraceMeaning>,
    conditional_path: Vec<WorthQueryConditionalTraceMeaning>,
    lineage: Vec<WorthQueryLineageTraceMeaning>,
    domain_evidence: Option<super::WorthQueryDomainEvidenceReplayMeaning>,
}

impl WorthQueryWorkflowStageTraceSemantics {
    pub fn stage_identity(&self) -> &str {
        &self.stage_identity
    }
    pub fn input(&self) -> &WorthQueryWorkflowSemanticValue {
        &self.input
    }
    pub fn output(&self) -> &WorthQueryWorkflowSemanticValue {
        &self.output
    }
    pub fn result_state(&self) -> Option<WorthQueryOperationResultState> {
        self.result_state
    }
    pub fn effects(&self) -> &[WorthQueryEffectTraceMeaning] {
        &self.effects
    }
    pub fn conditional_path(&self) -> &[WorthQueryConditionalTraceMeaning] {
        &self.conditional_path
    }
    pub fn lineage(&self) -> &[WorthQueryLineageTraceMeaning] {
        &self.lineage
    }
}

#[derive(Clone, Debug)]
pub struct WorthQueryWorkflowTraceSemantics {
    operation_identity: String,
    conditional_path: Vec<super::WorthQueryConditionalTraceMeaning>,
    stages: Vec<WorthQueryWorkflowStageTraceSemantics>,
    publication: Option<WorthQueryPublicationTraceMeaning>,
}

impl WorthQueryWorkflowTraceSemantics {
    pub fn operation_identity(&self) -> &str {
        &self.operation_identity
    }
    pub fn stages(&self) -> &[WorthQueryWorkflowStageTraceSemantics] {
        &self.stages
    }
    pub fn operation_conditional_path(&self) -> &[super::WorthQueryConditionalTraceMeaning] {
        &self.conditional_path
    }
}

pub type WorthQueryReplayNoiseContract =
    worth_query_installation::facade::WorthQueryOperationReplayNoiseContract;

/// A semantic difference found between an original workflow trace and its
/// replay. Variants with a `stage` name the stage that differed.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReplayDivergence {
    /// The traces belong to different operations.
    Operation,
    /// The operation-level conditional path differed.
    OperationConditionalPath,
    /// The set of stages differed.
    StageSet,
    /// The stage's predecessor stages differed.
    PredecessorTopology { stage: String },
    /// The stage's input differed.
    Input { stage: String },
    /// The stage's output differed.
    Output { stage: String },
    /// The stage's result state differed.
    ResultState { stage: String },
    /// The stage's diagnostic warnings differed and the installed noise
    /// contract does not tolerate that.
    Diagnostic { stage: String },
    /// The stage's domain evidence differed.
    DomainEvidence { stage: String },
    /// The stage's effects differed.
    Effect { stage: String },
    /// The stage's invariant results differed.
    Invariant { stage: String },
    /// The stage's conditional path differed, or the replay deferred at it.
    ConditionalPath { stage: String },
    /// The stage's lineage evidence differed.
    Lineage { stage: String },
    /// The published output differed.
    Publication,
    /// The semantic aspect dependency closures were missing or did not
    /// converge.
    DependencyClosure,
}

/// The verdict of comparing an original workflow trace with its replay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryReplayComparison {
    /// No semantic difference was found.
    Equivalent,
    /// The traces differ as described.
    Diverged(WorthQueryReplayDivergence),
}

impl<D, O, F, L: BasisOperationLane> WorthQueryCompletedWorkflowTrace<D, O, F, L> {
    pub fn semantics(&self) -> WorthQueryWorkflowTraceSemantics {
        let lineage_by_stage = lineage_evidence_by_stage(self);
        let mut stages = self
            .stage_receipts()
            .iter()
            .map(|receipt| stage_semantics(receipt, &lineage_by_stage))
            .collect::<Vec<_>>();
        stages.sort_by(|left, right| left.stage_identity.cmp(&right.stage_identity));
        let publication = publication_meaning(self, &stages);
        WorthQueryWorkflowTraceSemantics {
            operation_identity: self.run.bound.definition().canonical_identity().to_owned(),
            conditional_path: self
                .operation_conditional_provenance()
                .iter()
                .map(super::workflow_conditional_trace::conditional_trace_meaning)
                .collect(),
            stages,
            publication,
        }
    }

    pub(crate) fn exact_counters(&self) -> WorthQueryWorkflowRunCounters {
        self.counters()
    }
}

type LineageEvidenceByStage<'a> = std::collections::BTreeMap<
    &'a str,
    Vec<&'a crate::domain_installation::WorthQueryTraceLineageEvidence>,
>;

fn lineage_evidence_by_stage<D, O, F, L: BasisOperationLane>(
    trace: &WorthQueryCompletedWorkflowTrace<D, O, F, L>,
) -> LineageEvidenceByStage<'_> {
    let mut by_stage = LineageEvidenceByStage::new();
    if let Some(report) = trace.lineage_report() {
        for evidence in report.evidence() {
            by_stage
                .entry(evidence.stage_identity())
                .or_default()
                .push(evidence);
        }
    }
    by_stage
}

fn stage_semantics(
    receipt: &super::WorthQueryWorkflowStageReceipt,
    lineage_by_stage: &LineageEvidenceByStage<'_>,
) -> WorthQueryWorkflowStageTraceSemantics {
    let effect_index = receipt
        .effect_evidence()
        .iter()
        .enumerate()
        .map(|(index, effect)| (effect.receipt_identity(), index))
        .collect::<std::collections::BTreeMap<_, _>>();
    WorthQueryWorkflowStageTraceSemantics {
        stage_identity: receipt.stage_identity().to_owned(),
        predecessor_stage_identities: receipt.predecessor_stage_identities().to_vec(),
        input: receipt.input().clone(),
        output: receipt.output_semantics().clone(),
        result_state: receipt.result_state(),
        warnings: receipt.warnings().to_vec(),
        effects: receipt
            .effect_evidence()
            .iter()
            .map(effect_meaning)
            .collect(),
        invariants: invariant_meanings(receipt, &effect_index),
        conditional_path: receipt
            .conditional_provenance()
            .iter()
            .map(super::workflow_conditional_trace::conditional_trace_meaning)
            .collect(),
        lineage: lineage_meanings(receipt, lineage_by_stage, &effect_index),
        domain_evidence: receipt
            .domain_evidence()
            .map(super::WorthQueryAdmittedDomainEvidence::replay_meaning),
    }
}

fn invariant_meanings(
    receipt: &super::WorthQueryWorkflowStageReceipt,
    effect_index: &std::collections::BTreeMap<&str, usize>,
) -> Vec<WorthQueryInvariantTraceMeaning> {
    receipt
        .invariant_outcomes()
        .iter()
        .map(|outcome| {
            let mut effect_indices = outcome
                .effect_receipt_identities()
                .iter()
                .map(|identity| {
                    *effect_index
                        .get(identity.as_str())
                        .expect("invariant admission validated every effect receipt identity")
                })
                .collect::<Vec<_>>();
            effect_indices.sort_unstable();
            WorthQueryInvariantTraceMeaning {
                invariant_role: outcome.invariant_role().to_owned(),
                installed_invariant_identity: outcome.installed_invariant_identity().to_owned(),
                effect_indices,
            }
        })
        .collect()
}

fn lineage_meanings(
    receipt: &super::WorthQueryWorkflowStageReceipt,
    lineage_by_stage: &LineageEvidenceByStage<'_>,
    effect_index: &std::collections::BTreeMap<&str, usize>,
) -> Vec<WorthQueryLineageTraceMeaning> {
    lineage_by_stage
        .get(receipt.stage_identity())
        .into_iter()
        .flat_map(|evidence| evidence.iter().copied())
        .map(|evidence| WorthQueryLineageTraceMeaning {
            outcome: evidence.outcome().clone(),
            effect_indices: evidence
                .effect_receipt_identities()
                .iter()
                .map(|identity| {
                    *effect_index
                        .get(identity.as_str())
                        .expect("lineage binding validated every effect receipt identity")
                })
                .collect(),
        })
        .collect()
}

fn effect_meaning(
    effect: &super::WorthQueryWorkflowEffectEvidence,
) -> WorthQueryEffectTraceMeaning {
    let mutation = effect
        .mutation_receipt()
        .map(|receipt| WorthQueryMutationTraceMeaning {
            target_entity: receipt.target_entity_identity().cloned(),
            target_collection: receipt
                .target_collection_identity()
                .map(|identity| identity.as_str().to_owned()),
            deltas: receipt.deltas().to_vec(),
            declared_aspect_operations: receipt.declared_aspect_operations().to_vec(),
            declared_aspect_value_digest: receipt.declared_aspect_value_digest().map(str::to_owned),
            naming: receipt.naming_mutation_evidence().cloned(),
            continuity: receipt.continuity_mutation_evidence().cloned(),
        });
    WorthQueryEffectTraceMeaning {
        family: effect.family(),
        mutation,
    }
}

fn publication_meaning<D, O, F, L: BasisOperationLane>(
    trace: &WorthQueryCompletedWorkflowTrace<D, O, F, L>,
    stages: &[WorthQueryWorkflowStageTraceSemantics],
) -> Option<WorthQueryPublicationTraceMeaning> {
    let worth_query_installation::facade::WorthQueryOperationPublicationContract::DerivedProjection {
        projection_role,
    } = &trace.bound().definition().semantics().publication
    else {
        return None;
    };
    trace
        .run
        .graph
        .stages()
        .iter()
        .find(|stage| stage.is_publishable())
        .and_then(|publication_stage| {
            stages
                .iter()
                .find(|stage| stage.stage_identity == publication_stage.identity())
        })
        .map(|stage| WorthQueryPublicationTraceMeaning {
            projection_role: projection_role.as_str().to_owned(),
            stage_identity: stage.stage_identity.clone(),
            output: stage.output.clone(),
        })
}

#[cfg(test)]
#[path = "semantic_trace_tests.rs"]
mod tests;
