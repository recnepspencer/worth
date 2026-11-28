use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use crate::domain_computation::primary_graph::OutputDemandSettlementRuntimeExt;
use worth_query_execution_attempt::facade::ports::{
    AdmittedOperationExecutionPort as _, CompleteReadSetFactAppendInternalPort as _,
    CompleteReadSetInternalPort as _, EffectProgramInternalPort as _, EffectProgramParts,
};
use worth_query_execution_lineage::facade::ports::{
    ObservedSourceInternalPort as _, OutputDemandSettlementInternalPort as _,
};
use worth_query_execution_runtime::facade::ports::ProductBranchInternalPort as _;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    publication::PreparedWorkflowAssessmentProjection, PreparedWorkflowAdvance,
    PreparedWorkflowAssessment,
};
use crate::domain_computation::primary_graph::workflow::{
    visit_workflow_assessment_facts, WorkflowAssessmentEvidenceMeaning,
};
use crate::domain_computation::primary_graph::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
    WorthQueryApplicationEffectProgram, WorthQueryObservedSource, WorthQueryOutputDemandSettlement,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryWorkflowAssessmentPosture,
};

pub use worth_query_execution_attempt::facade::domain_computation::primary_graph::application_attempt::workflow_transition_program::assessment::{
    evidence_budget, identity,
};
#[path = "assessment/replay.rs"]
pub mod replay;

use identity::{decode_hex_identity, evidence_meaning};

pub trait PreparedWorkflowAssessmentRuntimeExt<Schema, Operation, Input, Scope>: Sized
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    fn settle<Query>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        settlement: &WorthQueryOutputDemandSettlement,
        source: &WorthQueryObservedSource<Query>,
        posture: WorthQueryWorkflowAssessmentPosture,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    >;
}

impl<Schema, Operation, Input, Scope>
    PreparedWorkflowAssessmentRuntimeExt<Schema, Operation, Input, Scope>
    for PreparedWorkflowAssessment<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    fn settle<Query>(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        settlement: &WorthQueryOutputDemandSettlement,
        source: &WorthQueryObservedSource<Query>,
        posture: WorthQueryWorkflowAssessmentPosture,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        use crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact as Fact;
        let currentness_facts = validate(execution, runtime, &self, settlement, source)?;
        let mut durable_currentness_facts = currentness_facts
            .iter()
            .filter(|f| {
                matches!(
                    f,
                    Fact::Entity { .. }
                        | Fact::SourceEntity { .. }
                        | Fact::SourceAspectRevision { .. }
                        | Fact::AspectPresence { .. }
                        | Fact::SourceFieldRevision { .. }
                        | Fact::SourceAdjacencyRevision { .. }
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        // Conditional membership is a dependency of this assessment's
        // evidence, not just a one-time admission check. Retain only this
        // node's applicability revision, never a sibling's observed facts.
        durable_currentness_facts.extend(self.applicability_dependencies.iter().cloned());
        let durable_currentness_facts: std::sync::Arc<[_]> = durable_currentness_facts.into();
        let read_set = self.admitted.read_set();
        check_request_live(
            read_set.admission().publication_request(),
            self.required.node_path(),
        )?;
        let mut meaning = evidence_meaning(
            &self.required,
            self.admitted.subject(),
            settlement,
            source,
            posture,
            durable_currentness_facts,
        )
        .ok_or_else(|| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
                self.required.node_path(),
            )
        })?;
        let transition_identity = self.admitted.identity().to_owned();
        let transition_identity_bytes = *self.admitted.identity_bytes();
        let node_path = self.admitted.node_path().to_owned();
        let retained_bytes = evidence_budget::admit(
            &self.layout,
            self.evidence_allowance,
            self.required.node_path(),
            |observe| {
                visit_workflow_assessment_facts(&self.layout, &self.admitted, &meaning, observe)
                    .map(|_| ())
            },
        )?;
        meaning.retained_bytes = retained_bytes;

        let mut effects = Vec::new();
        visit_workflow_assessment_facts(&self.layout, &self.admitted, &meaning, |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        })?;

        let progress_update = if self.admitted.is_assessment_collection() {
            self.admitted.prepare_assessment_collection_update()?
        } else {
            self.admitted.prepare_progress_update(
                worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome::Completed,
                None,
            )?
        }
        .charging_evidence(retained_bytes);
        let mut read_set = self.admitted.into_read_set();
        bind_currentness_facts(execution, &mut read_set, &currentness_facts, &node_path)?;
        // These verified dependencies become this assessment's completed
        // program read, including when their original backing was decoded.
        let currentness_facts = super::super::OutputCurrentnessFacts::admit_owned_program_facts(
            currentness_facts.iter().cloned().collect(),
            super::super::retained_decision_facts::StorageControl::new(
                execution.allocation_policy(),
                Some(read_set.admission().publication_request()),
            ),
        )
        .map_err(|cause| cause.into_attempt_denial(&node_path))?;
        let program = WorthQueryApplicationEffectProgram::from_parts(EffectProgramParts {
            synchronous_outputs: Vec::new(),
            authored_stages: Vec::new(),
            producer_round_limits: crate::domain_computation::primary_graph::application_contribution::producer::synchronous_outputs::default_producer_round_bounds(),
            content_attachments: Default::default(),
            read_set,
            effects,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            output_correspondence: Default::default(),
            persistent_names: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: Some(currentness_facts),
        });
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision: self.program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: self.layout.transition.identity.clone(),
            assessment_identity_locator: self.layout.assessment_evidence.identity.clone(),
            instance: self.required.instance(),
            node_path,
            assessment: Some(projection_with_locator(
                &self.layout.assessment_evidence.identity,
                meaning,
            )),
            supporting_identity: None,
            operation_receipt_identity: None,
            progress_update: Some(progress_update),
            terminal: false,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays: self.replays,
        })
    }
}

