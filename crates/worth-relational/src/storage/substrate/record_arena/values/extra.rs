//! Only actual entity/relation payload owners carry retained memory.
use super::{funding, ExtraPayloadDenial, RelationEndpoints};
use crate::identity::data::{LineageId, StructuralFingerprint};
use std::sync::atomic::{AtomicU64, Ordering};
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest};
use worth_foundational::facade::{
    AuthoritativeRecordAspectPatch as Patch, AuthoritativeRecordAspectState as State,
};

static NEXT_EXTRA_BIRTH_SEQ: AtomicU64 = AtomicU64::new(1);

fn next_extra_birth_seq() -> Option<u64> {
    let mut current = NEXT_EXTRA_BIRTH_SEQ.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1)?;
        match NEXT_EXTRA_BIRTH_SEQ.compare_exchange_weak(
            current,
            next,
            Ordering::AcqRel,
            Ordering::Relaxed,
        ) {
            Ok(seq) => return Some(seq),
            Err(observed) => current = observed,
        }
    }
}

/// Exclusive upper bound on the `birth_seq` of every `EntityExtra`/`RelationExtra`
/// already constructed before this call. Returns `None` if exhausted so callers
/// fall back to exact refcount/membership resolution without rollover hazard.
pub(crate) fn current_extra_birth_watermark() -> Option<u64> {
    let next = NEXT_EXTRA_BIRTH_SEQ.load(Ordering::Acquire);
    (next < u64::MAX).then_some(next)
}

#[derive(serde::Serialize)]
pub(crate) struct EntityExtra {
    pub(crate) structural_fingerprint: Option<StructuralFingerprint>,
    pub(crate) lineage_id: Option<LineageId>,
    authoritative_aspect_state: Option<State>,
    #[serde(skip)]
    birth_seq: Option<u64>,
    // Last; moving the whole extra conservatively retains its fixed backing bill.
    #[serde(skip)]
    _funding: Option<ExecutionMemoryReservation>,
}
#[derive(serde::Serialize)]
pub(crate) struct RelationExtra {
    pub(crate) endpoints: Option<RelationEndpoints>,
    authoritative_aspect_state: Option<State>,
    #[serde(skip)]
    birth_seq: Option<u64>,
    #[serde(skip)]
    _funding: Option<ExecutionMemoryReservation>,
}
impl Default for EntityExtra {
    fn default() -> Self {
        Self::from_unbudgeted_parts(None, None, None)
    }
}
impl Default for RelationExtra {
    fn default() -> Self {
        Self::from_unbudgeted_parts(None, None)
    }
}
impl EntityExtra {
    pub(crate) const fn birth_seq(&self) -> Option<u64> {
        self.birth_seq
    }

    pub(crate) fn from_funded_parts(
        structural_fingerprint: Option<StructuralFingerprint>,
        lineage_id: Option<LineageId>,
        authoritative_aspect_state: Option<State>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            structural_fingerprint,
            lineage_id,
            authoritative_aspect_state,
            birth_seq: next_extra_birth_seq(),
            _funding: memory,
        }
    }

    pub(crate) fn empty_for_request(
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        funding::copy(execution, None, |state, memory| Self {
            structural_fingerprint: None,
            lineage_id: None,
            authoritative_aspect_state: state,
            birth_seq: next_extra_birth_seq(),
            _funding: memory,
        })
    }
    /// Ordinary independent input construction; this does not adopt a ticket.
    pub(crate) fn from_unbudgeted_parts(
        structural_fingerprint: Option<StructuralFingerprint>,
        lineage_id: Option<LineageId>,
        authoritative_aspect_state: Option<State>,
    ) -> Self {
        Self {
            structural_fingerprint,
            lineage_id,
            authoritative_aspect_state,
            birth_seq: next_extra_birth_seq(),
            _funding: None,
        }
    }
    pub(crate) fn authoritative_aspect_state(&self) -> Option<&State> {
        self.authoritative_aspect_state.as_ref()
    }
    pub(crate) fn apply_payload(
        execution: &ExecutionRequest<'_, '_>,
        current: Option<&State>,
        patch: &Patch,
    ) -> Result<Self, ExtraPayloadDenial> {
        funding::apply(execution, current, patch, |state, memory| Self {
            structural_fingerprint: None,
            lineage_id: None,
            authoritative_aspect_state: state,
            birth_seq: next_extra_birth_seq(),
            _funding: memory,
        })
    }
    pub(crate) fn copy_for_request(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        funding::copy(
            execution,
            self.authoritative_aspect_state(),
            |state, memory| Self {
                structural_fingerprint: self.structural_fingerprint,
                lineage_id: self.lineage_id,
                authoritative_aspect_state: state,
                birth_seq: next_extra_birth_seq(),
                _funding: memory,
            },
        )
    }
}
impl RelationExtra {
    pub(crate) const fn birth_seq(&self) -> Option<u64> {
        self.birth_seq
    }

