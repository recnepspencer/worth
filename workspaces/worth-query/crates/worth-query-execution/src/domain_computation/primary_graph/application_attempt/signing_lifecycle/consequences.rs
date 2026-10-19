//! Collect immutable supersession and current-membership removal as one admitted native batch.
use super::{
    attempt, capacity, coordinates, observation, stopped, unavailable, Effect, EntityId, Fact,
    Lease, Locus, Outcome, QueryRequestExecution, Rule,
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
impl Batch {
    pub(super) fn admit<Schema, Operation, Input, Scope>(
        &self,
        admission: &crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation<Schema, Operation, Input, Scope>,
    ) -> Result<
        worth_query_declaration::facade::application_operation::ApplicationCandidateRequirements,
        Outcome,
    > {
        let mut demand = super::super::effect_program::PlatformEffectDemand::default();
        for effect in &self.effects {
            demand.observe(effect).map_err(attempt)?;
        }
        let requirements = demand.requirements();
        super::super::effect_program::admit_signing_lifecycle_effects(admission, demand)
            .map_err(attempt)?
            .materialize(&self.effects)
            .map_err(attempt)?;
        Ok(requirements)
    }
}

pub(super) fn collect(
    rule: &Rule,
    subject: EntityId,
    changed: &[Locus],
    actor: EntityId,
    operation: &str,
    logical_request: [u8; 32],
    lease: &Lease,
    execution: &QueryRequestExecution<'_>,
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
    lease.handle().with_runtime(request, |runtime| {
        let edges = observation::adjacency(
            runtime,
            request,
            lease.snapshot(),
            subject,
            current.kind,
            rule.spec().maximum_evidence,
            &mut batch.facts,
        )?;
        for edge in edges {
            context.checkpoint(1).map_err(stopped)?;
            let evidence = edge.to;
            let (footprint, _memory) = observation::footprint(
                rule,
                evidence,
                lease,
                runtime,
                execution,
                request,
                &mut batch.facts,
            )?;
            if footprint.authentication().subject != coordinates(subject)
                || footprint
                    .dependencies()
                    .iter()
                    .any(|dependency| dependency.source != coordinates(subject))
            {
                return Err(unavailable());
            }
            let affected = affected_loci(
                footprint
                    .dependencies()
                    .iter()
                    .map(|dependency| &dependency.source_field),
                changed,
            );
            if affected.is_empty() {
                continue;
            }
            let previous = observation::adjacency(
                runtime,
                request,
                lease.snapshot(),
                evidence,
                consequence.kind,
                1,
                &mut batch.facts,
            )?;
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
    })
}
fn affected_loci<'a>(
    dependencies: impl Iterator<Item = &'a Locus>,
    changed: &[Locus],
) -> Vec<Locus> {
    let mut affected = Vec::new();
    for field in dependencies {
        if changed.contains(field) && !affected.contains(field) {
            affected.push(field.clone());
        }
    }
    affected
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

#[cfg(test)]
mod tests {
    use super::{affected_loci, Locus};
    #[test]
    fn each_rows_own_dependency_footprint_not_the_installed_union_decides_supersession() {
        let description = Locus::new("Item", "Facts", "Description");
        let assignment = Locus::new("Item", "Facts", "Assignment");
        let first = [description.clone(), assignment.clone()];
        let second = [assignment];
        let changed = [description];
        assert_eq!(affected_loci(first.iter(), &changed), changed);
        assert!(affected_loci(second.iter(), &changed).is_empty());
    }
}
