//! Record the two actual Native observations that jointly answer an orbit.
#[cfg(test)]
mod tests;
use super::super::decision_reads::structural_walks::WalkRecordDenial;
use super::*;
use crate::domain_computation::primary_graph::aggregate_identity::WorthQueryAggregateIdentity;
use crate::domain_computation::primary_graph::invariant_projection::{
    WorthQueryInvariantProjectionOrbit, WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    /// Read persistent naming and successor topology as publication dependencies.
    pub fn decision_orbit<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        let execution = *self.reader.execution;
        let result = execution.run_native_with_checkpoint(|context| {
            let denied = || {
                WorthQueryInvariantProjectionTraversalDenial::new(
                    Kind::UndeclaredDecisionTarget,
                    relation.name(),
                )
            };
            let admission = self.admission_identity.ok_or_else(denied)?;
            let contracts = self.admitted_contracts.clone().ok_or_else(denied)?;
            self.admit_decision_target(&relation_target(&relation))
                .map_err(|denial| decision_traversal_denial(&denial))?;
            let orbit = self.reader.orbit(relation, member)?;
            if !orbit
                .walk()
                .issued_at(self.reader.snapshot, self.reader.authority_identity)
                || orbit.aggregate().source() != self.reader.snapshot
            {
                return Err(WorthQueryInvariantProjectionTraversalDenial::new(
                    Kind::ForeignIdentity,
                    relation.name(),
                ));
            }
            let registered = self
                .decision_facts
                .record_aggregate(
                    orbit.aggregate(),
                    admission,
                    contracts.clone(),
                    self.reader.authority_identity,
                    context,
                )
                .and_then(|()| {
                    self.decision_facts.record_walk(
                        orbit.walk(),
                        admission,
                        contracts,
                        &execution,
                        context,
                    )
                });
            registered.map_err(|denial| match denial {
                WalkRecordDenial::Execution(cause) => self.reader.walk_stop(cause, relation.name()),
                WalkRecordDenial::Conflict => WorthQueryInvariantProjectionTraversalDenial::new(
                    Kind::ForeignIdentity,
                    relation.name(),
                ),
            })?;
            Ok(orbit)
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
    /// Select identity through the same complete paired decision read.
    pub fn decision_orbit_of<Relation: OperationReads<Operation>, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<
        WorthQueryAggregateIdentity<Schema, Relation>,
        WorthQueryInvariantProjectionTraversalDenial,
    > {
        Ok(self.decision_orbit(relation, member)?.identity().clone())
    }
}
