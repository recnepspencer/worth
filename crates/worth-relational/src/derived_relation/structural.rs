//! Genuine proposed/committed source admission and frozen proof consumption.
use super::affinity::DerivedSourceAffinity;
use super::evaluation::SourceRead;
use super::proof::{
    DerivedRelationReadIdentity as Identity, PreparedBody, PreparedDerivedRelationRead as Prepared,
};
use super::{DerivedRelationAdjacency as Adjacency, DerivedRelationData as Data};
use super::{DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle};
use super::{DerivedRelationReadDenial as Denial, DerivedRelationReads as Reads};
use crate::identity::data::{EntityId, KindId};
use crate::validation::data::{
    StructuralReadError, StructuralRelationRecord, StructuralRelationView,
};
use std::sync::Arc;

impl<'runtime> StructuralRelationView<'runtime> {
    pub(crate) fn with_derived_source(
        mut self,
        runtime: &crate::validation::engine::InvariantRuntimeView<'_>,
        observation: &crate::validation::engine::InvariantObservation<'_>,
        committed: bool,
    ) -> Self {
        let version = self.aggregate_read_parts().0.version_id();
        self.derived_source = Some((
            runtime.selected_registry.clone(),
            DerivedSourceAffinity {
                observation: observation.observation_identity(),
                committed,
                version,
                proposal: observation.proposal_identity().cloned(),
            },
        ));
        self
    }

    /// Creates only a sealed lookup key, without reading graph data. Evaluation
    /// can therefore reject an unprepared root before any source acquisition.
    pub fn derived_read_identity(
        &self,
        definition: &Handle,
        root: EntityId,
    ) -> Result<Identity, Denial> {
        self.read_budget()
            .check_live()
            .map_err(Denial::Structural)?;
        let (registry, source) = self
            .derived_source
            .as_ref()
            .ok_or(DefinitionDenial::ForeignObservation)?;
        if !registry.registry().admits_derived_relation(definition) {
            return Err(DefinitionDenial::ForeignDefinition.into());
        }
        // An observation can be shared by rules with different read rosters.
        // A genuine prepared proof must never widen its consumer's audience.
        for (relation, from, to) in definition.physical_sources() {
            self.require_relation_kind(*relation)
                .map_err(|error| self.derived_refusal(error))?;
            self.require_entity_kind(*from)
                .map_err(|error| self.derived_refusal(error))?;
            self.require_entity_kind(*to)
                .map_err(|error| self.derived_refusal(error))?;
        }
        Ok(Identity {
            source: source.clone(),
            definition: definition.clone(),
            root,
        })
    }

    /// Evaluates the genuine current view and seals all positive and negative
    /// reads for later use by that same observation/posture after planning.
    pub fn prepare_derived_relations_from(
        &self,
        definition: &Handle,
        root: EntityId,
    ) -> Result<Prepared, Denial> {
        let identity = self.derived_read_identity(definition, root)?;
        let (registry, _) = self
            .derived_source
            .as_ref()
            .expect("identity admitted source");
        let mut read = StructuralSource {
            view: self,
            reads: Reads::default(),
        };
        let data = super::evaluation::evaluate(registry.registry(), definition, root, &mut read)?;
        self.read_budget()
            .claim_retained((size_of::<PreparedBody>() + 2 * size_of::<usize>()) as u64)
            .map_err(Denial::Structural)?;
        self.read_budget()
            .check_live()
            .map_err(Denial::Structural)?;
        Ok(Prepared {
            identity,
            body: Arc::new(PreparedBody {
                data,
                _retention: self.work_meter(),
            }),
        })
    }

    /// Returns borrowed complete data only after checking the actual selected
    /// registry and exact observation, posture, proposal, root and definition.
    pub fn read_prepared_derived_relations<'borrow>(
        &'borrow self,
        prepared: &'borrow Prepared,
    ) -> Result<&'borrow Data, Denial> {
        let expected =
            self.derived_read_identity(&prepared.identity.definition, prepared.identity.root)?;
        if expected != prepared.identity || expected.source != prepared.identity.source {
            return Err(DefinitionDenial::ForeignObservation.into());
        }
        Ok(&prepared.body.data)
    }

    fn derived_refusal(&self, error: StructuralReadError) -> Denial {
        Denial::Structural(self.read_budget().refusal(error))
    }
}

struct StructuralSource<'borrow, 'runtime> {
    view: &'borrow StructuralRelationView<'runtime>,
    reads: Reads,
}

impl SourceRead for StructuralSource<'_, '_> {
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
