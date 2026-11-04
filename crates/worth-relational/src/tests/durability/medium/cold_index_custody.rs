//! An old index image needs placement custody independently of its directory.
use super::*;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberCustody};
#[derive(Debug)]
struct ReplacedIndexSource {
    producer: Arc<MemberSource>,
    index: RelationalDurableMemberLocator,
    owners: Arc<AtomicUsize>,
    replaced: AtomicBool,
}
struct IndexPlacement(Arc<AtomicUsize>);
impl Drop for IndexPlacement {
    fn drop(&mut self) {
        assert!(self.0.fetch_sub(1, Ordering::AcqRel) > 0);
    }
}
impl RelationalDurableColdSource for ReplacedIndexSource {
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
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            Ok(members.contains(&self.index).then(|| {
                self.owners.fetch_add(1, Ordering::AcqRel);
                IndexPlacement(self.owners.clone())
            }))
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
        let underlying = self.producer.retain_members(members)?;
        let placement = members.contains(&self.index).then(|| {
            self.owners.fetch_add(1, Ordering::AcqRel);
            IndexPlacement(self.owners.clone())
        });
        Ok(RelationalDurableMemberCustody::new((underlying, placement)))
    }
    fn read_member(
        &self,
        member: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.check_placement(member)?;
        self.producer.read_member(member, limit)
    }
    fn read_commit(
        &self,
        position: crate::publication::patch::data::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.producer.read_commit(position, limit)
    }
}
impl ReplacedIndexSource {
    fn check_placement(&self, member: RelationalDurableMemberLocator) -> Result<(), Cause> {
        if member == self.index
            && self.replaced.load(Ordering::Acquire)
            && self.owners.load(Ordering::Acquire) == 0
        {
            return Err(Cause::MissingMember);
        }
        Ok(())
    }
}
#[test]
fn an_older_index_directory_retains_its_cold_frame_after_home_replacement() {
    let host = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&host);
    let mut placed = None;
    let fixture = super::cold_index_fixture::reopened_with_source(execution, |producer| {
        let index = producer
            .members
            .iter()
            .find_map(|(member, bytes)| {
                use crate::durability::log::native_checkpoint_codec::member_frame::{
                    self, MemberKind,
                };
                member_frame::decode(bytes, *member, MemberKind::DerivedIndex)
                    .is_ok()
                    .then_some(*member)
            })
            .expect("the actual producer wrote its generation frame");
        let source = Arc::new(ReplacedIndexSource {
            producer,
            index,
            owners: Arc::default(),
            replaced: AtomicBool::new(false),
        });
        placed = Some(source.clone());
        source
    });
    let source = placed.unwrap();
    source.replaced.store(true, Ordering::Release);
    let (_, basis) = fixture
        .runtime
        .observe_branch(&fixture.runtime.main_branch_identity())
        .unwrap();
    let view = fixture
        .runtime
        .read_truth()
        .project_observation(&basis.observation())
        .unwrap();
    let pin = fixture
        .runtime
        .read_truth()
        .pin_view_read_capability(&view, &[fixture.index], execution)
        .unwrap();
    assert_eq!(
        pin.index_generation(fixture.index),
        Some(fixture.generation.generation_id)
    );
    assert!(source
        .producer
        .member_reads
        .lock()
        .unwrap()
        .contains(&fixture.member));
    drop(pin);
    drop(view);
    drop(basis);
    drop(fixture);
    assert_eq!(
        source.owners.load(Ordering::Acquire),
        0,
        "the final cold image owner releases placement"
    );
}
