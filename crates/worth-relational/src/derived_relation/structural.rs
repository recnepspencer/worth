//! Genuine proposed/committed source admission and frozen proof consumption.
use super::evaluation::SourceRead;
use super::proof::{
    DerivedRelationReadIdentity as Identity, PreparedBody, PreparedDerivedRelationRead as Prepared,
};
use super::DerivedRelationReadShape as Shape;
use super::{DerivedRelationAdjacency as Adjacency, DerivedRelationData as Data};
use super::{DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle};
use super::{DerivedRelationReadDenial as Denial, DerivedRelationReads as Reads};
use crate::graph_member::NativeMemberKey;
use crate::identity::data::{EntityId, KindId};
use crate::validation::data::{
    StructuralReadError, StructuralRelationRecord, StructuralRelationView,
};
use std::sync::Arc;

mod changes;
pub(crate) use changes::{prepare as prepare_changes, read as read_changes};
pub use changes::{DerivedRelationChangeData, PreparedDerivedRelationChanges};

impl<'runtime> StructuralRelationView<'runtime> {
    /// Creates only a sealed lookup key, without reading graph data. Evaluation
    /// can therefore reject an unprepared root before any source acquisition.
    pub fn derived_read_identity(
        &self,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
    ) -> Result<Identity, Denial> {
        self.derived_source_identity(definition, root, Shape::Adjacency { reverse: false })
    }

    /// Issues a lookup identity for one exact interpretation without acquiring data.
    pub fn derived_source_identity(
        &self,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
        shape: Shape,
    ) -> Result<Identity, Denial> {
        self.require_derived_relation(definition)?;
        if !matches!(shape, Shape::Adjacency { .. }) {
            let names = definition
                .parts()
                .from
                .name_bytes()
                .and_then(|n| n.checked_add(definition.parts().to.name_bytes()?))
                .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
            self.read_budget()
                .checkpoint(names as u64)
                .map_err(Denial::Structural)?;
            if definition.parts().from != definition.parts().to {
                return Err(DefinitionDenial::DiscontinuousEndpoints.into());
            }
        }
        let (_, source) = self.structural_source.as_ref().expect("source admitted");
        Ok(Identity {
            source: source.clone(),
            definition: definition.clone(),
            root: root.into(),
            shape,
        })
    }

    /// Admit only the actual registered source and this view's full audience.
    /// This does not create a root, observed member or prepared read proof.
    pub(crate) fn require_derived_relation(&self, definition: &Handle) -> Result<(), Denial> {
        self.read_budget()
            .check_live()
            .map_err(Denial::Structural)?;
        let (registry, _) = self
            .structural_source
            .as_ref()
            .ok_or(DefinitionDenial::ForeignObservation)?;
        let names = definition
            .parts()
            .identity
            .schema
            .0
            .len()
            .checked_add(definition.parts().identity.name.len())
            .ok_or(DefinitionDenial::WorkBudgetExceeded)?;
        let levels = u64::from(
            usize::BITS
                - registry
                    .registry()
                    .derived_relations
                    .len()
                    .max(1)
                    .leading_zeros(),
        );
        self.read_budget()
            .checkpoint(
                (names as u64)
                    .checked_mul(11)
                    .and_then(|n| n.checked_mul(levels))
                    .ok_or(DefinitionDenial::WorkBudgetExceeded)?,
            )
            .map_err(Denial::Structural)?;
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
        // A carrier-only recipe may have no physical relation leaves. Its
        // existence and empty-result reads still require the actual carrier.
        for &carrier in &definition.entry.carriers {
            self.require_entity_kind(carrier)
                .map_err(|error| self.derived_refusal(error))?;
        }
        Ok(())
    }

    /// Evaluates the genuine current view and seals all positive and negative
    /// reads for later use by that same observation/posture after planning.
    pub fn prepare_derived_relations_from(
        &self,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
    ) -> Result<Prepared, Denial> {
        self.prepare_derived_source(definition, root, Shape::Adjacency { reverse: false })
    }

    /// Preparation is the only source-data acquisition door. The resulting
    /// proof cannot be consumed as a different interpretation after freeze.
    pub fn prepare_derived_source(
        &self,
        definition: &Handle,
        root: impl Into<NativeMemberKey>,
        shape: Shape,
    ) -> Result<Prepared, Denial> {
        let root = root.into();
        let identity = self.derived_source_identity(definition, root.clone(), shape)?;
        let (registry, _) = self
            .structural_source
            .as_ref()
            .expect("identity admitted source");
        let mut read = StructuralSource {
            view: self,
            reads: Reads::default(),
        };
        let (data, steps) =
            super::source::acquire(registry.registry(), definition, root, shape, &mut read)?;
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
                steps,
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
        if prepared.identity.shape != (Shape::Adjacency { reverse: false }) {
            return Err(DefinitionDenial::InvalidExpression.into());
        }
        self.read_prepared_derived_source(prepared, Shape::Adjacency { reverse: false })
    }

    /// Checks actual source currency for exactly the shape carried by this proof.
    pub fn read_prepared_derived_source<'borrow>(
        &'borrow self,
        prepared: &'borrow Prepared,
        shape: Shape,
    ) -> Result<&'borrow Data, Denial> {
        if prepared.identity.shape != shape {
            return Err(DefinitionDenial::InvalidExpression.into());
        }
        let expected = self.derived_source_identity(
            &prepared.identity.definition,
            prepared.identity.root.clone(),
            prepared.identity.shape,
        )?;
        if expected != prepared.identity || expected.source != prepared.identity.source {
            return Err(DefinitionDenial::ForeignObservation.into());
        }
        Ok(&prepared.body.data)
    }

    /// Returns actual breadth-first order only from a current traversal proof.
    pub fn read_prepared_derived_steps<'borrow>(
        &'borrow self,
        prepared: &'borrow Prepared,
        shape: Shape,
    ) -> Result<&'borrow [super::DerivedRelationWalkStep], Denial> {
        if matches!(shape, Shape::Adjacency { .. }) {
            return Err(DefinitionDenial::InvalidExpression.into());
        }
        self.read_prepared_derived_source(prepared, shape)?;
        Ok(&prepared.body.steps)
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
