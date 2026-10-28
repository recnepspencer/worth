//! Typed complete reachability on one installed candidate/before-image view.
use worth_query_declaration::facade::application_schema::RelationDirection;
use worth_relational::facade::runtime::{CompleteStructuralWalk, StructuralWalkDenialKind};

use super::super::WorthQueryApplicationInvariantRelationBinding;
use super::{
    denial, WorthQueryApplicationInvariantEntity, WorthQueryApplicationInvariantReadView,
    WorthQueryInvariantAccessDenial, WorthQueryInvariantAccessDenialKind,
};

mod result;
pub use result::{
    WorthQueryApplicationInvariantWalk, WorthQueryApplicationInvariantWalkIter,
    WorthQueryApplicationInvariantWalkStep,
};

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    /// Read the complete reachable set, including `from` at distance zero.
    /// Native traversal owns O((v+a) log(v+a)) work; Query admits O(v+e) facts
    /// against its prepared sets. The result retains its original native payer.
    pub fn walk<Relation, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        from: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<
        WorthQueryApplicationInvariantWalk<Schema, Relation, Entity>,
        WorthQueryInvariantAccessDenial,
    > {
        self.closure(binding, direction, std::slice::from_ref(from))
    }

    /// Complete multi-root reachability, ordered by minimum distance then
    /// native identity. Duplicate roots appear once; no depth cap is imposed.
    pub fn closure<Relation, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, Entity, Entity>,
        direction: RelationDirection,
        roots: &[WorthQueryApplicationInvariantEntity<Schema, Entity>],
    ) -> Result<
        WorthQueryApplicationInvariantWalk<Schema, Relation, Entity>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&binding.binding_identity)?;
        self.relations
            .require_relation_kind(binding.relation_kind)
            .map_err(|error| super::map_structural_error(error, "walk relation kind"))?;
        if binding.from_kind != binding.to_kind {
            return Err(denial(
                WorthQueryInvariantAccessDenialKind::WrongEntityKind,
                "walk endorelation",
            ));
        }
        for root in roots {
            self.relations
                .read_budget()
                .checkpoint(1)
                .map_err(walk_denial)?;
            self.check_entity_affinity(root, binding.from_kind)?;
        }
        self.relations
            .read_budget()
            .claim_retained((roots.len() as u64).saturating_mul(std::mem::size_of::<
                worth_relational::facade::identity::EntityId,
            >() as u64))
            .map_err(walk_denial)?;
        let mut seeds = Vec::with_capacity(roots.len());
        for root in roots {
            self.relations
                .read_budget()
                .check_live()
                .map_err(walk_denial)?;
            seeds.push(root.entity_id);
        }
        let native = match direction {
            RelationDirection::Forward => self
                .relations
                .walk_outgoing_from(&seeds, binding.relation_kind),
            RelationDirection::Reverse => self
                .relations
                .walk_incoming_from(&seeds, binding.relation_kind),
        };
        let native = match native {
            Ok(native) => {
                self.reads.walk(native.reads());
                native
            }
            Err(denial) => {
                self.reads.walk(denial.reads);
                return Err(walk_denial(denial.kind));
            }
        };
        self.admit_walk(&native, binding.from_kind)?;
        self.relations
            .read_budget()
            .check_live()
            .map_err(walk_denial)?;
        Ok(WorthQueryApplicationInvariantWalk::new(
            self,
            native,
            binding.from_kind,
        ))
    }

    /// Forward reachability; a valid local entity reaches itself by zero hops.
    /// Interruption refuses instead of returning a partial membership answer.
    pub fn reaches<Relation, Entity>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, Entity, Entity>,
        from: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
        to: &WorthQueryApplicationInvariantEntity<Schema, Entity>,
    ) -> Result<bool, WorthQueryInvariantAccessDenial> {
        self.check_binding(&binding.binding_identity)?;
        self.check_entity_affinity(to, binding.to_kind)?;
        let walk = self.walk(binding, RelationDirection::Forward, from)?;
        let reaches = walk.contains_id(to.entity_id);
        self.relations
            .read_budget()
            .check_live()
            .map_err(walk_denial)?;
        Ok(reaches)
    }

    fn admit_walk(
        &self,
        native: &CompleteStructuralWalk,
        expected: worth_relational::facade::identity::KindId,
    ) -> Result<(), WorthQueryInvariantAccessDenial> {
        for step in native.steps() {
            if step.kind != expected {
                return Err(denial(
                    WorthQueryInvariantAccessDenialKind::WrongEntityKind,
                    "walk entity kind",
                ));
            }
            self.admission
                .entity(step.entity, self.relations.read_budget())?;
        }
        for record in native.relations() {
            self.admission
                .relation(record.relation_id, self.relations.read_budget())?;
        }
        Ok(())
    }
}

fn walk_denial(cause: StructuralWalkDenialKind) -> WorthQueryInvariantAccessDenial {
    WorthQueryInvariantAccessDenial::from_native_read(cause, "complete relation read")
}
