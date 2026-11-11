//! Producer-written region members and their immutable counted archive.
use super::super::{capture_region_members, RelationalColdRegion, RelationalRegionResidency};
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
    RelationalDurableOwnedReadRefusal,
};
use crate::history::data::BranchId;
use crate::identity::data::KindId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionContentCommitment;
use crate::tests::support::*;
use crate::transactions::data::{MutationIntent, WorkerIntentBatch};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub(super) struct CountedSource {
    pub(super) members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    pub(super) reads: AtomicUsize,
    pub(super) history_reads: AtomicUsize,
    pub(super) history: RelationalDurableMemberLocator,
    pub(super) corrupt: AtomicBool,
}

impl RelationalDurableColdSource for CountedSource {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        // This source owns its immutable archive for its entire Arc lifetime.
        // Only the actual custody owner needs new request-funded backing.
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }

    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        use RelationalDurableOwnedReadRefusal as Refusal;
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        if !original.is::<()>() {
            return Err(Refusal::ForeignProtection);
        }
        let bytes = self.member(locator, limit).map_err(Refusal::Storage)?;
        let mut copy =
            worth_execution::ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
                .map_err(Refusal::Buffer)?;
        if let Some((&first, rest)) = bytes.split_first() {
            copy.push(if self.corrupt.load(Ordering::SeqCst) {
                first ^ 1
            } else {
                first
            })
            .map_err(Refusal::Buffer)?;
            copy.extend_from_slice(rest).map_err(Refusal::Buffer)?;
        }
        copy.seal_immutable_bytes().map_err(Refusal::Buffer)
    }

    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<crate::durability::medium::RelationalDurableMemberCustody, Cause> {
        Ok(crate::durability::medium::RelationalDurableMemberCustody::new(()))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        let mut bytes = self.member(locator, limit)?.to_vec();
        if self.corrupt.load(Ordering::SeqCst) {
            bytes[0] ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(&self, _: PatchStreamPosition, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
}

impl CountedSource {
    fn member(&self, locator: RelationalDurableMemberLocator, limit: u64) -> Result<&[u8], Cause> {
        self.reads.fetch_add(1, Ordering::SeqCst);
        if locator == self.history {
            self.history_reads.fetch_add(1, Ordering::SeqCst);
        }
        let bytes = self.members.get(&locator).ok_or(Cause::MissingMember)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes)
    }
}

pub(super) struct RegionFixture {
    pub(super) cell: Arc<RelationalRegionResidency>,
    pub(super) cold: RelationalColdRegion,
    pub(super) source: Arc<CountedSource>,
    pub(super) entity: crate::identity::data::EntityId,
    pub(super) relation: crate::identity::data::RelationId,
    pub(super) expected_digest: [u8; 32],
    pub(super) expected_history: usize,
}

pub(super) fn region_with_history() -> RegionFixture {
    let request = crate::test_execution_policy::serial_request();
    let execution = &worth_execution::ExecutionRequest::serial(&request);
    let runtime = runtime_with_test_schema();
    let entity = create_entity(execution, &runtime, "cold-first");
    for label in ["cold-second", "cold-third", "cold-fourth"] {
        update_entity_and_release_snapshot(execution, &runtime, entity, label);
    }
    let target = create_entity(execution, &runtime, "cold-target");
    let relation_commit = create_relation_outcome(execution, &runtime, entity, target, "cold-edge");
    let relation = changed_relations(&relation_commit)[0];
    release_test_commit_snapshot(&runtime, &relation_commit);
    let other = create_entity(execution, &runtime, "cold-other-target");
    for target in [other, target] {
        let mut transaction = test_owner_begin_transaction_for_main(&runtime);
        transaction
            .push_batch(
                WorkerIntentBatch::new("relation-history").push(MutationIntent::Relation(
                    crate::transactions::data::RelationMutationIntent::UpdateEndpoints(
                        crate::transactions::data::UpdateRelationEndpointsIntent {
                            relation_id: relation,
                            kind_id: KindId(2),
                            source: crate::transactions::data::EntityReference::Existing(entity),
                            target: crate::transactions::data::EntityReference::Existing(target),
                        },
                    ),
                )),
                execution.allocation_policy(),
            )
            .unwrap();
        let committed = transaction.commit(execution, &runtime).unwrap();
        release_test_commit_snapshot(&runtime, &committed);
    }
    let root = runtime
        .history
        .branch_cell(&BranchId("main".into()))
        .and_then(|cell| cell.root())
        .unwrap();
    let partition = root
        .partition_state(entity.partition_id)
        .expect("fixture root has admitted Full custody")
        .unwrap();
    let symbols = Arc::new(runtime.services.symbols.interner_snapshot());
    let (commitment, _) = PartitionContentCommitment::capture(partition, &symbols, None).unwrap();
    let schema = Arc::new(root.schema_authority().clone());
    let schema_digest = crate::durability::data::DurableBranchRootSchemaImage::capture(&schema)
        .unwrap()
        .carrier_digest();
    let members = capture_region_members(partition, &commitment, &schema, schema_digest).unwrap();
    let source = Arc::new(CountedSource {
        history: members.history.0,
        members: BTreeMap::from([members.head.clone(), members.history.clone()]),
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    });
    let expected_digest = commitment.digest(partition.partition_id);
    let expected_history = partition
        .entity_arena
        .metadata_history_at(entity.slot_index())
        .unwrap()
        .len();
    assert!(
        expected_history > 1,
        "fixture must have real committed history"
    );
    let symbol_authority = crate::runtime::RuntimeSymbolTable::default();
    symbol_authority.replace((*symbols).clone());
    let cold = RelationalColdRegion {
        history_members: members.history_members,
        kind_authority: members.kind_authority,
        source: Some(source.clone()),
        partition_id: partition.partition_id,
        head: members.head.0,
        history: members.history.0,
        schema,
        schema_carrier_digest: schema_digest,
        symbols: symbol_authority,
        content_digest: expected_digest,
        entity_slots: partition.entity_arena.slot_count(),
        relation_slots: partition.relation_arena.slot_count(),
        unavailable_records: partition.entity_arena.lifecycle_counts().unavailable
            + partition.relation_arena.lifecycle_counts().unavailable,
    };
    RegionFixture {
        cell: RelationalRegionResidency::cold(cold.clone()).unwrap(),
        cold,
        source,
        entity,
        relation,
        expected_digest,
        expected_history,
    }
}

#[derive(Debug)]
pub(super) struct CountedPlacementSource {
    pub(super) source: Arc<CountedSource>,
    pub(super) active: Arc<AtomicUsize>,
    pub(super) refusal: Mutex<Option<Cause>>,
}

struct CustodyOwner {
    active: Arc<AtomicUsize>,
    _source: RelationalDurableMemberCustody,
}

impl Drop for CustodyOwner {
    fn drop(&mut self) {
        self.active.fetch_sub(1, Ordering::SeqCst);
    }
}

impl RelationalDurableColdSource for CountedPlacementSource {
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
        if let Some(cause) = *self.refusal.lock().unwrap() {
            return Err(Refusal::Storage(cause));
        }
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            let source: Arc<dyn RelationalDurableColdSource> = self.source.clone();
            let original =
                source.retain_members_for_request(request, members, Arc::downgrade(&source))?;
            self.active.fetch_add(1, Ordering::SeqCst);
            Ok(CustodyOwner {
                active: self.active.clone(),
                _source: original,
            })
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
        let original = original.downcast_ref::<CustodyOwner>().ok_or(
            crate::durability::medium::RelationalDurableOwnedReadRefusal::ForeignProtection,
        )?;
        let source: Arc<dyn RelationalDurableColdSource> = self.source.clone();
        original
            ._source
            .read_selected_member(&source, request, member, limit)
    }

    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        if let Some(cause) = *self.refusal.lock().unwrap() {
            return Err(cause);
        }
        let source = self.source.retain_members(members)?;
        self.active.fetch_add(1, Ordering::SeqCst);
        Ok(RelationalDurableMemberCustody::new(CustodyOwner {
            active: self.active.clone(),
            _source: source,
        }))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.source.read_member(locator, limit)
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.source.read_commit(position, limit)
    }
}
