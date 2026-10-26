//! Fresh privileged historical inspection, independent of current live membership.
use super::{
    WorthQueryApplicationInvariantProjectionReader,
    WorthQueryInvariantProjectionTraversalDenial as Denial,
    WorthQueryInvariantProjectionTraversalDenialKind as Kind,
};
use crate::domain_computation::primary_graph::aggregate_identity::{
    WorthQueryAggregateIdentity, WorthQueryAggregateLineage,
};
use worth_query_installation::facade::{ApplicationRelationRef, ApplicationSchema};
use worth_relational::facade::{
    runtime::{
        PinnedAggregateLineageScope, PinnedAggregateReadDenial, PinnedAggregateReadDenialKind,
        RelationalPinnedReadAdmissionDenial,
    },
    transactions::RelationalExecutionDenialCause as Cause,
};

impl<Schema: ApplicationSchema> WorthQueryApplicationInvariantProjectionReader<'_, Schema> {
    /// Inspect incident history and resolve merge continuations at this reader's
    /// exact snapshot. Requires this privileged reader's full projection authority;
    /// the name supplies selection only. No publication dependencies are recorded.
    /// Native scans the schema roster and occupied slots once, then visits incident
    /// lineage per merge hop; this is not a constant-time name lookup.
    pub fn lineage<Relation, Entity>(
        &mut self,
        relation: ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        identity: &WorthQueryAggregateIdentity<Schema, Relation>,
    ) -> Result<WorthQueryAggregateLineage<Schema, Relation>, Denial> {
        let execution = *self.execution;
        let result = execution.run_native_with_checkpoint(|_| {
            let scope = self.lineage_selection(&relation, identity)?;
            let pin = self
                .runtime
                .read_truth()
                .pin_read_capability(self.snapshot, &[], self.execution)
                .map_err(|denial| match denial {
                    RelationalPinnedReadAdmissionDenial::Request(stop) => self.walk_stop(
                        worth_execution::WorkCeilingDenial::Stopped(stop).into(),
                        relation.name(),
                    ),
                    _ => Denial::new(Kind::EndpointUnavailable, relation.name()),
                })?;
            let read = pin.read_aggregate_lineage(self.execution, scope);
            if let Err(PinnedAggregateReadDenial {
                kind: PinnedAggregateReadDenialKind::Read(cause),
                ..
            }) = &read
            {
                self.native_read_stop.get_or_insert(*cause);
            }
            let work = match &read {
                Ok(value) => value.work_units(),
                Err(denial) => denial.work,
            };
            if !usize::try_from(work).is_ok_and(|work| self.work_budget.consume(work)) {
                self.work_budget.mark_exceeded();
                return Err(Denial::new(Kind::WorkBudgetExceeded, relation.name()));
            }
            let native = read.map_err(|denial| self.lineage_denial(denial, relation.name()))?;
            Ok(WorthQueryAggregateLineage::new(
                native,
                identity.clone(),
                self.version(),
            ))
        });
        match result {
            Ok((answer, _)) => answer,
            Err(stop) => Err(self.walk_stop(Cause::from(stop), relation.name())),
        }
    }

    fn lineage_selection<Relation, Entity>(
        &self,
        relation: &ApplicationRelationRef<Schema, Relation, Entity, Entity>,
        identity: &WorthQueryAggregateIdentity<Schema, Relation>,
    ) -> Result<PinnedAggregateLineageScope<'static>, Denial> {
        let layout = self
            .layout
            .relation(relation.name())
            .ok_or_else(|| Denial::new(Kind::RelationNotInstalled, relation.name()))?;
        if !layout.orbit {
            return Err(Denial::new(Kind::NotOrbitRelation, relation.name()));
        }
        if layout.from != layout.to {
            return Err(Denial::new(Kind::InvalidOrbit, relation.name()));
        }
        if self.layout.entity_kind(relation.from()) != Some(layout.from)
            || self.layout.entity_kind(relation.to()) != Some(layout.to)
        {
            return Err(Denial::new(Kind::ForeignIdentity, relation.name()));
        }
        identity
            .lineage_selection(self.binding_identity, layout.kind)
            .ok_or_else(|| Denial::new(Kind::ForeignIdentity, relation.name()))
    }

    fn lineage_denial(&mut self, denial: PinnedAggregateReadDenial, subject: &str) -> Denial {
        use PinnedAggregateReadDenialKind as Native;
        let kind = match denial.kind {
            Native::Read(cause) => return self.walk_stop(cause, subject),
            Native::NotAggregateRelation => Kind::NotOrbitRelation,
            Native::DefinitionUnavailable => Kind::AggregateDefinitionUnavailable,
            Native::IdentityUnavailable => Kind::AggregateIdentityUnavailable,
            Native::WorkLimitExceeded => {
                self.work_budget.mark_exceeded();
                Kind::WorkBudgetExceeded
            }
            _ => Kind::InvalidAggregateLineage,
        };
        Denial::new(kind, subject)
    }
}
