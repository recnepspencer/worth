//! Orbit identity and successor order from the same Native invariant view.
use super::super::WorthQueryApplicationInvariantRelationBinding;
use super::*;
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::runtime::AggregateReadDenial;
mod result;
pub use result::WorthQueryApplicationInvariantOrbit;

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    /// Complete orbit in successor order from its current least member.
    /// Native owns identity, permutation proof and the forward traversal;
    /// Query admits every business member/edge and compares the two member sets
    /// in O(n log n). Private identity metadata is not an application read grant.
    pub fn orbit<Relation, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, Entity, Entity>,
        member: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<
        WorthQueryApplicationInvariantOrbit<Schema, Relation, Entity>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&binding.binding_identity)?;
        self.relations
            .require_relation_kind(binding.relation_kind)
            .map_err(|cause| map_structural_error(cause, "orbit relation kind"))?;
        self.check_entity_affinity(member, binding.from_kind)?;
        if !binding.orbit {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::NotOrbitRelation,
                "declared orbit relation",
            ));
        }
        if binding.from_kind != binding.to_kind {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::WrongEntityKind,
                "orbit endorelation",
            ));
        }
        let aggregate = self
            .relations
            .aggregates()
            .of_source_relation(binding.relation_kind, member.entity_id)
            .map_err(aggregate_denial)?;
        let anchor = self.entity(aggregate.anchor(), binding.from_kind);
        let walk = self.walk(binding, RelationDirection::Forward, &anchor)?;
        if walk.len() != aggregate.len() || walk.relations().len() != aggregate.len() {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::InvalidOrbit,
                "orbit member/edge agreement",
            ));
        }
        for entity in walk.entities() {
            self.relations.read_budget().check_live().map_err(|cause| {
                WorthQueryInvariantAccessDenial::from_native_read(cause, "orbit member agreement")
            })?;
            if !aggregate.contains(entity.entity_id) {
                return Err(denial(
                    WorthQueryInvariantAccessDenialKind::InvalidOrbit,
                    "orbit member agreement",
                ));
            }
        }
        Ok(WorthQueryApplicationInvariantOrbit::new(
            aggregate,
            walk,
            self.binding_identity.clone(),
            binding.relation_kind,
        ))
    }

    /// Identity selection retains the complete orbit read footprint.
    pub fn orbit_of<Relation, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, Entity, Entity>,
        member: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<
        super::super::WorthQueryAggregateIdentity<Schema, Relation>,
        WorthQueryInvariantAccessDenial,
    > {
        Ok(self.orbit(binding, member)?.identity().clone())
    }
}

fn aggregate_denial(cause: AggregateReadDenial) -> WorthQueryInvariantAccessDenial {
    use WorthQueryInvariantAccessDenialKind as Kind;
    let kind = match cause {
        AggregateReadDenial::Budget(cause) => {
            return WorthQueryInvariantAccessDenial::from_native_read(cause, "orbit identity")
        }
        AggregateReadDenial::SourceUnavailable => Kind::AggregateSourceUnavailable,
        AggregateReadDenial::NotAggregateRelation => Kind::NotOrbitRelation,
        AggregateReadDenial::WrongMemberKind => Kind::WrongEntityKind,
        AggregateReadDenial::PreparationRefused => Kind::AggregatePreparationRefused,
        AggregateReadDenial::Execution(cause) => Kind::Execution(cause),
    };
    denial(kind, "orbit identity")
}
