use crate::domain_computation::primary_graph::application_attempt::check_request_live;
use crate::domain_computation::primary_graph::application_checkpoint::facts::FactSequence;
use worth_query_installation::facade::ApplicationSchema;

use super::{
    publication::PreparedWorkflowAssessmentProjection, PreparedWorkflowAdvance,
    PreparedWorkflowAssessment,
};
use crate::domain_computation::primary_graph::workflow::{
    visit_workflow_assessment_facts, WorkflowAssessmentEvidenceMeaning,
};
use crate::domain_computation::primary_graph::{
    RequiredWorkflowAssessment, WorthQueryApplicationAttemptDenial,
    WorthQueryApplicationAttemptDenialKind, WorthQueryApplicationEffectProgram,
    WorthQueryObservedSource, WorthQueryOutputDemandSettlement,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryWorkflowAssessmentPosture,
};

#[path = "assessment/evidence_budget.rs"]
mod evidence_budget;
#[path = "assessment/identity.rs"]
mod identity;
#[path = "assessment/replay.rs"]
mod replay;

use identity::{decode_hex_identity, evidence_meaning};

impl<Schema, Operation, Input, Scope> PreparedWorkflowAssessment<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    pub(super) fn settle<Query>(
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
        let currentness_facts = validate(execution, runtime, &self, settlement, source)?;
        let mut durable_currentness_facts = currentness_facts
            .iter()
            .filter(|fact| {
                matches!(
                fact,
                crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::Entity { .. }
                    | crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceEntity { .. }
                    | crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceAspectRevision { .. }
                    | crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceFieldRevision { .. }
                    | crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
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
            read_set.admission.publication_request(),
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
        let (demand, retained_bytes) = evidence_budget::admit(
            &self.layout,
            self.evidence_allowance,
            self.required.node_path(),
            |observe| {
                visit_workflow_assessment_facts(&self.layout, &self.admitted, &meaning, observe)
                    .map(|_| ())
            },
        )?;
        meaning.retained_bytes = retained_bytes;
        let reservation = super::admit_platform_effects(self.admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_assessment_facts(&self.layout, &self.admitted, &meaning, |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        })?;
        reservation.materialize(&effects)?;
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
        let program = WorthQueryApplicationEffectProgram {
            read_set,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            effect_posture: crate::domain_computation::provider_session::WorthQueryApplicationEffectPosture::Platform,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: Some(currentness_facts.into()),
        };
        let assessment = Some(projection(&self.layout, meaning));
        let replays = self.replays;
        Ok(PreparedWorkflowAdvance::Transition {
            program,
            program_revision: self.program_revision,
            transition_identity,
            transition_identity_bytes,
            transition_identity_locator: self.layout.transition.identity.clone(),
            assessment_identity_locator: self.layout.assessment_evidence.identity.clone(),
            instance: self.required.instance(),
            node_path,
            assessment,
            supporting_identity: None,
            operation_receipt_identity: None,
            progress_update: Some(progress_update),
            terminal: false,
            approval: None,
            approval_identity: None,
            approval_authentication: None,
            replays,
        })
    }
}

pub(super) fn bind_currentness_facts<Schema, Operation, Input, Scope>(
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
    check_request_live(read_set.admission.publication_request(), subject)?;
    ensure_current(execution, read_set, currentness_facts, subject)?;
    let mut merged = std::collections::BTreeMap::new();
    // Full canonical workflow merge, retaining original bodies by position.
    for (ordinal, fact) in read_set.facts.iter().enumerate() {
        let locator = fact.dependency_key();
        match merged.entry(locator) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert((false, ordinal));
            }
            std::collections::btree_map::Entry::Occupied(entry) => {
                let (_, existing) = *entry.get();
                if read_set.facts.get(existing) != Some(fact) {
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
                    read_set.facts.get(position)
                };
                if existing != Some(fact) {
                    return Err(currentness_mismatch(subject));
                }
            }
        }
    }
    // New evidence is an explicit full workflow owner; original Fact bodies
    // are never recopied merely to put them in canonical locator order.
    let policy = worth_execution::ExecutionAllocationPolicy::SystemAllocation;
    let request = read_set.admission.publication_request();
    let check = || {
        read_set
            .admission
            .validate_current_authority()
            .map_err(WorthQueryApplicationAttemptDenial::request_authority_lost)
    };
    let tail = super::super::read_set::admit_completed_facts(
        appended.len(),
        appended.into_iter().map(|ordinal| {
            currentness_facts
                .get(ordinal)
                .expect("indexed currentness fact")
                .clone()
        }),
        policy,
        request,
        subject,
        check,
    )?;
    let control = super::super::retained_decision_facts::StorageControl::new(policy, Some(request));
    let mut joined = super::super::read_set::admit_completed_facts(
        0,
        std::iter::empty(),
        policy,
        request,
        subject,
        check,
    )?;
    for (additional, ordinal) in merged.into_values() {
        check()?;
        let backing = if additional { &tail } else { &read_set.facts };
        joined = joined
            .append_completed(backing.slice(ordinal..ordinal + 1), control)
            .map_err(|cause| cause.into_attempt_denial(subject))?;
    }
    check()?;
    read_set.facts = joined;
    Ok(())
}

/// Denies unless every currentness fact still holds in the read set's
/// snapshot: evidence observed before a later write is stale.
pub(super) fn ensure_current<Schema, Operation, Input, Scope>(
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
    let current = read_set.lease.handle().with_runtime(execution, |runtime| {
        for fact in currentness_facts.iter() {
            check_request_live(read_set.admission.publication_request(), subject)?;
            if !fact.remains_equal_in(runtime, execution, read_set.lease.snapshot())? {
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
        .lease
        .product()
        .product_branch();
    let authority_matches = if let Some(receipt) = settlement.application_commit_receipt() {
        source.selected_product_occurrence() == Some(receipt.product_branch().occurrence())
            && receipt.idempotency_binding().source_identity()
                == Some(source.idempotency_identity().bytes())
            && receipt.product_branch() == fresh_branch
    } else if let Some(stable) = settlement.stable.as_ref() {
        source.selected_product_occurrence() == Some(stable.observation().lifecycle_incarnation())
            && stable.source_identity()
                == Some(crate::domain_computation::primary_graph::output_lineage::RecordedSourceIdentity::Runtime(
                    source.idempotency_identity(),
                ))
            && stable.source_partition_identity() == Some(source.partition_identity())
            && crate::basis::WorthQueryProductBranch::from_occurrence(
                stable.observation().lifecycle_incarnation(),
            ) == fresh_branch
    } else {
        false
    };
    let valid = settlement.belongs_to(runtime)
        && source.source_root() == prepared.admitted.subject()
        && source.query_identifier.as_str() == prepared.required.query()
        && authority_matches;
    if !valid {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
            prepared.required.node_path(),
        ));
    }
    runtime
        .current_output_source_facts(execution, settlement)
        .map_err(|_| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
                prepared.required.node_path(),
            )
        })
}

fn projection(
    layout: &crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout,
    meaning: WorkflowAssessmentEvidenceMeaning,
) -> PreparedWorkflowAssessmentProjection {
    projection_with_locator(&layout.assessment_evidence.identity, meaning)
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
