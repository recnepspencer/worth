use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::*;
use crate::domain_computation::primary_graph::workflow::evidence_dependency::{
    encode_direction, WorkflowEvidenceDependencyKind,
};

pub(super) fn visit_dependency_facts<Error>(
    layout: &WorthQueryWorkflowLayout,
    evidence: &CreatedEntityRef,
    meaning: &WorkflowAssessmentEvidenceMeaning,
    emit: &mut impl FnMut(WorthQueryApplicationRealizedEffect) -> Result<(), Error>,
) -> Result<(), Error> {
    for (index, fact) in meaning.currentness_facts.iter().enumerate() {
        let dependency = CreatedEntityRef {
            partition_id: evidence.partition_id,
            kind_id: layout.evidence_dependency.entity_kind,
            client_key: ClientKey::raw(format!("evidence-dependency-{index}")),
        };
        let (entity, fact_kind) = match fact {
            super::super::super::application_attempt::WorthQueryApplicationObservedFact::SourceEntity {
                entity_id,
            } => (*entity_id, WorkflowEvidenceDependencyKind::Entity),
            super::super::super::application_attempt::WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id,
                ..
            } => (*entity_id, WorkflowEvidenceDependencyKind::AspectRevision),
            super::super::super::application_attempt::WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                anchor,
                ..
            } => (*anchor, WorkflowEvidenceDependencyKind::AdjacencyRevision),
            _ => unreachable!("assessment settlement admits only native source currentness facts"),
        };
        let mut fields = BTreeMap::from([
            (
                layout.evidence_dependency.fact_kind.clone(),
                text(fact_kind.encode()),
            ),
            (
                layout.evidence_dependency.entity_partition.clone(),
                AspectValue::UInt64(entity.partition_value_u64()),
            ),
            (
                layout.evidence_dependency.entity_slot.clone(),
                AspectValue::UInt64(entity.local_slot_value()),
            ),
            (
                layout.evidence_dependency.entity_generation.clone(),
                AspectValue::UInt64(u64::from(entity.generation_value())),
            ),
        ]);
        match fact {
            super::super::super::application_attempt::WorthQueryApplicationObservedFact::SourceAspectRevision {
                aspect,
                native_revision,
                ..
            } => {
                fields.insert(layout.evidence_dependency.aspect.clone(), text(aspect.as_str()));
                if let Some(revision) = native_revision {
                    fields.insert(
                        layout.evidence_dependency.native_revision.clone(),
                        AspectValue::UInt64(*revision),
                    );
                }
            }
            super::super::super::application_attempt::WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
                relation_kind,
                direction,
                native_revision,
                comparison_work_limit,
                ..
            } => {
                fields.insert(
                    layout.evidence_dependency.relation_kind.clone(),
                    AspectValue::UInt64(u64::from(relation_kind.as_u32())),
                );
                fields.insert(
                    layout.evidence_dependency.direction.clone(),
                    text(encode_direction(*direction)),
                );
                if let Some(revision) = native_revision {
                    fields.insert(
                        layout.evidence_dependency.native_revision.clone(),
                        AspectValue::UInt64(revision.0),
                    );
                }
                fields.insert(
                    layout.evidence_dependency.comparison_work_limit.clone(),
                    AspectValue::UInt64(
                        u64::try_from(*comparison_work_limit).unwrap_or(u64::MAX),
                    ),
                );
            }
            _ => {}
        }
        emit(WorthQueryApplicationRealizedEffect::CreateEntity {
            kind: dependency.kind_id,
            key: format!("evidence-dependency-{index}"),
            fields,
            partition: WorthQueryApplicationCreationPartition::Context(evidence.partition_id),
        })?;
        emit(WorthQueryApplicationRealizedEffect::CreateRelation {
            kind: layout.evidence_dependency_relation,
            key: format!("evidence-dependency-{index}"),
            from: EntityReference::Created(evidence.clone()),
            to: EntityReference::Created(dependency),
        })?;
    }
    Ok(())
}

fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
