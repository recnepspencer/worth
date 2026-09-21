use worth_foundational::facade::{AspectValue, InternedString};
use worth_query_declaration::facade::application_program::ApplicationWorkflowControlOutcome;
use worth_relational::facade::identity::{EntityId, PartitionId, RelationId};

use super::ObservedWorkflowAssessmentEvidence;
use super::{adjacency_with_kind, denial, exact, exact_u64, text};
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::{
    instance::{decode_transition_outcome, encode_transition_outcome, SettledWorkflowTransition},
    schema::WorthQueryWorkflowLayout,
};

mod dependency;
pub(in crate::domain_computation::primary_graph::application_attempt) use dependency::observe_evidence_dependencies;

pub(super) fn observe_settled_transition(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<SettledWorkflowTransition, WorthQueryApplicationAttemptDenial> {
    let kind = layout.transition.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: transition,
        kind,
    });
    exact(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.protocol_version,
        AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        facts,
    )?;
    let occurrence = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.occurrence,
        facts,
    )?;
    let outcome = decode_transition_outcome(exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.outcome,
        facts,
    )?)
    .ok_or_else(|| denial("workflow transition outcome is unsupported"))?;
    let nodes = adjacency_with_kind(
        runtime,
        snapshot,
        layout.transition_node_relation,
        transition,
        2,
        facts,
    )?;
    let [node] = nodes.as_slice() else {
        return Err(denial("workflow transition node binding is not singular"));
    };
    Ok(SettledWorkflowTransition::new(*node, occurrence, outcome))
}

pub(super) fn observe_assessment_evidence(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<ObservedWorkflowAssessmentEvidence>, WorthQueryApplicationAttemptDenial> {
    let evidence = adjacency_with_kind(
        runtime,
        snapshot,
        layout.transition_assessment_evidence_relation,
        transition,
        2,
        facts,
    )?;
    let ([] | [_]) = evidence.as_slice() else {
        return Err(denial(
            "workflow transition has duplicate assessment evidence",
        ));
    };
    let Some(evidence) = evidence.first().copied() else {
        return Ok(None);
    };
    let kind = layout.assessment_evidence.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: evidence,
        kind,
    });
    exact(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.protocol_version,
        AspectValue::UInt64(
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        facts,
    )?;
    let identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.identity,
        facts,
    )?;
    for locator in [
        &layout.assessment_evidence.producer,
        &layout.assessment_evidence.family,
    ] {
        observed_text(runtime, snapshot, evidence, kind, locator, facts)?;
    }
    let source_identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.source_identity,
        facts,
    )?;
    let publication_identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.publication_identity,
        facts,
    )?;
    let output_content_identity = observed_text(
        runtime,
        snapshot,
        evidence,
        kind,
        &layout.assessment_evidence.output_content_identity,
        facts,
    )?;
    for locator in [
        &layout.assessment_evidence.subject_partition,
        &layout.assessment_evidence.subject_slot,
        &layout.assessment_evidence.subject_generation,
    ] {
        observed_u64(runtime, snapshot, evidence, kind, locator, facts)?;
    }
    Ok(Some(ObservedWorkflowAssessmentEvidence {
        entity: evidence,
        identity,
        query: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.query,
            facts,
        )?,
        parameter_type: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.parameter_type,
            facts,
        )?,
        result_type: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.result_type,
            facts,
        )?,
        binding: observed_text(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.binding,
            facts,
        )?,
        passing: observed_bool(
            runtime,
            snapshot,
            evidence,
            kind,
            &layout.assessment_evidence.passing,
            facts,
        )?,
        source_identity,
        publication_identity,
        output_content_identity,
    }))
}

fn observed_text(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<String, WorthQueryApplicationAttemptDenial> {
    let value = super::observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow assessment evidence field is unavailable"))?;
    let retained = value.clone();
    let AspectValue::String(value) = &value else {
        return Err(denial(
            "workflow assessment evidence field has the wrong type",
        ));
    };
    let text = match value {
        InternedString::Raw(value) => value.clone(),
        InternedString::Symbol(_) => {
            return Err(denial(
                "workflow assessment evidence text is not materialized",
            ))
        }
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: retained,
    });
    Ok(text)
}

fn observed_bool(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<bool, WorthQueryApplicationAttemptDenial> {
    let value = super::observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow assessment evidence field is unavailable"))?;
    let AspectValue::Bool(posture) = value else {
        return Err(denial(
            "workflow assessment evidence field has the wrong type",
        ));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::Bool(posture),
    });
    Ok(posture)
}

fn observed_u64(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: worth_relational::facade::identity::KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<u64, WorthQueryApplicationAttemptDenial> {
    let value = super::observe_field_value(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow assessment evidence field is unavailable"))?;
    let AspectValue::UInt64(number) = value else {
        return Err(denial(
            "workflow assessment evidence field has the wrong type",
        ));
    };
    facts.push(WorthQueryApplicationObservedFact::Field {
        entity_id: entity,
        kind,
        locator: locator.clone(),
        value: AspectValue::UInt64(number),
    });
    Ok(number)
}

pub(in crate::domain_computation::primary_graph) fn recover_settled_live_membership(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    transition: EntityId,
    expected_identity: &str,
    expected_occurrence: u64,
) -> Result<(RelationId, Vec<WorthQueryApplicationObservedFact>), WorthQueryApplicationAttemptDenial>
{
    let kind = layout.transition.entity_kind;
    let mut facts = vec![WorthQueryApplicationObservedFact::Entity {
        entity_id: transition,
        kind,
    }];
    exact(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.identity,
        text(expected_identity.to_owned()),
        &mut facts,
    )?;
    for (locator, expected) in [
        (
            &layout.transition.protocol_version,
            crate::domain_computation::primary_graph::workflow::schema::version::WORKFLOW_FACT_PROTOCOL_VERSION,
        ),
        (&layout.transition.occurrence, expected_occurrence),
        (
            &layout.transition.outcome,
            encode_transition_outcome(ApplicationWorkflowControlOutcome::Completed),
        ),
    ] {
        exact(
            runtime,
            snapshot,
            transition,
            kind,
            locator,
            AspectValue::UInt64(expected),
            &mut facts,
        )?;
    }
    let partition = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_partition,
        &mut facts,
    )?;
    let slot = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_slot,
        &mut facts,
    )?;
    let generation = exact_u64(
        runtime,
        snapshot,
        transition,
        kind,
        &layout.transition.live_membership_generation,
        &mut facts,
    )?;
    let partition = u32::try_from(partition)
        .map_err(|_| denial("settled live-membership partition is invalid"))?;
    let generation = u32::try_from(generation)
        .map_err(|_| denial("settled live-membership generation is invalid"))?;
    Ok((
        RelationId::new(PartitionId::new(partition), slot, generation),
        facts,
    ))
}
