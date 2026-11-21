//! Complete original Native truth, independent of its consuming Query producer.
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, MapKernelContext, WorkCeilingDenial};
use worth_relational::facade::{
    runtime::PinnedStructuralWalk, snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause as Cause,
};
pub mod currentness;
pub mod dependencies;
pub mod support;
pub use dependencies::StructuralWalkDependencies;
pub mod commitment;
pub mod key;
pub mod memory;
pub mod read;
pub use key::StructuralWalkKey;
pub use read::StructuralWalkQuery;

#[derive(Debug)]
pub struct StructuralWalkObservation {
    query: StructuralWalkKey,
    native: PinnedStructuralWalk,
    commitment: [u8; 32],
    snapshot: super::projection_read_origin::ProjectionReadOrigin,
    _memory: Option<ExecutionMemoryReservation>,
}
impl StructuralWalkObservation {
    /// Fallible storage comparison; graph currentness still performs a fresh Native read.
    pub fn same_truth_with_poll<E>(
        &self,
        other: &Self,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        if self.commitment != other.commitment {
            return Ok(false);
        }
        let (a, b) = (self.key(), other.key());
        if a.relation() != b.relation()
            || a.entity_kind() != b.entity_kind()
            || a.relation_kind() != b.relation_kind()
            || a.direction() != b.direction()
            || a.roots().len() != b.roots().len()
            || self.native.visits().len() != other.native.visits().len()
            || self.native.relations().len() != other.native.relations().len()
        {
            return Ok(false);
        }
        for (a, b) in a.roots().iter().zip(b.roots()) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        for (a, b) in self.native.visits().iter().zip(other.native.visits()) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        for (a, b) in self.native.relations().iter().zip(other.native.relations()) {
            poll()?;
            if a != b {
                return Ok(false);
            }
        }
        poll()?;
        Ok(true)
    }
    pub fn key(&self) -> &StructuralWalkKey {
        &self.query
    }
    pub fn native(&self) -> &PinnedStructuralWalk {
        &self.native
    }
    pub fn commitment(&self) -> [u8; 32] {
        self.commitment
    }
    pub fn source(
        &self,
    ) -> &super::projection_read_origin::ProjectionReadOrigin {
        &self.snapshot
    }
    pub fn checkpoint(
        context: &mut MapKernelContext<'_, '_>,
    ) -> Result<(), Cause> {
        (if context.has_accounting() {
            context.checkpoint(1)
        } else {
            context.poll()
        })
        .map_err(|stop| Cause::from(WorkCeilingDenial::Stopped(stop)))
    }
}
impl PartialEq for StructuralWalkObservation {
    fn eq(&self, other: &Self) -> bool {
        self.commitment == other.commitment
            && self.query == other.query
            && self.native.visits() == other.native.visits()
            && self.native.relations() == other.native.relations()
    }
}
impl Eq for StructuralWalkObservation {}

mod court_doors_for_test;
