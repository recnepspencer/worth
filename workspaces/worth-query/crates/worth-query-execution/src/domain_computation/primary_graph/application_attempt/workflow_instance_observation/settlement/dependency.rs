use worth_foundational::facade::{AspectKey, AspectValue, InternedString};
use worth_relational::facade::identity::{EntityId, KindId, PartitionId, VersionId};

use super::{denial, observed_text, observed_u64};
use crate::domain_computation::primary_graph::application_attempt::{
    observation::{observe_field, WorthQueryApplicationFieldObservation},
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationObservedFact,
};
use crate::domain_computation::primary_graph::workflow::evidence_dependency::{
    decode_direction, evidence_dependency_adjacency_work, maximum_evidence_dependencies,
    WorkflowEvidenceDependencyKind,
};
use crate::domain_computation::primary_graph::workflow::schema::WorthQueryWorkflowLayout;

pub(in crate::domain_computation::primary_graph::application_attempt) fn observe_evidence_dependencies(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    evidence: EntityId,
    maximum_dependency_facts: usize,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Vec<WorthQueryApplicationObservedFact>, WorthQueryApplicationAttemptDenial> {
    let starting_fact_count = facts.len();
    let maximum_dependencies = maximum_evidence_dependencies(maximum_dependency_facts);
    let dependencies = super::adjacency_with_kind(
        runtime,
        snapshot,
        layout.evidence_dependency_relation,
        evidence,
        evidence_dependency_adjacency_work(maximum_dependencies),
        facts,
    )?;
    if dependencies.len() > maximum_dependencies {
        return Err(budget_denial());
    }
    let observed = dependencies
        .into_iter()
        .map(|dependency| observe_dependency(runtime, snapshot, layout, dependency, facts))
        .collect::<Result<Vec<_>, _>>()?;
    if facts.len().saturating_sub(starting_fact_count) > maximum_dependency_facts {
        return Err(budget_denial());
    }
    Ok(observed)
}

fn observe_dependency(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    layout: &WorthQueryWorkflowLayout,
    dependency: EntityId,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<WorthQueryApplicationObservedFact, WorthQueryApplicationAttemptDenial> {
    let kind = layout.evidence_dependency.entity_kind;
    facts.push(WorthQueryApplicationObservedFact::Entity {
        entity_id: dependency,
        kind,
    });
    let fact_kind = observed_text(
        runtime,
        snapshot,
        dependency,
        kind,
        &layout.evidence_dependency.fact_kind,
        facts,
    )?;
    let entity = EntityId::new(
        PartitionId::new(narrow_u32(observed_u64(
            runtime,
            snapshot,
            dependency,
            kind,
            &layout.evidence_dependency.entity_partition,
            facts,
        )?)?),
        observed_u64(
            runtime,
            snapshot,
            dependency,
            kind,
            &layout.evidence_dependency.entity_slot,
            facts,
        )?,
        narrow_u32(observed_u64(
            runtime,
            snapshot,
            dependency,
            kind,
            &layout.evidence_dependency.entity_generation,
            facts,
        )?)?,
    );
    let aspect = optional_text(
        runtime,
        snapshot,
        dependency,
        kind,
        &layout.evidence_dependency.aspect,
        facts,
    )?;
    let relation_kind = optional_u64(
        runtime,
        snapshot,
        dependency,
        kind,
        &layout.evidence_dependency.relation_kind,
        facts,
    )?;
    let direction = optional_text(
        runtime,
        snapshot,
        dependency,
        kind,
        &layout.evidence_dependency.direction,
        facts,
    )?;
    let native_revision = optional_u64(
        runtime,
        snapshot,
        dependency,
        kind,
        &layout.evidence_dependency.native_revision,
        facts,
    )?;
    let comparison_work_limit = optional_u64(
        runtime,
        snapshot,
        dependency,
        kind,
        &layout.evidence_dependency.comparison_work_limit,
        facts,
    )?;
    match WorkflowEvidenceDependencyKind::decode(&fact_kind) {
        Some(WorkflowEvidenceDependencyKind::Entity)
            if aspect.is_none()
                && relation_kind.is_none()
                && direction.is_none()
                && native_revision.is_none()
                && comparison_work_limit.is_none() =>
        {
            Ok(WorthQueryApplicationObservedFact::SourceEntity { entity_id: entity })
        }
        Some(WorkflowEvidenceDependencyKind::AspectRevision)
            if relation_kind.is_none()
                && direction.is_none()
                && comparison_work_limit.is_none() =>
        {
            let aspect = aspect
                .and_then(AspectKey::new)
                .ok_or_else(|| denial("workflow evidence dependency aspect is invalid"))?;
            Ok(WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id: entity,
                aspect,
                native_revision,
            })
        }
        Some(WorkflowEvidenceDependencyKind::AdjacencyRevision) if aspect.is_none() => {
            let relation_kind = relation_kind
                .and_then(|value| u32::try_from(value).ok())
                .map(KindId::new)
                .ok_or_else(|| denial("workflow evidence dependency relation kind is invalid"))?;
            let direction = direction
                .as_deref()
                .and_then(decode_direction)
                .ok_or_else(|| denial("workflow evidence dependency direction is invalid"))?;
            let comparison_work_limit = comparison_work_limit
                .and_then(|value| usize::try_from(value).ok())
                .ok_or_else(|| denial("workflow evidence dependency work limit is invalid"))?;
            Ok(WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                relation_kind,
                anchor: entity,
                direction,
                native_revision: native_revision.map(VersionId),
                comparison_work_limit,
                endpoints: Vec::new(),
            })
        }
        _ => Err(denial("workflow evidence dependency shape is invalid")),
    }
}

