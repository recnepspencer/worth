//! Original per-member placement obligations retained across checkpoint replacement.
use super::immutable_members::ImmutableCheckpointMemberSource;
use std::collections::BTreeMap;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex, TryLockError,
};
use worth_execution::{ExecutionImmutableBytes, ExecutionMemoryReservation, ExecutionRequest};
use worth_relational::facade::{
    durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug)]
pub struct ReplacingMemberSource {
    pub producer: Arc<ImmutableCheckpointMemberSource>,
    custody: Arc<Mutex<BTreeMap<RelationalDurableMemberLocator, usize>>>,
    replaced: AtomicBool,
}
struct MemberOwners {
    members: Vec<RelationalDurableMemberLocator>,
    next: usize,
    custody: Arc<Mutex<BTreeMap<RelationalDurableMemberLocator, usize>>>,
    producer_original: Option<RelationalDurableReadPayload>,
    _memory: Option<ExecutionMemoryReservation>,
}
impl Drop for MemberOwners {
    fn drop(&mut self) {
        // Abandonment of the remaining original registration is explicitly unbudgeted.
        let mut custody = self
            .custody
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        for member in &self.members[self.next..] {
            let count = custody
                .get_mut(member)
                .expect("only registered originals own a member");
            *count -= 1;
        }
    }
}
impl ReplacingMemberSource {
    pub fn new(producer: Arc<ImmutableCheckpointMemberSource>) -> Arc<Self> {
        Arc::new(Self {
            producer,
            custody: Arc::default(),
            replaced: AtomicBool::new(false),
        })
    }
    pub fn replace_selected_placement(&self) {
        self.replaced.store(true, Ordering::Release);
    }
    pub fn has_owner(&self, member: RelationalDurableMemberLocator) -> bool {
        self.custody
            .lock()
            .unwrap()
            .get(&member)
            .is_some_and(|count| *count > 0)
    }
    pub fn owners_released(&self) -> bool {
        self.custody
            .lock()
            .unwrap()
            .values()
            .all(|count| *count == 0)
    }
    fn register(
        &self,
        members: &[RelationalDurableMemberLocator],
        memory: Option<ExecutionMemoryReservation>,
        producer_original: Option<RelationalDurableReadPayload>,
    ) -> Result<MemberOwners, Cause> {
        if members.iter().any(|member| {
            !self.producer.members.contains_key(member)
                && !self.producer.canonical_members.contains_key(member)
        }) {
            return Err(Cause::MissingMember);
        }
        let mut custody = self.custody.lock().unwrap();
        for member in members.iter() {
            let count = custody.entry(*member).or_default();
            *count = count.checked_add(1).expect(
                "each count owns a nonzero, distinct admitted locator cell in an original registration");
        }
        Ok(MemberOwners {
            members: members.to_vec(),
            next: 0,
            custody: self.custody.clone(),
            producer_original,
            _memory: memory,
        })
    }
    fn check_placement(&self, member: RelationalDurableMemberLocator) -> Result<(), Cause> {
        if self.replaced.load(Ordering::Acquire) && !self.has_owner(member) {
            return Err(Cause::MissingMember);
        }
        Ok(())
    }
}
impl RelationalDurableColdSource for ReplacingMemberSource {
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        self.producer.checkpoint_selected_read_protection(request)
    }
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        use RelationalDurableOwnedReadRefusal as Refusal;
        if issuer.as_ptr().cast::<()>() != std::ptr::from_ref(self).cast::<()>() {
            return Err(Refusal::ForeignSource);
        }
        let bytes = std::alloc::Layout::array::<RelationalDurableMemberLocator>(members.len())
            .map_err(|_| Refusal::Bounds)?
            .size();
        let memory = request.reserve_retained_memory(bytes as u64)?;
        let producer_original = self.producer.checkpoint_selected_read_protection(request)?;
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            self.register(members, memory, Some(producer_original))
                .map_err(Refusal::Storage)
        })
    }
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.register(members, None, None)
            .map(RelationalDurableMemberCustody::new)
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        let Some(owner) = original.downcast_ref::<MemberOwners>() else {
            return self.producer.validate_member_custody_retirement(original);
        };
        if Arc::ptr_eq(&owner.custody, &self.custody) {
            Ok(())
        } else {
            Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        use RelationalDurableMemberRetirementObservation as O;
        if !original.is::<MemberOwners>() {
            return self
                .producer
                .advance_member_custody_retirement(request, original);
        }
        if let Err(cause) = self.validate_member_custody_retirement(original) {
            return O::Refused(cause);
        }
        if let Err(cause) = request.consult() {
            return O::Stopped(cause);
        }
        let owner = original
            .downcast_mut::<MemberOwners>()
            .expect("validated original registration");
        if owner.next < owner.members.len() {
            let mut custody = match self.custody.try_lock() {
                Ok(custody) => custody,
                Err(TryLockError::WouldBlock) => return O::Busy,
                Err(TryLockError::Poisoned(error)) => error.into_inner(),
            };
            let Some(count) = custody.get_mut(&owner.members[owner.next]) else {
                return O::Refused(RelationalDurableMemberRetirementRefusal::ForeignCustody);
            };
            if *count == 0 {
                return O::Refused(RelationalDurableMemberRetirementRefusal::ForeignCustody);
            }
            *count -= 1;
            owner.next += 1;
        }
        // Zero metadata slots are inert, retained until the backend's unbudgeted Drop.
        if owner.next == owner.members.len() {
            O::Complete
        } else {
            O::Advanced
        }
    }
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        self.check_placement(member)
            .map_err(RelationalDurableOwnedReadRefusal::Storage)?;
        if let Some(owner) = original.downcast_ref::<MemberOwners>() {
            self.validate_member_custody_retirement(original)
                .map_err(|_| RelationalDurableOwnedReadRefusal::ForeignProtection)?;
            let Some(producer_original) = owner.producer_original.as_ref() else {
                return Err(RelationalDurableOwnedReadRefusal::ForeignProtection);
            };
            self.producer
                .read_member_owned(request, producer_original.as_ref(), member, limit)
        } else {
            self.producer
                .read_member_owned(request, original, member, limit)
        }
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.check_placement(member)?;
        self.producer.read_member(member, limit)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.producer.read_commit(position, limit)
    }
}
