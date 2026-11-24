//! Read metered physical records from one admitted structural observation.
use crate::derived_relation::definition::{
    DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle,
};
use crate::derived_relation::evaluation::{self, SourceRead};
use crate::derived_relation::read::{
    DerivedRelationAdjacency as Adjacency, DerivedRelationReads as Reads,
};
use crate::derived_relation::read_denial::DerivedRelationReadDenial as Denial;
use crate::identity::data::{EntityId, KindId};
use crate::validation::custom_rule::structural_views::StructuralRelationView;
use crate::validation::data::structural_read::{StructuralReadError, StructuralRelationRecord};

pub(crate) struct StructuralSource<'borrow, 'runtime> {
    pub(crate) view: &'borrow StructuralRelationView<'runtime>,
    pub(crate) reads: Reads,
}

impl SourceRead for StructuralSource<'_, '_> {
    fn admit_definition(
        &mut self,
        registry: &crate::schema::data::RelationalSchemaRegistry,
        handle: &Handle,
    ) -> Result<(), Denial> {
        if self
            .view
            .structural_source
            .as_ref()
            .is_some_and(|(actual, _)| std::ptr::eq(actual.registry(), registry))
        {
            self.view.require_derived_relation(handle)
        } else {
            evaluation::admit_definition(registry, handle, self)
        }
    }
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial> {
        self.view
            .read_budget()
            .checkpoint(units as u64)
            .map_err(Denial::Structural)
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Denial> {
        self.view
            .read_budget()
            .claim_retained(bytes as u64)
            .map_err(Denial::Structural)
    }
    fn check_scratch_peak(&mut self, bytes: usize) -> Result<(), Denial> {
        self.view
            .read_budget()
            .check_scratch_peak(bytes as u64)
            .map_err(Denial::Structural)
    }
    fn entity_kind(&mut self, id: EntityId) -> Result<KindId, Denial> {
        if id.generation.is_zero() {
            return Err(DefinitionDenial::RecordUnavailable.into());
        }
        let kind = self
            .view
            .entity_kind(id)
            .map_err(|error| self.view.derived_refusal(error))?;
        self.reads.entity_records += 1;
        Ok(kind)
    }
    fn adjacency(&mut self, probe: Adjacency) -> Result<Vec<StructuralRelationRecord>, Denial> {
        self.view
            .require_relation_kind(probe.relation)
            .map_err(|error| self.view.derived_refusal(error))?;
        let ids = if probe.outgoing {
            self.view.outgoing_relations_for_entity(probe.anchor)
        } else {
            self.view.incoming_relations_for_entity(probe.anchor)
        }
        .map_err(|_| {
            self.view
                .derived_refusal(StructuralReadError::WorkBudgetExceeded)
        })?;
        self.reads.adjacency_lists += 1;
        let mut result = Vec::new();
        for id in ids {
            self.checkpoint(1)?;
            let record = self
                .view
                .relation(id)
                .map_err(|error| self.view.derived_refusal(error))?;
            self.reads.relation_records += 1;
            if record.kind_id == probe.relation {
                self.retain(size_of::<StructuralRelationRecord>())?;
                result.push(record);
            }
        }
        Ok(result)
    }
    fn reads(&self) -> Reads {
        self.reads
    }
}