fn optional_text(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<String>, WorthQueryApplicationAttemptDenial> {
    match optional_value(runtime, snapshot, entity, kind, locator, facts)? {
        Some(AspectValue::String(InternedString::Raw(value))) => Ok(Some(value)),
        Some(_) => Err(denial(
            "workflow evidence dependency field has the wrong type",
        )),
        None => Ok(None),
    }
}

fn optional_u64(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<u64>, WorthQueryApplicationAttemptDenial> {
    match optional_value(runtime, snapshot, entity, kind, locator, facts)? {
        Some(AspectValue::UInt64(value)) => Ok(Some(value)),
        Some(_) => Err(denial(
            "workflow evidence dependency field has the wrong type",
        )),
        None => Ok(None),
    }
}

fn optional_value(
    runtime: &worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &worth_relational::facade::snapshots::SnapshotHandle,
    entity: EntityId,
    kind: KindId,
    locator: &worth_foundational::facade::AspectFieldLocator,
    facts: &mut Vec<WorthQueryApplicationObservedFact>,
) -> Result<Option<AspectValue>, WorthQueryApplicationAttemptDenial> {
    match observe_field(runtime, snapshot, entity, kind, locator)
        .ok_or_else(|| denial("workflow evidence dependency field is unavailable"))?
    {
        WorthQueryApplicationFieldObservation::Present(value) => {
            facts.push(WorthQueryApplicationObservedFact::Field {
                entity_id: entity,
                kind,
                locator: locator.clone(),
                value: value.clone(),
            });
            Ok(Some(value))
        }
        WorthQueryApplicationFieldObservation::Absent => {
            facts.push(WorthQueryApplicationObservedFact::AbsentField {
                entity_id: entity,
                kind,
                locator: locator.clone(),
            });
            Ok(None)
        }
    }
}

fn narrow_u32(value: u64) -> Result<u32, WorthQueryApplicationAttemptDenial> {
    u32::try_from(value)
        .map_err(|_| denial("workflow evidence dependency entity coordinate is invalid"))
}

fn budget_denial() -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(
        crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationAttemptDenialKind::DecisionFactBudgetExceeded,
        "workflow assessment evidence dependency fact budget",
    )
}
