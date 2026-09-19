use std::marker::PhantomData;

use super::{
    denial, map_structural_error, WorthQueryApplicationInvariantEntity,
    WorthQueryApplicationInvariantReadView, WorthQueryApplicationInvariantRelation,
};
use crate::domain_computation::primary_graph::application_invariant::{
    WorthQueryApplicationInvariantRelationBinding, WorthQueryInvariantAccessDenial,
    WorthQueryInvariantAccessDenialKind,
};

impl<Schema> WorthQueryApplicationInvariantReadView<'_, Schema> {
    pub fn relations_from<Relation, From, To>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, From, To>,
        from: &WorthQueryApplicationInvariantEntity<Schema, From>,
    ) -> Result<
        Vec<WorthQueryApplicationInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantAccessDenial,
    > {
        self.relations(binding, from, true)
    }

    pub fn relations_to<Relation, From, To>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, From, To>,
        to: &WorthQueryApplicationInvariantEntity<Schema, To>,
    ) -> Result<
        Vec<WorthQueryApplicationInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantAccessDenial,
    > {
        self.relations(binding, to, false)
    }

    fn relations<Relation, From, To, Endpoint>(
        &self,
        binding: &WorthQueryApplicationInvariantRelationBinding<Schema, Relation, From, To>,
        endpoint: &WorthQueryApplicationInvariantEntity<Schema, Endpoint>,
        outgoing: bool,
    ) -> Result<
        Vec<WorthQueryApplicationInvariantRelation<Schema, Relation, From, To>>,
        WorthQueryInvariantAccessDenial,
    > {
        self.check_binding(&binding.binding_identity)?;
        self.relations
            .require_relation_kind(binding.relation_kind)
            .map_err(|error| map_structural_error(error, "relation kind"))?;
        self.check_entity(
            endpoint,
            if outgoing {
                binding.from_kind
            } else {
                binding.to_kind
            },
        )?;
        let ids = if outgoing {
            self.relations
                .outgoing_relations_for_entity(endpoint.entity_id)
        } else {
            self.relations
                .incoming_relations_for_entity(endpoint.entity_id)
        }
        .map_err(|error| {
            denial(
                WorthQueryInvariantAccessDenialKind::WorkBudgetExceeded,
                format!("{error:?}"),
            )
        })?;
        let mut typed = Vec::new();
        for id in ids {
            let record = self
                .relations
                .relation(id)
                .map_err(|error| map_structural_error(error, "relation record"))?;
            if record.kind_id != binding.relation_kind {
                continue;
            }
            let from_kind = self
                .relations
                .entity_kind(record.source)
                .map_err(|error| map_structural_error(error, "relation source"))?;
            let to_kind = self
                .relations
                .entity_kind(record.target)
                .map_err(|error| map_structural_error(error, "relation target"))?;
            if from_kind != binding.from_kind || to_kind != binding.to_kind {
                return Err(denial(
                    WorthQueryInvariantAccessDenialKind::WrongEntityKind,
                    "relation endpoint kind",
                ));
            }
            self.admission.relation(record.relation_id)?;
            self.admission.entity(record.source)?;
            self.admission.entity(record.target)?;
            typed.push(WorthQueryApplicationInvariantRelation {
                relation_id: record.relation_id,
                from: self.entity(record.source, from_kind),
                to: self.entity(record.target, to_kind),
                _marker: PhantomData,
            });
        }
        Ok(typed)
    }
}
