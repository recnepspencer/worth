//! Collect immutable supersession and current-membership removal as one admitted native batch.
use super::{
    capacity, coordinates, observation, stopped, unavailable, Effect, EntityId, Fact, Lease, Locus,
    Outcome, Rule,
};
use std::collections::BTreeMap;
use worth_query_declaration::facade::application_schema::{
    ApplicationScalarValueBinding, ApplicationSigningSupersession,
    ApplicationSigningSupersessionBinding, ApplicationSigningSupersessionParts,
};
use worth_relational::facade::transactions::{CreatedEntityRef, EntityReference};

#[derive(Default)]
pub(super) struct Batch {
    pub(super) effects: Vec<Effect>,
    pub(super) facts: Vec<Fact>,
}

pub(super) fn collect(
    rule: &Rule,
    subject: EntityId,
    changed: &BTreeMap<EntityId, Vec<Locus>>,
    effects: &[Effect],
    actor: EntityId,
    operation: &str,
    logical_request: [u8; 32],
    lease: &Lease,
    execution: &super::QueryRequestExecution<'_>,
    request: &worth_execution::ExecutionRequest<'_, '_>,
    context: &mut worth_execution::MapKernelContext<'_, '_>,
    batch: &mut Batch,
) -> Result<(), Outcome> {
    let current = lease
        .layout
        .relation(&rule.spec().current_relation)
        .ok_or_else(unavailable)?;
    let consequence = lease
        .layout
        .relation(&rule.spec().consequence_relation)
        .ok_or_else(unavailable)?;
    let edges = lease.handle().with_runtime(request, |runtime| {
        observation::adjacency(
            runtime,
            request,
            lease.snapshot(),
            subject,
            current.kind,
            rule.spec().maximum_evidence,
            &mut batch.facts,
        )
    })?;
    for edge in edges {
        context.checkpoint(1).map_err(stopped)?;
        let evidence = edge.to;
        let footprint = lease.handle().with_runtime(request, |runtime| {
            observation::footprint(rule, evidence, lease, runtime, request, &mut batch.facts)
        })?;
        if !footprint.has_valid_subject(coordinates(subject)) {
            return Err(unavailable());
        }
        let mut affected = Vec::new();
        for (source, field) in footprint.dependencies() {
            if changed
                .iter()
                .any(|(entity, fields)| coordinates(*entity) == source && fields.contains(field))
                && !affected.contains(field)
            {
                affected.push(field.clone());
            }
        }
        for field in super::graph::changed_selections(
            rule, subject, &footprint, lease, effects, execution, request, context, batch,
        )? {
            if !affected.contains(&field) {
                affected.push(field);
            }
        }
        if affected.is_empty() {
            continue;
        }
        let previous = lease.handle().with_runtime(request, |runtime| {
            observation::adjacency(
                runtime,
                request,
                lease.snapshot(),
                evidence,
                consequence.kind,
                1,
                &mut batch.facts,
            )
        })?;
        if !previous.is_empty() {
            return Err(unavailable());
        }
        let record =
            ApplicationSigningSupersession::from_parts(ApplicationSigningSupersessionParts {
                evidence: coordinates(evidence),
                subject: coordinates(subject),
                actor: coordinates(actor),
                operation: operation.into(),
                logical_request,
                changed_dependencies: affected,
            })
            .map_err(|_| unavailable())?;
        // Lowering the unlink needs the identity of the edge actually read
        // above, in addition to the membership revision used for staleness.
        batch.facts.push(Fact::Relation {
            relation_kind: current.kind,
            from: subject,
            to: evidence,
            matching_relations: vec![edge.relation_id],
        });
        append(
            rule,
            subject,
            evidence,
            edge.relation_id,
            record,
            lease,
            batch,
        )?;
    }
    Ok(())
}
fn append(
    rule: &Rule,
    subject: EntityId,
    evidence: EntityId,
    current: worth_relational::facade::identity::RelationId,
    record: ApplicationSigningSupersession,
    lease: &Lease,
    batch: &mut Batch,
) -> Result<(), Outcome> {
    let field = &rule.spec().consequence_field;
    let locator = lease
        .layout
        .field_locator(field.entity(), field.aspect(), field.field())
        .ok_or_else(unavailable)?
        .clone();
    let kind = lease
        .layout
        .entity_kind(field.entity())
        .ok_or_else(unavailable)?;
    let relation = lease
        .layout
        .relation(&rule.spec().consequence_relation)
        .ok_or_else(unavailable)?;
    let key = format!(
        "signing-supersession-{}-{}-{}",
        evidence.partition_value_u64(),
        evidence.local_slot_value(),
        evidence.generation_value()
    );
    let reference = EntityReference::Created(CreatedEntityRef {
        partition_id: subject.partition_id,
        kind_id: kind,
        client_key: worth_relational::facade::symbols::ClientKey::raw(key.clone()),
    });
    batch.effects.push(Effect::CreateEntity {
        kind,
        key: key.clone(),
        fields: BTreeMap::from([(
            locator,
            ApplicationSigningSupersessionBinding::encode(&record).map_err(|_| capacity())?,
        )]),
        partition: super::super::effect_program::WorthQueryApplicationCreationPartition::Context(
            subject.partition_id,
        ),
    });
    batch.effects.push(Effect::CreateRelation {
        kind: relation.kind,
        key: format!("{key}-evidence"),
        from: EntityReference::Existing(evidence),
        to: reference,
    });
    batch.effects.push(Effect::DeleteRelation {
        relation_id: current,
    });
    Ok(())
}
