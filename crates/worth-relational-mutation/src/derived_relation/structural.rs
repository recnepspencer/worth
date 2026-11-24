//! Genuine proposed/committed source admission and frozen proof consumption.
use super::evaluation::SourceRead;
use super::proof::{
    DerivedRelationReadIdentity as Identity, PreparedBody, PreparedDerivedRelationRead as Prepared,
};
use crate::contracts::derived_relation::definition::{
    DerivedRelationDenial as DefinitionDenial, DerivedRelationHandle as Handle,
};
use crate::contracts::derived_relation::read::{
    DerivedRelationData as Data, DerivedRelationReads as Reads,
};
use crate::contracts::derived_relation::read_denial::DerivedRelationReadDenial as Denial;
use crate::contracts::derived_relation::read_shape::DerivedRelationReadShape as Shape;
use crate::contracts::graph_member::domain::NativeMemberDomainOwnerPort as _;
use crate::contracts::graph_member::key::NativeMemberKey;
use crate::validation::custom_rule::structural_views::StructuralRelationView;
use std::sync::Arc;

pub mod admission;
pub mod changes;
pub mod source;
pub(crate) use changes::{prepare as prepare_changes, read as read_changes};
pub use changes::{DerivedRelationChangeData, PreparedDerivedRelationChanges};
pub(crate) use source::StructuralSource;

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
    ) -> Result<&'borrow [crate::derived_relation::proof::DerivedRelationWalkStep], Denial> {
        if matches!(shape, Shape::Adjacency { .. }) {
            return Err(DefinitionDenial::InvalidExpression.into());
        }
        self.read_prepared_derived_source(prepared, shape)?;
        Ok(&prepared.body.steps)
    }
}
