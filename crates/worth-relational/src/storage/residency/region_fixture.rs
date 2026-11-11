//! Producer-written region members and their immutable counted archive.
use super::super::{capture_region_members, RelationalColdRegion, RelationalRegionResidency};
#[path = "region_fixture/placement_registration.rs"]
mod placement_registration;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::durability::medium::{
    RelationalDurableMemberCustody, RelationalDurableMemberRetirementObservation,
    RelationalDurableMemberRetirementRefusal, RelationalDurableOwnedReadRefusal,
};
use crate::history::data::BranchId;
use crate::identity::data::KindId;
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::overlay::PartitionContentCommitment;
use crate::tests::support::*;
use crate::transactions::data::{MutationIntent, WorkerIntentBatch};
use placement_registration::{ControlledCustodyOwner, CustodyOwner};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};

#[derive(Debug)]
pub(super) struct CountedSource {
    pub(super) members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    pub(super) reads: AtomicUsize,
    pub(super) history_reads: AtomicUsize,
    pub(super) history: RelationalDurableMemberLocator,
    pub(super) corrupt: AtomicBool,
}

impl RelationalDurableColdSource for CountedSource {
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        if !original.is::<()>() {
            return Err(Refusal::ForeignProtection);
        }
        self.funded_member(request, locator, limit)
    }
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        // This immutable archive has no separate placement registration.
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if original.is::<()>() {
            Ok(())
        } else {
            Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }

    fn advance_member_custody_retirement(
        &self,
        _request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        use crate::durability::medium::RelationalDurableMemberRetirementObservation as Observation;
        match self.validate_member_custody_retirement(original) {
            Ok(()) => Observation::Complete,
            Err(cause) => Observation::Refused(cause),
        }
    }

    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
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
    let execution = &ExecutionRequest::serial(&request);
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
    let (commitment, _) = PartitionContentCommitment::capture(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        &ExecutionRequest::serial(&crate::test_execution_policy::serial_request()),
        partition,
        symbols.as_ref(),
    )
    .unwrap();
    let schema = Arc::new(root.schema_authority().clone());
    let schema_digest = crate::durability::data::DurableBranchRootSchemaImage::capture(&schema)
        .unwrap()
        .carrier_digest();
    let members = capture_region_members(
        &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
        execution,
        partition,
        &commitment,
        &schema,
        schema_digest,
    )
    .unwrap();
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
        directory: members
            .directory
            .portable(
                &crate::storage::substrate::maintenance::ReconstructionLane::oracle(),
                &ExecutionRequest::serial(&crate::test_execution_policy::serial_request()),
            )
            .unwrap(),
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
        live_counts: super::LiveRecordCounts::from_partition(partition),
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
pub(super) struct CustodySource {
    pub(super) source: Arc<CountedSource>,
    pub(super) active: Arc<AtomicUsize>,
    pub(super) refusal: Mutex<Option<Cause>>,
}

impl RelationalDurableColdSource for CustodySource {
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        request
            .check_live()
            .map_err(|cause| Refusal::Request(cause.into()))?;
        let owner = original
            .downcast_ref::<ControlledCustodyOwner>()
            .ok_or(Refusal::ForeignProtection)?;
        if !owner.registered
            || !Arc::ptr_eq(&owner.active, &self.active)
            || owner.source_issuer.as_ptr() != Arc::as_ptr(&self.source)
        {
            return Err(Refusal::ForeignProtection);
        }
        // The provider validates its actual original registration, then reads its immutable archive.
        // No manufactured child token or alternate issuer participates.
        self.source.funded_member(request, locator, limit)
    }
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            if let Some(cause) = *self.refusal.lock().unwrap() {
                return Err(Refusal::Storage(cause));
            }
            let source: Arc<dyn RelationalDurableColdSource> = self.source.clone();
            let child = RelationalDurableMemberCustody::capture(&source, request, members)?;
            // There is no fallible step after this actual registration.
            self.active.fetch_add(1, Ordering::SeqCst);
            Ok(ControlledCustodyOwner {
                source_issuer: Arc::downgrade(&self.source),
                active: self.active.clone(),
                registered: true,
                child: child.into_retirement(),
            })
        })
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        use crate::durability::medium::RelationalDurableMemberRetirementRefusal as Refusal;
        let owner = original
            .downcast_ref::<ControlledCustodyOwner>()
            .ok_or(Refusal::ForeignCustody)?;
        if !Arc::ptr_eq(&owner.active, &self.active)
            || owner.source_issuer.as_ptr() != Arc::as_ptr(&self.source)
        {
            return Err(Refusal::ForeignCustody);
        }
        Ok(())
    }

    fn advance_member_custody_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        use crate::durability::medium::RelationalDurableMemberRetirementObservation as Observation;
        if let Err(cause) = self.validate_member_custody_retirement(original) {
            return Observation::Refused(cause);
        }
        let owner = original
            .downcast_mut::<ControlledCustodyOwner>()
            .expect("validated original registration");
        // self.source is an immutable strong anchor throughout this callback.
        let observation = owner.child.advance(request, self.source.as_ref());
        if matches!(observation, Observation::Complete) && owner.registered {
            owner.active.fetch_sub(1, Ordering::SeqCst);
            owner.registered = false;
        }
        observation
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

pub(super) fn with_memory_request<R>(
    memory: worth_execution::SerialMemoryBudget,
    observe: impl FnOnce(ExecutionRequest<'_, '_>) -> R,
) -> R {
    let host = crate::test_execution_policy::serial_request_with_memory(memory);
    observe(ExecutionRequest::serial(&host))
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
    fn funded_member(
        &self,
        request: &ExecutionRequest<'_, '_>,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        use crate::durability::medium::RelationalDurableOwnedReadRefusal as Refusal;
        let bytes = self.member(locator, limit).map_err(Refusal::Storage)?;
        let mut copy = ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
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
}
