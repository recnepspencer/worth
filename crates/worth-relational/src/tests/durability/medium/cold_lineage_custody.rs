//! A replaced home directory may remove members that have no placement owner.
use super::cold_lineage_event_fixture::reopened_with_source;
use super::*;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberCustody};
use std::collections::BTreeMap;
use std::sync::Mutex;

#[derive(Debug)]
struct ReplacingHomeSource {
    producer: Arc<MemberSource>,
    custody: Arc<Mutex<BTreeMap<RelationalDurableMemberLocator, usize>>>,
    replaced: AtomicBool,
}
struct MemberOwners {
    members: Vec<RelationalDurableMemberLocator>,
    custody: Arc<Mutex<BTreeMap<RelationalDurableMemberLocator, usize>>>,
    _memory: Option<worth_execution::ExecutionMemoryReservation>,
}
impl Drop for MemberOwners {
    fn drop(&mut self) {
        let mut custody = self.custody.lock().unwrap();
        for member in &self.members {
            let count = custody.get_mut(member).unwrap();
            *count -= 1;
            if *count == 0 {
                custody.remove(member);
            }
        }
    }
}
impl RelationalDurableColdSource for ReplacingHomeSource {
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::medium::RelationalDurableReadPayload,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        self.producer.checkpoint_selected_read_protection(request)
    }
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<
        RelationalDurableMemberCustody,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        let bytes = std::alloc::Layout::array::<RelationalDurableMemberLocator>(members.len())
            .map_err(|_| Refusal::Bounds)?
            .size();
        let memory = request.reserve_retained_memory(bytes as u64)?;
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            self.register(members, memory).map_err(Refusal::Storage)
        })
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<
        worth_execution::ExecutionImmutableBytes,
        crate::durability::medium::RelationalDurableOwnedReadRefusal,
    > {
        self.check_placement(member)
            .map_err(crate::durability::medium::RelationalDurableOwnedReadRefusal::Storage)?;
        self.producer
            .read_member_owned(request, original, member, limit)
    }
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(
            self.register(members, None)?,
        ))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.check_placement(locator)?;
        self.producer.read_member(locator, limit)
    }
    fn read_commit(
        &self,
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.producer.read_commit(position, limit)
    }
}
impl ReplacingHomeSource {
    fn register(
        &self,
        members: &[RelationalDurableMemberLocator],
        memory: Option<worth_execution::ExecutionMemoryReservation>,
    ) -> Result<MemberOwners, Cause> {
        if members
            .iter()
            .any(|member| !self.producer.members.contains_key(member))
        {
            return Err(Cause::MissingMember);
        }
        let mut custody = self.custody.lock().unwrap();
        for member in members {
            *custody.entry(*member).or_default() += 1;
        }
        Ok(MemberOwners {
            members: members.to_vec(),
            custody: self.custody.clone(),
            _memory: memory,
        })
    }
    fn check_placement(&self, member: RelationalDurableMemberLocator) -> Result<(), Cause> {
        if self.replaced.load(Ordering::Acquire)
            && !self.custody.lock().unwrap().contains_key(&member)
        {
            return Err(Cause::MissingMember);
        }
        Ok(())
    }
}

#[test]
fn an_older_lineage_directory_retains_its_cold_pages_after_home_replacement() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut placed = None;
    let fixture = reopened_with_source(execution, |producer| {
        let source = Arc::new(ReplacingHomeSource {
            producer,
            custody: Arc::default(),
            replaced: AtomicBool::new(false),
        });
        placed = Some(source.clone());
        source
    });
    let source = placed.unwrap();
    let directory = fixture
        .runtime
        .durability
        .checkpoint_selection()
        .snapshot()
        .unwrap();
    let node_member = directory
        .manifest
        .lineage
        .iter()
        .find(|page| (page.first..=page.last).contains(&fixture.seed.0))
        .unwrap()
        .member;
    // The source models another runtime replacing the selected home checkpoint.
    // Only members retained by a live residency owner survive that replacement.
    source.replaced.store(true, Ordering::Release);
    drop(directory);
    let node = fixture
        .runtime
        .lineage
        .demand_node(execution, fixture.seed)
        .unwrap()
        .unwrap();
    assert_eq!(node.node().entity_id(), fixture.original);
    assert!(source.custody.lock().unwrap().contains_key(&node_member));
    drop(node);
    drop(fixture);
    assert!(
        source.custody.lock().unwrap().is_empty(),
        "the last owner releases placement"
    );
}
