//! Provider-owned placement retention, independent of payload admission.
use super::RelationalDurableColdSource;
use std::any::Any;
use std::sync::{Arc, Weak};

mod native_source_slot;
pub(crate) use native_source_slot::NativeSourceMemberState;
mod native_slot;
pub(crate) use native_slot::{NativeMemberRetirementHandoff, NativeMemberSlotDenial};
mod retirement;
pub use retirement::{
    RelationalDurableMemberRetirement, RelationalDurableMemberRetirementObservation,
    RelationalDurableMemberRetirementRefusal, RetiredRelationalDurableMemberCustody,
};

struct OriginalMemberCustody {
    owner: Box<dyn Any + Send + Sync>,
    source: Option<Weak<dyn RelationalDurableColdSource>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

/// Keeps the provider's selected member placement alive until the last clone
/// drops. This does not verify bytes or authorize record reads or publication.
#[derive(Clone)]
pub struct RelationalDurableMemberCustody {
    owner: Arc<OriginalMemberCustody>,
}

impl RelationalDurableMemberCustody {
    pub fn try_new_from_source_for_request<T: Send + Sync + 'static>(
        source: Weak<dyn RelationalDurableColdSource>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        make_owner: impl FnOnce() -> Result<T, super::RelationalDurableOwnedReadRefusal>,
    ) -> Result<Self, super::RelationalDurableOwnedReadRefusal> {
        let control = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<OriginalMemberCustody>())
            .map_err(|_| super::RelationalDurableOwnedReadRefusal::Bounds)?
            .0
            .pad_to_align()
            .size();
        let bytes = control
            .checked_add(std::alloc::Layout::new::<T>().size())
            .ok_or(super::RelationalDurableOwnedReadRefusal::Bounds)?;
        let memory = request.reserve_retained_memory(bytes as u64)?;
        request
            .check_live()
            .map_err(|cause| super::RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        let owner = make_owner()?;
        // No late request consultation can discard a successfully registered owner.
        Ok(Self {
            owner: Arc::new(OriginalMemberCustody {
                owner: Box::new(owner),
                source: Some(source),
                _memory: memory,
            }),
        })
    }
    pub(crate) fn capture_selected_read(
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<Self, super::RelationalDurableOwnedReadRefusal> {
        let bytes = std::alloc::Layout::new::<(usize, usize)>()
            .extend(std::alloc::Layout::new::<OriginalMemberCustody>())
            .map_err(|_| super::RelationalDurableOwnedReadRefusal::Bounds)?
            .0
            .pad_to_align()
            .size();
        let memory = request.reserve_retained_memory(bytes as u64)?;
        request
            .check_live()
            .map_err(|cause| super::RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        let issuer = Arc::downgrade(source);
        let owner = source.checkpoint_selected_read_protection(request)?;
        // The original provider Box moves directly; its control was admitted first.
        Ok(Self {
            owner: Arc::new(OriginalMemberCustody {
                owner,
                source: Some(issuer),
                _memory: memory,
            }),
        })
    }
    pub(crate) fn capture(
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        members: &[super::RelationalDurableMemberLocator],
    ) -> Result<Self, super::RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| super::RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        let issuer = Arc::downgrade(source);
        let original = source.retain_members_for_request(request, members, issuer)?;
        if original
            .owner
            .source
            .as_ref()
            .is_none_or(|issuer| issuer.as_ptr().cast::<()>() != Arc::as_ptr(source).cast::<()>())
        {
            return Err(super::RelationalDurableOwnedReadRefusal::ForeignSource);
        }
        Ok(original)
    }
    pub(crate) fn read_selected_member(
        &self,
        source: &Arc<dyn RelationalDurableColdSource>,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        member: super::RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, super::RelationalDurableOwnedReadRefusal>
    {
        use super::RelationalDurableOwnedReadRefusal as Refusal;
        let issuer = self
            .owner
            .source
            .as_ref()
            .ok_or(Refusal::ForeignProtection)?;
        if issuer.as_ptr().cast::<()>() != Arc::as_ptr(source).cast::<()>() {
            return Err(Refusal::ForeignSource);
        }
        if member.length > limit {
            return Err(Refusal::Storage(
                crate::storage::data::RelationalStorageUnavailableCause::LimitExceeded {
                    length: member.length,
                    limit,
                },
            ));
        }
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        source.read_member_owned(request, self.owner.owner.as_ref(), member, limit)
    }
    /// The provider's owner releases its placement obligation on Drop. An
    /// immutable source may use `()` because its source handle already retains
    /// every addressed byte for the handle's entire lifetime.
    pub fn new(owner: impl Send + Sync + 'static) -> Self {
        Self {
            owner: Arc::new(OriginalMemberCustody {
                owner: Box::new(owner),
                source: None,
                _memory: None,
            }),
        }
    }

    /// Bind the original provider payload to its actual issuing source. The
    /// weak identity keeps its allocation distinct without keeping it serving.
    /// Boxed payload custody adds one allocation and one
    /// indirection to the former unsized Arc representation.
    pub fn new_from_source(
        owner: impl Send + Sync + 'static,
        source: Weak<dyn RelationalDurableColdSource>,
    ) -> Self {
        Self {
            owner: Arc::new(OriginalMemberCustody {
                owner: Box::new(owner),
                source: Some(source),
                _memory: None,
            }),
        }
    }
}

impl std::fmt::Debug for RelationalDurableMemberCustody {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RelationalDurableMemberCustody")
            .finish_non_exhaustive()
    }
}
