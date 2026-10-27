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
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
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
        Ok(RelationalDurableMemberCustody::new(MemberOwners {
            members: members.to_vec(),
            custody: self.custody.clone(),
        }))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        if self.replaced.load(Ordering::Acquire)
            && !self.custody.lock().unwrap().contains_key(&locator)
        {
            return Err(Cause::MissingMember);
        }
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
