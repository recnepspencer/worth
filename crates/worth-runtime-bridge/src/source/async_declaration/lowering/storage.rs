//! Original-request funding stays with the immutable lowering body.
use super::*;
use std::alloc::Layout;
use worth_execution::{
    ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum LoweredBridgeAsyncSourceDeclarationBody {
    RequestResponse {
        declaration: ResourceNodeDeclaration,
        descriptor: LoweredResourceDescriptor,
    },
    SubscriptionBacked {
        declaration: AsyncNodeCapabilityDeclaration,
        bundle: LoweredAsyncNodeCapabilityBundle,
    },
}

pub(crate) struct OriginalLoweringStorage(Option<ExecutionMemoryReservation>);

impl OriginalLoweringStorage {
    pub(crate) fn reserve(execution: ExecutionRequest<'_, '_>) -> Result<Self, WorkCeilingDenial> {
        execution.reserve_retained_memory(Self::bytes()?).map(Self)
    }

    pub(crate) fn bytes() -> Result<u64, WorkCeilingDenial> {
        let overflow = || WorkCeilingDenial::Admission(LeaseDenial::ChargedBytesOverflow);
        let arc = |payload: Layout| {
            Layout::new::<[usize; 2]>()
                .extend(payload)
                .map(|(layout, _)| layout.pad_to_align().size())
                .map_err(|_| overflow())
        };
        let digest_len = "bridge-async-lowering:sha256:"
            .len()
            .checked_add(
                <Sha256 as Digest>::output_size()
                    .checked_mul(2)
                    .ok_or_else(overflow)?,
            )
            .ok_or_else(overflow)?;
        let digest = arc(Layout::array::<u8>(digest_len).map_err(|_| overflow())?)?;
        let bytes = arc(Layout::new::<LoweringBodyStorage>())?
            .checked_add(digest)
            .ok_or_else(overflow)?;
        u64::try_from(bytes).map_err(|_| overflow())
    }
}

pub(super) struct LoweringBodyStorage {
    pub(super) body: LoweredBridgeAsyncSourceDeclarationBody,
    pub(super) template_digest: Arc<str>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl LoweringBodyStorage {
    pub(super) fn retain(
        body: LoweredBridgeAsyncSourceDeclarationBody,
        template_digest: Arc<str>,
        storage: Option<OriginalLoweringStorage>,
    ) -> Arc<Self> {
        Arc::new(Self {
            body,
            template_digest,
            _memory: storage.and_then(|held| held.0),
        })
    }
}
impl std::fmt::Debug for LoweringBodyStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.body.fmt(f)
    }
}
impl PartialEq for LoweringBodyStorage {
    fn eq(&self, other: &Self) -> bool {
        self.body == other.body
    }
}
impl Eq for LoweringBodyStorage {}
