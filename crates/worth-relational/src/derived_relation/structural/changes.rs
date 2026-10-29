//! Candidate-local change proofs use the existing before/after pullback owner.
mod frontier;
mod proof;
pub use proof::{DerivedRelationChangeData, PreparedDerivedRelationChanges};

use super::{
    DefinitionDenial, Denial, Handle, Reads, SourceRead, StructuralRelationView, StructuralSource,
};
use crate::validation::data::TouchedStructuralSet;
use std::sync::Arc;

pub(crate) fn prepare(
    proposed: &StructuralRelationView<'_>,
    committed: &StructuralRelationView<'_>,
    touched: &TouchedStructuralSet,
    definition: &Handle,
) -> Result<PreparedDerivedRelationChanges, Denial> {
    let identity = proof::identity(proposed, committed, definition)?;
    proposed
        .require_derived_change_access(definition)
        .map_err(|cause| proposed.derived_refusal(cause))?;
    let budget = proposed.read_budget();
    let (records, adjacency) = frontier::acquire(proposed, committed, touched, definition)?;
    let mut before = ChangeSource(StructuralSource {
        view: committed,
        reads: Reads::default(),
    });
    let mut after = ChangeSource(StructuralSource {
        view: proposed,
        reads: Reads::default(),
    });
    let registry = &proposed
        .structural_source
        .as_ref()
        .expect("source admitted")
        .0;
    let affected = crate::derived_relation::affected::collect(
        registry.registry(),
        definition,
        &adjacency,
        &records,
        &mut before,
        &mut after,
    )?;
    budget
        .claim_retained((size_of::<proof::ChangeBody>() + 2 * size_of::<usize>()) as u64)
        .map_err(Denial::Structural)?;
    budget.check_live().map_err(Denial::Structural)?;
    Ok(PreparedDerivedRelationChanges {
        identity,
        body: Arc::new(proof::ChangeBody {
            data: DerivedRelationChangeData { records, affected },
            _retention: [committed.work_meter(), proposed.work_meter()],
        }),
    })
}

pub(crate) fn read<'a>(
    proposed: &StructuralRelationView<'_>,
    committed: &StructuralRelationView<'_>,
    prepared: &'a PreparedDerivedRelationChanges,
) -> Result<&'a DerivedRelationChangeData, Denial> {
    let current = proof::identity(proposed, committed, &prepared.identity.definition)?;
    if current != prepared.identity {
        return Err(DefinitionDenial::ForeignObservation.into());
    }
    Ok(&prepared.body.data)
}

/// Missing lifecycle endpoints are data only here; audience/control failures
/// retain their actual typed causes. Ordinary prepared reads are unchanged.
struct ChangeSource<'borrow, 'runtime>(StructuralSource<'borrow, 'runtime>);
impl super::SourceRead for ChangeSource<'_, '_> {
    fn checkpoint(&mut self, units: usize) -> Result<(), Denial> {
        self.0.checkpoint(units)
    }
    fn retain(&mut self, bytes: usize) -> Result<(), Denial> {
        self.0.retain(bytes)
    }
    fn entity_kind(&mut self, id: super::EntityId) -> Result<super::KindId, Denial> {
        match self.0.view.entity_kind(id) {
            Ok(kind) => {
                self.0.reads.entity_records += 1;
                Ok(kind)
            }
            Err(super::StructuralReadError::RecordUnavailable) => {
                Err(DefinitionDenial::RecordUnavailable.into())
            }
            Err(cause) => Err(self.0.view.derived_refusal(cause)),
        }
    }
    fn adjacency(
        &mut self,
        probe: super::Adjacency,
    ) -> Result<Vec<super::StructuralRelationRecord>, Denial> {
        self.0.adjacency(probe)
    }
    fn reads(&self) -> Reads {
        self.0.reads()
    }
}