    pub(crate) fn from_funded_parts(
        endpoints: Option<RelationEndpoints>,
        authoritative_aspect_state: Option<State>,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Self {
        Self {
            endpoints,
            authoritative_aspect_state,
            birth_seq: next_extra_birth_seq(),
            _funding: memory,
        }
    }

    pub(crate) fn empty_for_request(
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        funding::copy(execution, None, |state, memory| Self {
            endpoints: None,
            authoritative_aspect_state: state,
            birth_seq: next_extra_birth_seq(),
            _funding: memory,
        })
    }
    pub(crate) fn from_unbudgeted_parts(
        endpoints: Option<RelationEndpoints>,
        authoritative_aspect_state: Option<State>,
    ) -> Self {
        Self {
            endpoints,
            authoritative_aspect_state,
            birth_seq: next_extra_birth_seq(),
            _funding: None,
        }
    }
    pub(crate) fn authoritative_aspect_state(&self) -> Option<&State> {
        self.authoritative_aspect_state.as_ref()
    }
    pub(crate) fn apply_payload(
        execution: &ExecutionRequest<'_, '_>,
        current: Option<&State>,
        patch: &Patch,
    ) -> Result<Self, ExtraPayloadDenial> {
        funding::apply(execution, current, patch, |state, memory| Self {
            endpoints: None,
            authoritative_aspect_state: state,
            birth_seq: next_extra_birth_seq(),
            _funding: memory,
        })
    }
    pub(crate) fn copy_for_request(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, crate::execution::RelationalExecutionDenialCause> {
        funding::copy(
            execution,
            self.authoritative_aspect_state(),
            |state, memory| Self {
                endpoints: self.endpoints.clone(),
                authoritative_aspect_state: state,
                birth_seq: next_extra_birth_seq(),
                _funding: memory,
            },
        )
    }
}
impl Clone for EntityExtra {
    fn clone(&self) -> Self {
        Self::from_unbudgeted_parts(
            self.structural_fingerprint,
            self.lineage_id,
            self.authoritative_aspect_state.clone(),
        )
    }
}
impl Clone for RelationExtra {
    fn clone(&self) -> Self {
        Self::from_unbudgeted_parts(
            self.endpoints.clone(),
            self.authoritative_aspect_state.clone(),
        )
    }
}
impl std::fmt::Debug for EntityExtra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EntityExtra")
            .field("structural_fingerprint", &self.structural_fingerprint)
            .field("lineage_id", &self.lineage_id)
            .field(
                "authoritative_aspect_state",
                &self.authoritative_aspect_state,
            )
            .finish()
    }
}
impl std::fmt::Debug for RelationExtra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationExtra")
            .field("endpoints", &self.endpoints)
            .field(
                "authoritative_aspect_state",
                &self.authoritative_aspect_state,
            )
            .finish()
    }
}

impl PartialEq for EntityExtra {
    fn eq(&self, other: &Self) -> bool {
        self.structural_fingerprint == other.structural_fingerprint
            && self.lineage_id == other.lineage_id
            && self.authoritative_aspect_state == other.authoritative_aspect_state
    }
}
impl Eq for EntityExtra {}
impl PartialEq for RelationExtra {
    fn eq(&self, other: &Self) -> bool {
        self.endpoints == other.endpoints
            && self.authoritative_aspect_state == other.authoritative_aspect_state
    }
}
impl Eq for RelationExtra {}
impl PartialEq for crate::storage::substrate::StorageAllocation<EntityExtra> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for crate::storage::substrate::StorageAllocation<EntityExtra> {}
impl PartialEq for crate::storage::substrate::StorageAllocation<RelationExtra> {
    fn eq(&self, other: &Self) -> bool {
        self.as_ref() == other.as_ref()
    }
}
impl Eq for crate::storage::substrate::StorageAllocation<RelationExtra> {}
