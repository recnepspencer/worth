//! Exact-snapshot aggregate identity paired with complete business successor truth.
use super::{
    WorthQueryApplicationInvariantProjectionReader, WorthQueryInvariantEntityIdentity,
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use crate::domain_computation::primary_graph::aggregate_identity::WorthQueryAggregateIdentity;
use crate::domain_computation::primary_graph::aggregate_observation::{
    AggregateObservation, AggregateQuery,
};
use crate::domain_computation::primary_graph::structural_walk::StructuralWalkObservation;
use worth_execution::MapKernelContext;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema};
use worth_relational::facade::{
    runtime::{
        PinnedAggregateReadDenial, PinnedAggregateReadDenialKind,
        RelationalPinnedReadAdmissionDenial,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};
mod result;
pub use result::WorthQueryInvariantProjectionOrbit;

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Inspect the complete orbit in successor order from its current anchor.
    /// This read admits every business member through the projection scope;
    /// it does not record a publication decision dependency.
    pub fn orbit<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryInvariantProjectionOrbit<Schema, Relation, Entity>, Denial> {
        let execution = *self.execution;
        let result = execution.run_native_with_checkpoint(|context| {
            let layout = self
                .layout
                .relation(relation.name())
                .cloned()
                .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
            if !self.identity_is_local(member, relation.from()) || member.kind != layout.from {
                return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
            }
            if !layout.orbit {
                return Err(Denial::new(Kind::NotOrbitRelation, relation.name()));
            }
            if layout.from != layout.to {
                return Err(Denial::new(Kind::InvalidOrbit, relation.name()));
            }
            let aggregate = self.read_orbit_identity(
                layout.from,
                layout.kind,
                member.entity_id,
                relation.name(),
            )?;
            let anchor = WorthQueryInvariantEntityIdentity {
                entity_id: aggregate.native().anchor(),
                kind: layout.from,
                entity: member.entity.clone(),
                authority_identity: self.authority_identity,
                _marker: std::marker::PhantomData,
            };
            // Both Native pins are admitted at this reader's same immutable snapshot.
            // The ordinary walk owns complete physical-edge and realized-scope admission.
            let walk = self.walk(relation, RelationDirection::Forward, &anchor)?;
            if walk.len() != aggregate.native().len()
                || walk.relations().len() != aggregate.native().len()
            {
                return Err(Denial::new(Kind::InvalidOrbit, relation.name()));
            }
            for entity in walk.entities() {
                self.orbit_checkpoint(context, relation.name())?;
                if !aggregate.native().contains(entity.entity_id) {
                    return Err(Denial::new(Kind::InvalidOrbit, relation.name()));
                }
            }
            self.orbit_checkpoint(context, relation.name())?;
            Ok(WorthQueryInvariantProjectionOrbit::new(
                aggregate,
                walk,
                self.binding_identity.clone(),
                layout.kind,
            ))
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.walk_stop(stop.into(), relation.name())),
        }
    }

    /// Select a persistent name through the same complete orbit inspection.
    /// A name grants no read, write or history-disclosure authority.
    pub fn orbit_of<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        member: &WorthQueryInvariantEntityIdentity<Schema, Entity>,
    ) -> Result<WorthQueryAggregateIdentity<Schema, Relation>, Denial> {
        Ok(self.orbit(relation, member)?.identity().clone())
    }

    fn read_orbit_identity(
        &mut self,
        entity_kind: worth_relational::facade::identity::KindId,
        relation: worth_relational::facade::identity::KindId,
        member: worth_relational::facade::identity::EntityId,
        subject: &str,
    ) -> Result<std::sync::Arc<AggregateObservation>, Denial> {
        let pin = self
            .runtime
            .read_truth()
            .pin_read_capability(self.snapshot, &[], self.execution)
            .map_err(|denial| match denial {
                RelationalPinnedReadAdmissionDenial::Request(stop) => self.walk_stop(
                    worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                    subject,
                ),
                _ => Denial::new(Kind::EndpointUnavailable, subject),
            })?;
        let result = AggregateObservation::read(
            &pin,
            self.execution,
            AggregateQuery {
                relation: subject,
                entity_kind,
                relation_kind: relation,
                member,
                maximum_work: self.work_budget.remaining().map(|n| n as u64),
                control: None,
            },
        );
        let work = match &result {
            Ok(value) => value.native().work_units(),
            Err(denial) => denial.work,
        };
        if let Err(PinnedAggregateReadDenial {
            kind: PinnedAggregateReadDenialKind::Read(cause),
            ..
        }) = &result
        {
            self.native_read_stop.get_or_insert(*cause);
        }
        // Debit actual work to the existing projection allowance. Private identity
        // acquisitions are not business endpoint/edge diagnostic observations.
        if !usize::try_from(work).is_ok_and(|work| self.work_budget.consume(work)) {
            self.work_budget.mark_exceeded();
            return Err(Denial::new(Kind::WorkBudgetExceeded, subject));
        }
        result.map_err(|denial| self.orbit_denial(denial, subject))
    }

    fn orbit_denial(&mut self, denial: PinnedAggregateReadDenial, subject: &str) -> Denial {
        use PinnedAggregateReadDenialKind as Native;
        let kind = match denial.kind {
            Native::Read(cause) => return self.walk_stop(cause, subject),
            Native::NotAggregateRelation => Kind::NotOrbitRelation,
            Native::MemberUnavailable | Native::WrongMemberKind => Kind::EndpointUnavailable,
            Native::WorkLimitExceeded => {
                self.work_budget.mark_exceeded();
                Kind::WorkBudgetExceeded
            }
            _ => Kind::InvalidOrbit,
        };
        Denial::new(kind, subject)
    }

    fn orbit_checkpoint(
        &mut self,
        context: &mut MapKernelContext<'_, '_>,
        subject: &str,
    ) -> Result<(), Denial> {
        StructuralWalkObservation::checkpoint(context)
            .map_err(|cause: Cause| self.walk_stop(cause, subject))
    }
}
