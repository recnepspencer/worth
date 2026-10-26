//! Record the actual complete Native value, including empty frontier boundaries.
use super::*;
use crate::domain_computation::primary_graph::invariant_projection::{
    StructuralWalkObservation, WorthQueryInvariantProjectionTraversalDenialKind as Kind,
    WorthQueryInvariantProjectionWalk,
};
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;

impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Complete structural decision read, including its root.
    pub fn decision_walk<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        root: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        self.decision_closure(relation, direction, std::slice::from_ref(root))
    }
    /// One complete multi-root dependency, with original issued-value custody.
    pub fn decision_closure<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryInvariantEntityIdentity<Schema, Entity>],
    ) -> Result<
        WorthQueryInvariantProjectionWalk<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let execution = *self.reader.execution;
        let result = execution.run_native_with_checkpoint(|context| {
            let admission = self.admission_identity.ok_or_else(|| {
                WorthQueryInvariantProjectionTraversalDenial::new(
                    Kind::UndeclaredDecisionTarget,
                    relation.name(),
                )
            })?;
            let contracts = self.admitted_contracts.as_ref().cloned().ok_or_else(|| {
                WorthQueryInvariantProjectionTraversalDenial::new(
                    Kind::UndeclaredDecisionTarget,
                    relation.name(),
                )
            })?;
            let bytes = relation
                .name()
                .len()
                .checked_add(relation.from().len())
                .and_then(|n| n.checked_add(relation.to().len()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or_else(|| {
                    self.reader
                        .walk_stop(Cause::ChargedBytesOverflow, relation.name())
                })?;
            let _target_memory = super::super::super::walk::memory::reserve(
                &execution,
                context.has_accounting(),
                bytes,
            )
            .map_err(|cause| self.reader.walk_stop(cause, relation.name()))?;
            self.admit_decision_target(&relation_target(&relation))
                .map_err(|denial| decision_traversal_denial(&denial))?;
            let walk = self.reader.closure(relation, direction, roots)?;
            if !walk
                .observation
                .issued_at(self.reader.snapshot, self.reader.authority_identity)
            {
                return Err(WorthQueryInvariantProjectionTraversalDenial::new(
                    Kind::ForeignIdentity,
                    relation.name(),
                ));
            }
            self.decision_facts
                .record_walk(
                    std::sync::Arc::clone(&walk.observation),
                    admission,
                    contracts,
                    &execution,
                    context,
                )
                .map_err(|cause| match cause {
                    super::super::decision_reads::structural_walks::WalkRecordDenial::Execution(
                        cause,
                    ) => self.reader.walk_stop(cause, relation.name()),
                    super::super::decision_reads::structural_walks::WalkRecordDenial::Conflict => {
                        WorthQueryInvariantProjectionTraversalDenial::new(
                            Kind::ForeignIdentity,
                            relation.name(),
                        )
                    }
                })?;
            Ok(walk)
        });
        let result = match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.reader.walk_stop(stop.into(), relation.name())),
        };
        if result.is_err() {
            self.decision_facts.fail_walk();
        }
        result
    }
    /// Forward reachability backed by the entire complete decision walk.
    pub fn decision_reaches<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        from: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
        to: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<bool, WorthQueryInvariantProjectionTraversalDenial> {
        if !self.reader.identity_is_local(to, relation.to()) || to.kind != from.kind {
            self.decision_facts.fail_walk();
            return Err(WorthQueryInvariantProjectionTraversalDenial::new(
                Kind::ForeignIdentity,
                relation.name(),
            ));
        }
        let execution = *self.reader.execution;
        let result = execution.run_native_with_checkpoint(|context| {
            let walk = self.decision_walk(relation, RelationDirection::Forward, from)?;
            for visit in walk.observation.native().visits() {
                StructuralWalkObservation::checkpoint(context)
                    .map_err(|cause| self.reader.walk_stop(cause, relation.name()))?;
                if visit.entity.entity_id == to.entity_id {
                    return Ok(true);
                }
            }
            Ok(false)
        });
        let result = match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.reader.walk_stop(stop.into(), relation.name())),
        };
        if result.is_err() {
            self.decision_facts.fail_walk();
        }
        result
    }
}
