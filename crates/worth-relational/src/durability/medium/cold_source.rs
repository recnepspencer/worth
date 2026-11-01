use serde::{Deserialize, Serialize};

use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause;

/// Content identity of a checkpoint member. The home resolves physical
/// placement through its currently selected checkpoint's member table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RelationalDurableMemberLocator {
    pub sha256: [u8; 32],
    pub length: u64,
}

/// Read-only committed storage, injected before recovery readmission. A fork
/// may share this port without acquiring any publication authority.
pub trait RelationalDurableColdSource: Send + Sync + std::fmt::Debug {
    /// Capture the provider's actual immutable native cut. No legacy read fallback.
    fn checkpoint_read_protection(
        &self,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            super::RelationalDurableReadBounds,
            super::RelationalDurableReadPayload,
        ),
        super::RelationalDurableOwnedReadRefusal,
    > {
        Err(super::RelationalDurableOwnedReadRefusal::UnsupportedProvider)
    }
    /// Retain only the provider's already installed verified selected read.
    /// This must never recapture its latest physical root or reader.
    fn checkpoint_selected_read_protection(
        &self,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<super::RelationalDurableReadPayload, super::RelationalDurableOwnedReadRefusal> {
        Err(super::RelationalDurableOwnedReadRefusal::UnsupportedProvider)
    }
    fn read_commit_owned(
        &self,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
        _original: &(dyn std::any::Any + Send + Sync),
        _position: PatchStreamPosition,
        _limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, super::RelationalDurableOwnedReadRefusal>
    {
        Err(super::RelationalDurableOwnedReadRefusal::UnsupportedProvider)
    }
    fn read_transition_owned(
        &self,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
        _original: &(dyn std::any::Any + Send + Sync),
        _sequence: crate::branch::RelationalBranchTransitionSequence,
        _limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, super::RelationalDurableOwnedReadRefusal>
    {
        Err(super::RelationalDurableOwnedReadRefusal::UnsupportedProvider)
    }
    fn read_member_owned(
        &self,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
        _original: &(dyn std::any::Any + Send + Sync),
        _member: RelationalDurableMemberLocator,
        _limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, super::RelationalDurableOwnedReadRefusal>
    {
        Err(super::RelationalDurableOwnedReadRefusal::UnsupportedProvider)
    }

    /// Provider-only admission of the ORIGINAL opaque payload. This callback
    /// must perform no I/O, allocation, locking or payload destruction. Native
    /// has already checked the immutable issuing source's data identity.
    fn validate_member_custody_retirement(
        &self,
        _original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), super::RelationalDurableMemberRetirementRefusal> {
        Err(super::RelationalDurableMemberRetirementRefusal::UnsupportedProvider)
    }

    /// Advance one actual obligation using only nonwaiting lock admission and
    /// the original request. Complete requires disarmed business/source custody;
    /// keep the original inert payload allocated until unbudgeted reclamation.
    /// Never allocate, reserve, rehash, shrink or invoke arbitrary payload Drop.
    fn advance_member_custody_retirement(
        &self,
        _execution: &worth_execution::ExecutionRequest<'_, '_>,
        _original: &mut (dyn std::any::Any + Send + Sync),
    ) -> super::RelationalDurableMemberRetirementObservation {
        super::RelationalDurableMemberRetirementObservation::Refused(
            super::RelationalDurableMemberRetirementRefusal::UnsupportedProvider,
        )
    }

    /// Controlled selected-basis custody; legacy retention is never a fallback.
    fn retain_members_for_request(
        &self,
        _request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        _issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<super::RelationalDurableMemberCustody, super::RelationalDurableOwnedReadRefusal>
    {
        Err(super::RelationalDurableOwnedReadRefusal::UnsupportedProvider)
    }

    /// Retain selected placement without reading member payloads. The provider
    /// must preserve it across checkpoint replacement until the last owner
    /// drops. This makes no claim about payload availability or integrity.
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<super::RelationalDurableMemberCustody, RelationalStorageUnavailableCause>;

    /// Return member bytes only after enforcing the caller's byte limit.
    /// Relational independently checks the frame, content identity and schema.
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause>;

    /// Return the canonical artifact at exactly this position. Missing durable
    /// content refuses; implementations must never return an empty substitute.
    fn read_commit(
        &self,
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause>;
}
