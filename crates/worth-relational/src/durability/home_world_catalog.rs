//! World-selected source descriptions grant nothing without the actual Home seal.
use crate::branch::RecordedRelationalComponentReference;
use crate::history::data::{CommitId, RelationalCommitReceipt};
use crate::publication::patch::data::PatchStreamPosition;
use std::sync::Arc;
use worth_execution::{
    ExecutionArray, ExecutionMemoryReservation, ExecutionRequest, WorkCeilingDenial,
};
use worth_foundational::{CanonicalDigestId, WorldOriginalFate};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalWorldEffectCoordinates {
    commit: CommitId,
    position: PatchStreamPosition,
    digest: CanonicalDigestId,
}
impl RelationalWorldEffectCoordinates {
    pub fn from_recorded(
        commit: CommitId,
        position: PatchStreamPosition,
        digest: CanonicalDigestId,
    ) -> Self {
        Self {
            commit,
            position,
            digest,
        }
    }
    pub fn commit(&self) -> CommitId {
        self.commit
    }
    pub fn position(&self) -> PatchStreamPosition {
        self.position
    }
    pub fn digest(&self) -> &CanonicalDigestId {
        &self.digest
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RelationalWorldSourceDescription {
    Recorded(RecordedRelationalComponentReference),
    Published(RelationalWorldEffectCoordinates),
}
#[derive(Debug)]
pub struct RelationalWorldOriginalDescription {
    caller: CanonicalDigestId,
    source: RelationalWorldSourceDescription,
    fate: WorldOriginalFate,
    effect: Option<RelationalWorldEffectCoordinates>,
    receipt: Option<RelationalCommitReceipt>,
    _receipt_memory: Option<ExecutionMemoryReservation>,
}
impl RelationalWorldOriginalDescription {
    pub fn from_recorded(
        request: &ExecutionRequest<'_, '_>,
        caller: CanonicalDigestId,
        source: RelationalWorldSourceDescription,
        fate: WorldOriginalFate,
        effect: Option<RelationalWorldEffectCoordinates>,
        receipt: Option<&RelationalCommitReceipt>,
    ) -> Result<Self, RelationalWorldCatalogDenial> {
        let bytes = match receipt {
            Some(receipt) => receipt
                .parents
                .len()
                .checked_mul(size_of::<CommitId>())
                .and_then(|n| n.checked_add(receipt.branch_id.0.len()))
                .and_then(|n| u64::try_from(n).ok())
                .ok_or(RelationalWorldCatalogDenial::CapacityOverflow)?,
            None => 0,
        };
        let memory = request
            .reserve_retained_memory(bytes)
            .map_err(RelationalWorldCatalogDenial::Request)?;
        Ok(Self {
            caller,
            source,
            fate,
            effect,
            receipt: receipt.cloned(),
            _receipt_memory: memory,
        })
    }
    pub fn caller(&self) -> &CanonicalDigestId {
        &self.caller
    }
    pub fn source(&self) -> &RelationalWorldSourceDescription {
        &self.source
    }
    pub fn fate(&self) -> WorldOriginalFate {
        self.fate
    }
    pub fn effect(&self) -> Option<RelationalWorldEffectCoordinates> {
        self.effect
    }
    pub fn receipt(&self) -> Option<&RelationalCommitReceipt> {
        self.receipt.as_ref()
    }
}
/// Same immutable DATA on Native replay and later complete World readmission.
#[derive(Debug)]
pub struct RelationalWorldSourceCatalog {
    selected_checkpoint: Option<Arc<RelationalWorldSourceCatalog>>,
    originals: ExecutionArray<RelationalWorldOriginalDescription>,
    heads: ExecutionArray<RelationalWorldSourceDescription>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl RelationalWorldSourceCatalog {
    pub fn from_recorded(
        request: &ExecutionRequest<'_, '_>,
        originals: ExecutionArray<RelationalWorldOriginalDescription>,
        heads: ExecutionArray<RelationalWorldSourceDescription>,
        selected_checkpoint: Option<Arc<RelationalWorldSourceCatalog>>,
    ) -> Result<Arc<Self>, RelationalWorldCatalogDenial> {
        let bytes = std::alloc::Layout::new::<[usize; 2]>()
            .extend(std::alloc::Layout::new::<Self>())
            .expect("the fixed catalog layout fits the target pointer width")
            .0
            .pad_to_align()
            .size();
        let memory = request
            .reserve_retained_memory(bytes as u64)
            .map_err(RelationalWorldCatalogDenial::Request)?;
        Ok(Arc::new(Self {
            selected_checkpoint,
            originals,
            heads,
            _memory: memory,
        }))
    }
    /// The World image at the same selected Home checkpoint, before its tails.
    pub fn selected_checkpoint(&self) -> Option<&Self> {
        self.selected_checkpoint.as_deref()
    }
    pub fn originals(&self) -> &[RelationalWorldOriginalDescription] {
        self.originals.elements()
    }
    pub fn heads(&self) -> &[RelationalWorldSourceDescription] {
        self.heads.elements()
    }
}

#[derive(Debug)]
pub enum RelationalWorldCatalogDenial {
    Request(WorkCeilingDenial),
    CapacityOverflow,
}
