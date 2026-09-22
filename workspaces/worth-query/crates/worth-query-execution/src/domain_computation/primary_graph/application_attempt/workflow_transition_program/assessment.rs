use sha2::{Digest, Sha256};

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

#[path = "assessment/replay.rs"]
mod replay;

impl<Schema, Operation, Input, Scope> PreparedWorkflowAssessment<Schema, Operation, Input, Scope>
where
    Schema: ApplicationSchema,
    Operation: 'static,
{
    pub(super) fn settle<Query>(
        self,
        runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        settlement: &WorthQueryOutputDemandSettlement,
        source: &WorthQueryObservedSource<Query>,
        posture: WorthQueryWorkflowAssessmentPosture,
    ) -> Result<
        PreparedWorkflowAdvance<Schema, Operation, Input, Scope>,
        WorthQueryApplicationAttemptDenial,
    > {
        let currentness_facts = validate(runtime, &self, settlement, source)?;
        let durable_currentness_facts = currentness_facts
            .iter()
            .filter(|fact| {
                matches!(
                fact,
                crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceEntity { .. }
                    | crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceAspectRevision { .. }
                    | crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact::SourceAdjacencyRevision { .. }
                )
            })
            .cloned()
            .collect::<std::sync::Arc<[_]>>();
        let read_set = self.admitted.read_set();
        if read_set.facts.len().saturating_add(
            crate::domain_computation::primary_graph::workflow::evidence_dependency::evidence_dependency_observation_facts(
                durable_currentness_facts.len(),
            ),
        ) > read_set
            .admission
            .allowed_graph_contract()
            .decision_fact_budget()
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
                self.required.node_path(),
            ));
        }
        let meaning = evidence_meaning(
            &self.required,
            self.admitted.subject(),
            settlement,
            source,
            posture,
            durable_currentness_facts,
        );
        let transition_identity = self.admitted.identity().to_owned();
        let transition_identity_bytes = *self.admitted.identity_bytes();
        let node_path = self.admitted.node_path().to_owned();
        let mut demand = super::PlatformEffectDemand::default();
        visit_workflow_assessment_facts(&self.layout, &self.admitted, &meaning, |effect| {
            demand.observe(&effect)
        })?;
        let reservation = super::admit_platform_effects(self.admitted.read_set(), demand)?;
        let mut effects = Vec::new();
        visit_workflow_assessment_facts(&self.layout, &self.admitted, &meaning, |effect| {
            effects.push(effect);
            Ok::<(), WorthQueryApplicationAttemptDenial>(())
        })?;
        let validator_work_admission = reservation.materialize(&effects)?;
        let mut read_set = self.admitted.into_read_set();
        bind_currentness_facts(&mut read_set, &currentness_facts, &node_path)?;
        let program = WorthQueryApplicationEffectProgram {
            read_set,
            effects,
            emission_retained_bytes: 0,
            emission_retained_bytes_ceiling: 0,
            conditional_definition: None,
            platform_mutation: true,
            validator_work_admission,
            output_correspondence: Default::default(),
            retain_output_demand_observation: false,
            retain_client_observation: false,
            producer_required_invariants: &[],
            output_currentness_facts: Some(currentness_facts),
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
            approval: None,
            approval_identity: None,
            replays,
        })
    }
}

pub(super) fn bind_currentness_facts<Schema, Operation, Input, Scope>(
    read_set: &mut super::super::WorthQueryCompleteApplicationReadSet<
        Schema,
        Operation,
        Input,
        Scope,
        super::super::WorthQueryProjectedApplicationMutation,
    >,
    currentness_facts: &[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
    subject: &str,
) -> Result<(), WorthQueryApplicationAttemptDenial> {
    let fact_budget = read_set
        .admission
        .allowed_graph_contract()
        .decision_fact_budget();
    if read_set.facts.len().saturating_add(currentness_facts.len()) > fact_budget {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
            subject,
        ));
    }
    let currentness_matches = read_set.lease.handle().with_runtime(|runtime| {
        currentness_facts
            .iter()
            .all(|fact| fact.remains_equal_in(runtime, read_set.lease.snapshot()))
    });
    if !currentness_matches {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
            subject,
        ));
    }
    let mut merged = std::collections::BTreeMap::new();
    for fact in std::mem::take(&mut read_set.facts) {
        let locator = fact.locator_identity();
        match merged.entry(locator) {
            std::collections::btree_map::Entry::Vacant(entry) => {
                entry.insert(fact);
            }
            std::collections::btree_map::Entry::Occupied(entry) if entry.get() != &fact => {
                return Err(WorthQueryApplicationAttemptDenial::new(
                    WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
                    subject,
                ));
            }
            std::collections::btree_map::Entry::Occupied(_) => {}
        }
    }
    for fact in currentness_facts {
        let locator = fact.locator_identity();
        if merged
            .insert(locator, fact.clone())
            .is_some_and(|existing| existing != *fact)
        {
            return Err(WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
                subject,
            ));
        }
    }
    read_set.facts = merged.into_values().collect();
    Ok(())
}

fn validate<Schema, Operation, Input, Scope, Query>(
    runtime: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    prepared: &PreparedWorkflowAssessment<Schema, Operation, Input, Scope>,
    settlement: &WorthQueryOutputDemandSettlement,
    source: &WorthQueryObservedSource<Query>,
) -> Result<
    std::sync::Arc<[crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact]>,
    WorthQueryApplicationAttemptDenial,
