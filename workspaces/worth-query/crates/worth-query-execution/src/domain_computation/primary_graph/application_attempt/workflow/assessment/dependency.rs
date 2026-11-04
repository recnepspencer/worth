use std::collections::BTreeMap;

use worth_foundational::facade::{AspectValue, InternedString};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

use super::*;
use {crate::domain_computation::primary_graph::application_attempt::workflow::evidence_dependency::encode_direction, crate::domain_computation::primary_graph::application_attempt::workflow::evidence_dependency::WorkflowEvidenceDependencyKind};

pub fn visit_dependency_facts<Error>(
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
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::AspectPresence { entity_id, present, .. } =>
                (*entity_id, if *present { WorkflowEvidenceDependencyKind::AspectPresent } else { WorkflowEvidenceDependencyKind::AspectAbsent }),
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceEntity {
                entity_id,
            } => (*entity_id, WorkflowEvidenceDependencyKind::Entity),
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::Entity {
                entity_id,
                ..
            } => (*entity_id, WorkflowEvidenceDependencyKind::EntityKind),
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceAspectRevision {
                entity_id,
                ..
            } => (*entity_id, WorkflowEvidenceDependencyKind::AspectRevision),
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceFieldRevision {
                entity_id,
                ..
            } => (*entity_id, WorkflowEvidenceDependencyKind::FieldRevision),
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
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
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::AspectPresence { kind, aspect, .. } => {
                fields.insert(layout.evidence_dependency.native_entity_kind.clone(), AspectValue::UInt64(u64::from(kind.as_u32())));
                fields.insert(layout.evidence_dependency.aspect.clone(), text(aspect.as_str()));
            }
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::Entity {
                kind,
                ..
            } => {
                fields.insert(
                    layout.evidence_dependency.native_entity_kind.clone(),
                    AspectValue::UInt64(u64::from(kind.as_u32())),
                );
            }
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceAspectRevision {
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
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceFieldRevision {
                locator,
                native_revision,
                ..
            } => {
                append_field_revision_fields(&layout.evidence_dependency, locator, *native_revision, &mut fields);
            }
            crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact::SourceAdjacencyRevision {
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
            fields: Default::default(),
            kind: layout.evidence_dependency_relation,
            key: format!("evidence-dependency-{index}"),
            from: EntityReference::Created(evidence.clone()),
            to: EntityReference::Created(dependency),
        })?;
    }
    Ok(())
}

pub fn append_field_revision_fields(
    layout: &crate::domain_computation::primary_graph::application_attempt::workflow::schema::WorkflowEvidenceDependencyLayout,
    locator: &worth_foundational::facade::AspectFieldLocator,
    native_revision: Option<worth_relational::facade::runtime::RelationalFieldRevision>,
    fields: &mut BTreeMap<worth_foundational::facade::AspectFieldLocator, AspectValue>,
) {
    fields.insert(
        layout.aspect.clone(),
        text(locator.aspect().aspect_key().as_str()),
    );
    fields.insert(
        layout.field.clone(),
        text(locator.field_path().fields()[0].as_str()),
    );
    if let Some(revision) = native_revision {
        fields.insert(
            layout.native_revision.clone(),
            AspectValue::UInt64(revision.version().0),
        );
        fields.insert(
            layout.field_presence.clone(),
            text(match revision.presence() {
                worth_relational::facade::runtime::RelationalFieldPresence::Present => "present",
                worth_relational::facade::runtime::RelationalFieldPresence::Absent => "absent",
            }),
        );
    }
}

fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
