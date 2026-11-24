//! Producer-written region members and their immutable counted archive.
use worth_relational_contracts::internal_port::indexes::data::entry_map::DerivedIndexEntryMapOwnerPort as _;
use worth_relational_contracts::internal_port::transactions::data::outcomes::transaction_commit_error::TransactionCommitErrorOwnerPort as _;
use worth_relational_storage::internal_port::durability::medium::member_custody::RelationalDurableMemberCustodyOwnerPort as _;
use super::super::{RelationalColdRegion, RelationalRegionResidency};
#[path = "region_fixture/placement_registration.rs"]
mod placement_registration;
pub(super) use super::checkpoint_inputs::CountedSource;
use crate::durability::medium::{RelationalDurableColdSource, RelationalDurableMemberLocator};
use crate::durability::medium::{
    RelationalDurableMemberCustody, RelationalDurableMemberRetirementObservation,
    RelationalDurableMemberRetirementRefusal, RelationalDurableOwnedReadRefusal,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use placement_registration::{ControlledCustodyOwner, CustodyOwner};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};

use crate::durability::native_region_fixture;

pub(super) struct RegionFixture {
    pub(super) cell: Arc<RelationalRegionResidency>,
    pub(super) cold: RelationalColdRegion,
    pub(super) source: Arc<CountedSource>,
    pub(super) entity: crate::identity::data::EntityId,
    pub(super) relation: crate::identity::data::RelationId,
    pub(super) expected_digest: [u8; 32],
    pub(super) expected_history: usize,
    pub(super) partition: crate::storage::overlay::PartitionState,
}

pub(super) fn region_with_history() -> RegionFixture {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let native_region_fixture::HistoryPartition {
            partition,
            entity,
            relation,
            schema,
            symbols,
        } = native_region_fixture::populated_partition();
        let host = worth_harness::facade::execution_fixture::serial_request();
        let execution = &ExecutionRequest::serial(&host);
        let lane = fixture_lane;
        let expected_history = partition
            .entity_arena
            .metadata_history_at(entity.slot_index())
            .unwrap()
            .len();
        assert!(
            expected_history > 1,
            "fixture must have real committed history"
        );
        let (commitment, _) = crate::storage::overlay::PartitionContentCommitment::capture(
            &lane, execution, &partition, &symbols,
        )
        .unwrap();
        let expected_digest = commitment.digest(partition.partition_id);
        let schema_digest = crate::durability::data::DurableBranchRootSchemaImage::capture(&schema)
            .unwrap()
            .carrier_digest();
        let codec = &crate::storage::residency::region_load::CHECKPOINT_REGION_READMISSION;
        let members = (codec.capture_members)(
            &lane,
            execution,
            &partition,
            &commitment,
            schema.as_ref(),
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
        let symbol_authority =
            crate::runtime::state::subsystems::services::symbol_table::RuntimeSymbolTable::default(
            );
        symbol_authority.replace(symbols);
        let cold = RelationalColdRegion {
            readmission: codec,
            directory: members.directory.portable(&lane, execution).unwrap(),
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
            live_counts: super::LiveRecordCounts::from_partition(&partition),
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
            partition,
        }
    })
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
    let host = worth_harness::facade::execution_fixture::serial_request_with_memory(memory);
    observe(ExecutionRequest::serial(&host))
}

use crate::durability::data::{CapturedCheckpointMembers, RelationalCheckpointManifest};
use crate::indexes::data::DerivedIndexArtifacts;
use crate::storage::data::checkpoint_member::RelationalCheckpointMember;
use crate::storage::data::member_frame::{self, MemberKind};
use crate::storage::residency::PreparedResidentRegion;
use crate::storage::substrate::maintenance::ReconstructionLane;

#[derive(Debug)]
pub(super) struct SelectedRegionArchive {
    pub(crate) manifest: RelationalCheckpointManifest,
    pub(crate) members: CapturedCheckpointMembers,
}

pub(super) fn selected_region_archive(
    partition: &crate::storage::overlay::PartitionState,
    schema: &dyn crate::storage::residency::RelationalRegionSchema,
    schema_digest: [u8; 32],
    symbols: &crate::symbols::data::StringInterner,
    request: &ExecutionRequest<'_, '_>,
) -> SelectedRegionArchive {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let prepared =
            PreparedResidentRegion::initial(fixture_lane, request, partition.clone(), symbols)
                .unwrap();
        let captured = prepared
            .retention
            .cell()
            .capture_complete_for_request(request, schema, schema_digest, symbols)
            .unwrap();
        let (derived_indexes, indexes) = member_frame::encode(
            MemberKind::DerivedIndex,
            &(DerivedIndexArtifacts::default(), Option::<()>::None, 0_u16),
        )
        .unwrap();
        let mut members = Vec::from(captured.members);
        members.push(RelationalCheckpointMember::New {
            locator: derived_indexes,
            bytes: ExecutionImmutableBytes::from_external_bytes(Arc::new(
                indexes.into_boxed_slice(),
            )),
        });
        let manifest = crate::durability::data::selected_region_archive::selected_region_manifest(
            captured.leaf,
            schema_digest,
            derived_indexes,
        );
        manifest.assert_original_flat_wire();
        SelectedRegionArchive {
            manifest,
            members: CapturedCheckpointMembers::capture(request, members.len(), members).unwrap(),
        }
    })
}