pub fn bind_currentness_facts<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read_set: &mut super::super::WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        super::super::WorthQueryProjectedApplicationMutation,
    >,
    currentness_facts: &(impl FactSequence + ?Sized),
    subject: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    check_request_live(read_set.admission().publication_request(), subject)?;
    ensure_current(execution, read_set, currentness_facts, subject)?;
    let mut merged = std::collections::BTreeMap::new();
    // Completion's original prefix owns the installed projection ordinals.
    // Index dependencies for conflict checks without rearranging that prefix.
    for (ordinal, fact) in read_set.facts().iter().enumerate() {
        let locator = fact.dependency_key();
        match merged.entry(locator) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((false, ordinal));
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                let (_, existing) = *entry.get();
                if read_set.facts().get(existing) != Some(fact) {
                    return Err(currentness_mismatch(subject));
                }
            }
        }
    }
    let mut appended = Vec::new();
    for (ordinal, fact) in currentness_facts.iter().enumerate() {
        match merged.entry(fact.dependency_key()) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((true, appended.len()));
                appended.push(ordinal);
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                let (additional, position) = *entry.get();
                let existing = if additional {
                    currentness_facts.get(appended[position])
                } else {
                    read_set.facts().get(position)
                };
                if existing != Some(fact) {
                    return Err(currentness_mismatch(subject));
                }
            }
        }
    }
    // The completed-read owner retains original backing and replaces facts
    // with their matching effect routes only after admission succeeds.
    read_set
        .admission()
        .validate_current_authority()
        .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)?;
    read_set.append_completed_facts(
        appended.into_iter().map(|ordinal| {
            currentness_facts
                .get(ordinal)
                .expect("indexed currentness fact")
                .clone()
        }),
        worth_execution::ExecutionAllocationPolicy::SystemAllocation,
    )
}

/// Denies unless every currentness fact still holds in the read set's
/// snapshot: evidence observed before a later write is stale.
pub fn ensure_current<Schema, Operation, Input, Scope>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    read_set: &super::super::WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        super::super::WorthQueryProjectedApplicationMutation,
    >,
    currentness_facts: &(impl FactSequence + ?Sized),
    subject: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let current = read_set
        .lease()
        .handle()
        .with_runtime(execution, |runtime| {
            for fact in currentness_facts.iter() {
                check_request_live(read_set.admission().publication_request(), subject)?;
                if !fact.remains_equal_in(runtime, execution, read_set.lease().snapshot())? {
                    return Ok::<bool, WorthQueryApplicationAttemptDenial>(false);
                }
            }
            Ok(true)
        })?;
    if current {
        Ok(())
    } else {
        Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
            subject,
        ))
    }
}

