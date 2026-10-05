use serde::{Deserialize, Serialize};

use crate::data::aspect::Aspect;
use crate::data::handle::NodeId;
use crate::data::output::PartitionSubscription;

/// One possible observation of a checked evaluator. The declaration bounds
/// discovery; it does not create a dependency or an invalidation cause.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DeclaredSignalInput {
    pub source: NodeId,
    pub aspect: Aspect,
    pub scope: Option<PartitionSubscription>,
}

impl DeclaredSignalInput {
    pub fn new(source: NodeId, aspect: Aspect) -> Self {
        Self {
            source,
            aspect,
            scope: None,
        }
    }

    pub fn scoped(source: NodeId, aspect: Aspect, scope: PartitionSubscription) -> Self {
        Self {
            source,
            aspect,
            scope: Some(scope),
        }
    }
}

/// The complete finite set of observations a checked evaluator may make.
/// Empty means no graph inputs; absence of this declaration means open discovery.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(from = "Vec<DeclaredSignalInput>", into = "Vec<DeclaredSignalInput>")]
pub struct BoundedSignalInputs {
    inputs: Vec<DeclaredSignalInput>,
}

impl BoundedSignalInputs {
    pub fn new(inputs: impl IntoIterator<Item = DeclaredSignalInput>) -> Self {
        let mut inputs = inputs.into_iter().collect::<Vec<_>>();
        inputs.sort_unstable();
        inputs.dedup();
        Self { inputs }
    }

    pub fn as_slice(&self) -> &[DeclaredSignalInput] {
        &self.inputs
    }

    pub(crate) fn copy_work_bound(&self) -> usize {
        self.inputs.iter().fold(self.inputs.len(), |units, input| {
            input.scope.as_ref().map_or(units, |scope| {
                units
                    .saturating_add(scope.path().total_segment_bytes())
                    .saturating_add(scope.path().depth())
            })
        })
    }

    /// Captures reserve their complete finite envelope once. Scope clones use
    /// segment lengths, rather than the producer declaration's spare capacity.
    pub(crate) fn captured_scope_heap_bound(&self) -> Option<u64> {
        self.inputs.iter().try_fold(0_u64, |bytes, input| {
            let Some(scope) = &input.scope else {
                return Some(bytes);
            };
            let strings = scope
                .path()
                .depth()
                .checked_mul(std::mem::size_of::<String>())?;
            bytes
                .checked_add(u64::try_from(strings).ok()?)?
                .checked_add(u64::try_from(scope.path().checked_segment_bytes()?).ok()?)
        })
    }

    pub fn contains(
        &self,
        source: NodeId,
        aspect: Aspect,
        scope: Option<&PartitionSubscription>,
    ) -> bool {
        // Exact observation declarations deliberately do not widen a scoped
        // read to an unscoped read or to another aspect of the same producer.
        self.inputs.iter().any(|input| {
            input.source == source && input.aspect == aspect && input.scope.as_ref() == scope
        })
    }
}

impl From<Vec<DeclaredSignalInput>> for BoundedSignalInputs {
    fn from(inputs: Vec<DeclaredSignalInput>) -> Self {
        Self::new(inputs)
    }
}

impl From<BoundedSignalInputs> for Vec<DeclaredSignalInput> {
    fn from(inputs: BoundedSignalInputs) -> Self {
        inputs.inputs
    }
}

impl crate::data::retained_storage::RetainedStorageMeasurement for BoundedSignalInputs {
    fn retained_heap_charge(
        &self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        self.inputs.retained_heap_charge(work)
    }
}

impl crate::data::retained_storage::RetainedStorageMeasurement for DeclaredSignalInput {
    fn retained_heap_charge(
        &self,
        work: &mut crate::data::retained_storage::RetainedStoragePreparation,
    ) -> Result<
        crate::data::retained_storage::RetainedStorageCharge,
        crate::data::retained_storage::RetainedStoragePreparationDenial,
    > {
        work.visit()?;
        self.scope.retained_heap_charge(work)
    }
}