>
where
    Schema: ApplicationSchema,
{
    let receipt = settlement.receipt();
    let fresh_branch = prepared
        .admitted
        .read_set()
        .lease
        .product()
        .product_branch();
    let valid = settlement.belongs_to(runtime)
        && source.source_root() == prepared.admitted.subject()
        && source.query_identifier == prepared.required.query()
        && source.selected_product_occurrence() == Some(receipt.product_branch().occurrence())
        && receipt.idempotency_binding().source_identity() == Some(source.idempotency_identity())
        && receipt.product_branch() == fresh_branch;
    if !valid {
        return Err(WorthQueryApplicationAttemptDenial::new(
            WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
            prepared.required.node_path(),
        ));
    }
    runtime
        .current_output_source_facts(settlement)
        .map_err(|_| {
            WorthQueryApplicationAttemptDenial::new(
                WorthQueryApplicationAttemptDenialKind::WorkflowAssessmentEvidenceMismatch,
                prepared.required.node_path(),
            )
        })
}

fn evidence_meaning<Query>(
    required: &RequiredWorkflowAssessment,
    subject: worth_relational::facade::identity::EntityId,
    settlement: &WorthQueryOutputDemandSettlement,
    source: &WorthQueryObservedSource<Query>,
    posture: WorthQueryWorkflowAssessmentPosture,
    currentness_facts: std::sync::Arc<
        [crate::domain_computation::primary_graph::WorthQueryApplicationObservedFact],
    >,
) -> WorkflowAssessmentEvidenceMeaning {
    let receipt = settlement.receipt();
    let publication = receipt.committed_product_publication();
    let source_identity = hex(source.idempotency_identity());
    let output_content_identity = output_content_identity(receipt, source.idempotency_identity());
    let publication_identity = format!(
        "{}:{}:{}:{}:{}:{}:{}:{}:{}",
        publication.product_branch().owner_identity().get(),
        publication.product_branch().name().as_str(),
        publication.product_incarnation().ordinal(),
        publication.product_generation().get(),
        publication.composite_commit().ordinal(),
        publication.publication_attempt().ordinal(),
        publication.relational_commit().branch_id.0,
        publication.relational_commit().commit_id.0,
        publication.relational_commit().version_id.0,
    );
    let passing = posture == WorthQueryWorkflowAssessmentPosture::Passing;
    let mut digest = Sha256::new();
    digest.update(b"worth-query:workflow-assessment-evidence:v1");
    for value in [
        required.transition_identity(),
        settlement.producer_identity(),
        settlement.output_family_identity(),
        required.query(),
        required.parameter_type(),
        required.result_type(),
        required.binding(),
        required.proposal_identity(),
        required.coverage_identity(),
        &source_identity,
        &publication_identity,
        &output_content_identity,
    ] {
        digest.update((value.len() as u64).to_le_bytes());
        digest.update(value.as_bytes());
    }
    digest.update(subject.partition_value_u64().to_le_bytes());
    digest.update(subject.local_slot_value().to_le_bytes());
    digest.update(u64::from(subject.generation_value()).to_le_bytes());
    digest.update([u8::from(passing)]);
    WorkflowAssessmentEvidenceMeaning {
        identity: hex(digest.finalize().into()),
        producer: settlement.producer_identity().to_owned(),
        family: settlement.output_family_identity().to_owned(),
        query: required.query().to_owned(),
        parameter_type: required.parameter_type().to_owned(),
        result_type: required.result_type().to_owned(),
        binding: required.binding().to_owned(),
        subject,
        proposal_identity: required.proposal_identity().to_owned(),
        coverage_identity: required.coverage_identity().to_owned(),
        source_identity,
        passing,
        publication_identity,
        output_content_identity,
        currentness_facts,
    }
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

fn output_content_identity(
    receipt: &crate::domain_computation::primary_graph::WorthQueryApplicationCommitReceipt,
    source_identity: [u8; 32],
) -> String {
    let publication = receipt.committed_product_publication();
    let mut digest = Sha256::new();
    digest.update(b"worth-query:workflow-assessment-output-content:v1");
    digest.update(receipt.output_correspondence().workflow_content_identity());
    digest.update(source_identity);
    digest.update(receipt.installed_operation());
    digest.update(
        publication
            .product_branch()
            .owner_identity()
            .get()
            .to_le_bytes(),
    );
    digest.update(publication.product_branch().name().as_str().as_bytes());
    digest.update(publication.product_incarnation().ordinal().to_le_bytes());
    digest.update(publication.composite_commit().ordinal().to_le_bytes());
    digest.update(publication.relational_commit().commit_id.0.to_le_bytes());
    digest.update(publication.relational_commit().version_id.0.to_le_bytes());
    hex(digest.finalize().into())
}

fn hex(bytes: [u8; 32]) -> String {
    use std::fmt::Write;

    let mut text = String::with_capacity(64);
    for byte in bytes {
        write!(&mut text, "{byte:02x}").expect("writing to a String cannot fail");
    }
    text
}

fn decode_hex_identity(text: &str) -> Option<[u8; 32]> {
    if text.len() != 64 {
        return None;
    }
    let mut bytes = [0_u8; 32];
    for (index, byte) in bytes.iter_mut().enumerate() {
        let offset = index * 2;
        *byte = u8::from_str_radix(&text[offset..offset + 2], 16).ok()?;
    }
    Some(bytes)
}