fn validate<Schema, Operation, Input, Scope, Query>(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: &PreparedWorkflowAssessment<Schema, Operation, Input, Scope>,
    settlement: &WorthQueryOutputDemandSettlement,
    source: &WorthQueryObservedSource<Query>,
) -> Result<
    crate::domain_computation::primary_graph::output_lineage::ComparableSourceFacts,
    WorthQueryApplicationAttemptDenial,
>
where
    Schema: ApplicationSchema,
{
    let fresh_branch = prepared
        .admitted
        .read_set()
        .lease()
        .product()
        .product_branch();
    let authority_matches = if let Some(receipt) = settlement.publication_view() {
        source.selected_product_occurrence() == Some(receipt.product_branch().occurrence())
            && receipt.idempotency_binding().source_identity()
                == Some(source.idempotency_identity().bytes())
            && receipt.product_branch() == fresh_branch
    } else if let Some(stable) = settlement.stable() {
        source.selected_product_occurrence() == Some(stable.observation().lifecycle_incarnation())
            && stable.source_identity().is_some_and(|identity| {
                identity
                    .matches_current(source.idempotency_identity(), source.checkpoint_identity())
            })
            && stable.source_partition_identity() == Some(source.partition_identity())
            && crate::basis::WorthQueryProductBranch::from_occurrence(
                stable.observation().lifecycle_incarnation(),
            ) == fresh_branch
    } else if let Some(restored) = settlement.restored_source() {
        use crate::domain_computation::primary_graph::application_output_demand::RestoredOutputOrigin;
        let identity = match &restored.origin {
            RestoredOutputOrigin::Checkpoint(_) => Some(
                crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Checkpoint(restored.source_identity),
            )
        };
        source.selected_product_occurrence() == Some(restored.observation.lifecycle_incarnation())
            && identity.is_some_and(|identity| {
                identity
                    .matches_current(source.idempotency_identity(), source.checkpoint_identity())
            })
            && restored.origin.partition() == source.partition_identity()
            && crate::basis::WorthQueryProductBranch::from_occurrence(
                restored.observation.lifecycle_incarnation(),
            ) == fresh_branch
    } else {
        false
    };
    let valid = settlement.belongs_to(runtime)
        && source.source_root() == prepared.admitted.subject()
        && source.query_identifier().as_str() == prepared.required.query()
        && authority_matches;
    if !valid {
        return Err(currentness_mismatch(prepared.required.node_path()));
    }
    runtime
        .current_output_source_facts(execution, settlement)
        .map_err(|_| currentness_mismatch(prepared.required.node_path()))
}

fn projection_with_locator(
    identity_locator: &worth_foundational::facade::AspectFieldLocator,
    meaning: WorkflowAssessmentEvidenceMeaning,
) -> PreparedWorkflowAssessmentProjection {
    let intent_identity = decode_hex_identity(&meaning.identity)
        .expect("workflow assessment identity is an encoded SHA-256 digest");
    PreparedWorkflowAssessmentProjection {
        identity_locator: identity_locator.clone(),
        identity: meaning.identity,
        intent_identity,
        producer: meaning.producer,
        family: meaning.family,
        query: meaning.query,
        parameter_type: meaning.parameter_type,
        result_type: meaning.result_type,
        binding: meaning.binding,
        subject: meaning.subject,
        proposal_identity: meaning.proposal_identity,
        coverage_identity: meaning.coverage_identity,
        source_identity: meaning.source_identity,
        passing: meaning.passing,
        publication_identity: meaning.publication_identity,
        output_content_identity: meaning.output_content_identity,
    }
}

fn currentness_mismatch(subject: &str) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
        subject,
    )
}
