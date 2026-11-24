//! Canonical provenance of a payload selected from an actual retained root.
use crate::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use crate::branch::reference_checkpoint_data::CANONICAL_RUNTIME_INSTANCE_ID;
use crate::branch::target::RelationalBranchTarget;
use crate::branch::version::RelationalBranchVersion;
use crate::history::data::BranchId;
use crate::identity::data::{EntityId, KindId};
use serde::{Deserialize, Serialize};
use worth_foundational::FoundationalBranchReferenceGeneration;
use worth_relational_contracts::internal_port::branch::target::RelationalBranchTargetOwnerPort as _;

/// Describes the historical payload of a fresh selection occurrence.
/// This is provenance, never a source lease or permission to select again.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanonicalRetainedEntitySelection {
    source_branch: BranchId,
    source_generation: FoundationalBranchReferenceGeneration,
    source_truth_version: RelationalBranchVersion,
    source_head: RelationalBranchTarget,
    entity: EntityId,
    kind: KindId,
    #[serde(skip)]
    pub(crate) _decoded_memory: Option<std::sync::Arc<worth_execution::ExecutionMemoryReservation>>,
}

impl CanonicalRetainedEntitySelection {
    pub fn source_branch(&self) -> &BranchId {
        &self.source_branch
    }

    pub const fn source_generation(&self) -> FoundationalBranchReferenceGeneration {
        self.source_generation
    }

    pub const fn source_truth_version(&self) -> RelationalBranchVersion {
        self.source_truth_version
    }

    pub fn source_head(&self) -> &RelationalBranchTarget {
        &self.source_head
    }

    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    pub const fn kind(&self) -> KindId {
        self.kind
    }
}

impl PartialEq for CanonicalRetainedEntitySelection {
    fn eq(&self, other: &Self) -> bool {
        self.source_branch == other.source_branch
            && self.source_generation == other.source_generation
            && self.source_truth_version == other.source_truth_version
            && self.source_head == other.source_head
            && self.entity == other.entity
            && self.kind == other.kind
    }
}
impl Eq for CanonicalRetainedEntitySelection {}

/// Operations used by the Relational owners.
pub trait CanonicalRetainedEntitySelectionOwnerPort: Sized {
    fn from_selected_descriptor(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &RelationalBranchBasisDescriptor,
        entity: EntityId,
        kind: KindId,
    ) -> Result<
        Option<std::sync::Arc<Self>>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    >;
}

impl CanonicalRetainedEntitySelectionOwnerPort for CanonicalRetainedEntitySelection {
    fn from_selected_descriptor(
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &RelationalBranchBasisDescriptor,
        entity: EntityId,
        kind: KindId,
    ) -> Result<
        Option<std::sync::Arc<Self>>,
        crate::execution::denial_cause::RelationalExecutionDenialCause,
    > {
        let worth_foundational::FoundationalBranchTarget::Basis(target) =
            source.reference().target()
        else {
            return Ok(None);
        };
        use crate::execution::denial_cause::RelationalExecutionDenialCause as Cause;
        let bytes = std::mem::size_of::<(usize, usize, Self)>()
            .checked_add(std::mem::size_of::<(
                usize,
                usize,
                worth_execution::ExecutionMemoryReservation,
            )>())
            .and_then(|n| n.checked_add(source.branch_id().0.len()))
            .and_then(|n| {
                target
                    .parent_commit_ids()
                    .len()
                    .checked_mul(8)
                    .and_then(|parents| n.checked_add(parents))
            })
            .and_then(|n| u64::try_from(n).ok())
            .ok_or(Cause::ChargedBytesOverflow)?;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(Cause::from)?;
        Ok(Some(std::sync::Arc::new(Self {
            _decoded_memory: memory.map(std::sync::Arc::new),
            source_branch: source.branch_id().clone(),
            source_generation: source.reference().generation(),
            source_truth_version: source.truth_version(),
            source_head: target.rebind_runtime_instance_id(CANONICAL_RUNTIME_INSTANCE_ID),
            entity,
            kind,
        })))
    }
}
