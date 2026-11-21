//! Complete actual Native derived truth, shared independently of its Query rows.
use std::sync::Arc;
use worth_execution::{ExecutionMemoryReservation, MapKernelContext};
use worth_relational::facade::{
    runtime::{DerivedRelationData, PinnedDerivedRelationRead},
    schema::DerivedRelationHandle,
    snapshots::SnapshotHandle,
    transactions::RelationalExecutionDenialCause as Cause,
};
pub mod commitment;
pub mod currentness;
pub mod dependencies;
pub mod key;
pub mod read;
pub use dependencies::DerivedSourceDependencies;
pub use key::DerivedSourceKey;
pub use read::DerivedSourceQuery;

pub struct DerivedSourceObservation {
    key: DerivedSourceKey,
    definition: DerivedRelationHandle,
    native: PinnedDerivedRelationRead,
    commitment: [u8; 32],
    snapshot: super::projection_read_origin::ProjectionReadOrigin,
    _memory: Option<ExecutionMemoryReservation>,
}
impl DerivedSourceObservation {
    pub fn key(&self) -> &DerivedSourceKey {
        &self.key
    }
    pub fn native(
        &self,
    ) -> &PinnedDerivedRelationRead {
        &self.native
    }
    pub fn definition(
        &self,
    ) -> &DerivedRelationHandle {
        &self.definition
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
        .map_err(Into::into)
    }
    pub fn same_truth_with_poll<E>(
        &self,
        other: &Self,
        mut poll: impl FnMut() -> Result<(), E>,
    ) -> Result<bool, E> {
        poll()?;
        if self.snapshot.runtime_instance_id() != other.snapshot.runtime_instance_id()
            || self.commitment != other.commitment
            || !self.key.same_with_poll(&other.key, &mut poll)?
            || self.definition != other.definition
        {
            return Ok(false);
        }
        same_data(self.native.data(), other.native.data(), poll)
    }
}
fn same_data<E>(
    left: &DerivedRelationData,
    right: &DerivedRelationData,
    mut poll: impl FnMut() -> Result<(), E>,
) -> Result<bool, E> {
    macro_rules! compare {
        ($access:ident) => {
            if left.$access().len() != right.$access().len() {
                poll()?;
                return Ok(false);
            }
            for (a, b) in left.$access().iter().zip(right.$access()) {
                poll()?;
                if a != b {
                    return Ok(false);
                }
            }
        };
    }
    compare!(endpoints);
    compare!(entity_dependencies);
    compare!(physical_relations);
    compare!(adjacency_dependencies);
    poll()?;
    Ok(true)
}
impl PartialEq for DerivedSourceObservation {
    fn eq(&self, other: &Self) -> bool {
        self.same_truth_with_poll(other, || Ok::<_, std::convert::Infallible>(()))
            .unwrap()
    }
}
impl Eq for DerivedSourceObservation {}
impl std::fmt::Debug for DerivedSourceObservation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DerivedSourceObservation")
            .field("key", &self.key)
            .field("data", self.native.data())
            .finish()
    }
}
