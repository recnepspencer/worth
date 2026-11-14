//! Real native checkpoint inputs and their selected-read/member custody for retirement.
use super::region_fixture::{CountedSource, CustodySource, RegionFixture};
use super::{RelationalColdRegion, RelationalRegionResidency};
use crate::durability::data::{
    PreparedCheckpointDirectory, RelationalCheckpointManifest, RelationalCheckpointMember,
    RelationalNativeCheckpoint,
};
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
    RelationalDurableMemberRetirementObservation as Observation,
    RelationalDurableMemberRetirementRefusal as RetirementRefusal,
    RelationalDurableOwnedReadRefusal as ReadRefusal, RelationalDurableReadPayload,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::storage::substrate::maintenance::ReconstructionLane;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, Weak};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};

pub(super) struct CheckpointFixture {
    pub(super) cell: Arc<RelationalRegionResidency>,
    pub(super) source: Arc<CheckpointSource>,
    pub(super) archive: Arc<CountedSource>,
    pub(super) active: Arc<AtomicUsize>,
}

#[derive(Debug)]
pub(super) struct CheckpointSource {
    native: Arc<RelationalNativeCheckpoint>,
    custody: CustodySource,
}

pub(super) fn readmit(
    fixture: &RegionFixture,
    request: &ExecutionRequest<'_, '_>,
) -> CheckpointFixture {
    let native = Arc::new(
        fixture
            .runtime
            .durability_authority()
            .native_checkpoint(
                request,
                worth_execution::ExecutionAllocationPolicy::SystemAllocation,
            )
            .unwrap(),
    );
    #[derive(serde::Deserialize)]
    struct DirectoryInput {
        manifest: RelationalCheckpointManifest,
    }
    let input: DirectoryInput = rmp_serde::from_slice(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap();
    let cold = &fixture.cold;
    assert!(input
        .manifest
        .roots
        .iter()
        .flat_map(|root| &root.regions)
        .chain(&input.manifest.mirror)
        .any(|region| region.partition_id == cold.partition_id
            && region.head == cold.head
            && region.history == cold.history
            && region.content_digest == cold.content_digest));
    let archive = Arc::new(CountedSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("fresh memory checkpoint must own every member's actual bytes")
                }
            })
            .collect::<BTreeMap<_, _>>(),
        history: cold.history,
        reads: AtomicUsize::new(0),
        history_reads: AtomicUsize::new(0),
        corrupt: AtomicBool::new(false),
    });
    let active = Arc::new(AtomicUsize::new(0));
    let source = Arc::new(CheckpointSource {
        native,
        custody: CustodySource {
            source: archive.clone(),
            active: active.clone(),
            refusal: Mutex::new(None),
        },
    });
    let directory = PreparedCheckpointDirectory {
        history_members: input
            .manifest
            .readmit_history_members(&ReconstructionLane::oracle())
            .unwrap(),
        manifest: input.manifest,
        source: source.clone(),
    }
    .readmit(request)
    .unwrap();
    let mut cold: RelationalColdRegion = fixture.cold.clone();
    cold.source = Some(directory.source.clone());
    cold.history_members = directory.history_members.clone();
    let cell =
        RelationalRegionResidency::from_selected_directory(cold, &directory, request).unwrap();
    // Consume the directory's genuine shares before the cell retires the last member owner.
    let mut retirement = directory.into_retirement();
    while !retirement.advance(request).unwrap() {}
    drop(retirement); // Explicit unbudgeted reclaim of the now inert directory payload.
    assert!(active.load(Ordering::SeqCst) > 0);
    CheckpointFixture {
        cell,
        source,
        archive,
        active,
    }
}

impl RelationalDurableColdSource for CheckpointSource {
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, ReadRefusal> {
        request
            .check_live()
            .map_err(|cause| ReadRefusal::Request(cause.into()))?;
        // The payload owns this actual immutable checkpoint, including its complete member inputs.
        Ok(Box::new(self.native.clone()))
    }

    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, ReadRefusal> {
        if let Some(native) = original.downcast_ref::<Arc<RelationalNativeCheckpoint>>() {
            request
                .check_live()
                .map_err(|cause| ReadRefusal::Request(cause.into()))?;
            if !Arc::ptr_eq(native, &self.native) {
                return Err(ReadRefusal::ForeignProtection);
            }
            if locator.length > limit {
                return Err(ReadRefusal::Storage(Cause::LimitExceeded {
                    length: locator.length,
                    limit,
                }));
            }
            let bytes = native
                .members()
                .iter()
                .find_map(|member| match member {
                    RelationalCheckpointMember::New {
                        locator: found,
                        bytes,
                    } if *found == locator => Some(bytes),
                    _ => None,
                })
                .ok_or(ReadRefusal::Storage(Cause::MissingMember))?;
            self.custody.source.reads.fetch_add(1, Ordering::SeqCst);
            if locator == self.custody.source.history {
                self.custody
                    .source
                    .history_reads
                    .fetch_add(1, Ordering::SeqCst);
            }
            let mut output = ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
                .map_err(ReadRefusal::Buffer)?;
            output
                .extend_from_slice(bytes.bytes())
                .map_err(ReadRefusal::Buffer)?;
            return output.seal_immutable_bytes().map_err(ReadRefusal::Buffer);
        }
        self.custody
            .read_member_owned(request, original, locator, limit)
    }

    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, ReadRefusal> {
        self.custody
            .retain_members_for_request(request, members, issuer)
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RetirementRefusal> {
        if let Some(native) = original.downcast_ref::<Arc<RelationalNativeCheckpoint>>() {
            return if Arc::ptr_eq(native, &self.native) {
                Ok(())
            } else {
                Err(RetirementRefusal::ForeignCustody)
            };
        }
        self.custody.validate_member_custody_retirement(original)
    }

    fn advance_member_custody_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> Observation {
        if original.is::<Arc<RelationalNativeCheckpoint>>() {
            return match self.validate_member_custody_retirement(original) {
                Ok(()) => Observation::Complete, // Immutable bytes have no placement registration.
                Err(cause) => Observation::Refused(cause),
            };
        }
        self.custody
            .advance_member_custody_retirement(request, original)
    }

    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        self.custody.retain_members(members)
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.custody.read_member(locator, limit)
    }

    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.custody.read_commit(position, limit)
    }
}
