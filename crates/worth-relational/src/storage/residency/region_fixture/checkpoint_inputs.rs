//! Real native checkpoint inputs and their selected-read/member custody for retirement.
use super::region_fixture::{
    selected_region_archive, CustodySource, RegionFixture, SelectedRegionArchive,
};
use super::{RelationalColdRegion, RelationalRegionResidency};
use crate::durability::data::PreparedCheckpointDirectory;
use crate::durability::medium::{
    RelationalDurableColdSource, RelationalDurableMemberCustody, RelationalDurableMemberLocator,
    RelationalDurableMemberRetirementObservation, RelationalDurableMemberRetirementRefusal,
    RelationalDurableOwnedReadRefusal, RelationalDurableReadPayload,
};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::checkpoint_member::RelationalCheckpointMember;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
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
    native: Arc<SelectedRegionArchive>,
    custody: CustodySource,
}

pub(super) fn readmit(
    fixture: &RegionFixture,
    request: &ExecutionRequest<'_, '_>,
) -> CheckpointFixture {
    crate::branch::root::capture::with_reconstruction_fixture(|fixture_lane| {
        let native = Arc::new(selected_region_archive(
            &fixture.partition,
            fixture.cold.schema.as_ref(),
            fixture.cold.schema_carrier_digest,
            &fixture.cold.symbols.interner_snapshot(),
            request,
        ));
        let cold = &fixture.cold;
        assert!(native
            .manifest
            .roots
            .iter()
            .flat_map(|root| &root.regions)
            .chain(&native.manifest.mirror)
            .any(|region| region.partition_id == cold.partition_id
                && region.head == cold.head
                && region.history == cold.history
                && region.content_digest == cold.content_digest));
        let archive = Arc::new(CountedSource {
            members: native
                .members
                .entries()
                .iter()
                .map(|member| match member {
                    RelationalCheckpointMember::New { locator, bytes } => {
                        (*locator, bytes.to_vec())
                    }
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
            native: native.clone(),
            custody: CustodySource {
                source: archive.clone(),
                active: active.clone(),
                refusal: Mutex::new(None),
            },
        });
        let directory = PreparedCheckpointDirectory {
            history_members: native
                .manifest
                .readmit_history_members(fixture_lane)
                .unwrap(),
            manifest: native.manifest.clone(),
            source: source.clone(),
        }
        .readmit(request)
        .unwrap();
        let mut cold: RelationalColdRegion = fixture.cold.clone();
        cold.source = Some(directory.source.clone());
        cold.history_members = directory.history_members.clone();
        let cell = RelationalRegionResidency::from_selected_directory(
            cold,
            directory.region_member_basis(),
            &directory.history_members,
            request,
        )
        .unwrap();
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
    })
}

impl RelationalDurableColdSource for CheckpointSource {
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        // The payload owns this actual immutable checkpoint, including its complete member inputs.
        Ok(Box::new(self.native.clone()))
    }

    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        if let Some(native) = original.downcast_ref::<Arc<SelectedRegionArchive>>() {
            request
                .check_live()
                .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
            if !Arc::ptr_eq(native, &self.native) {
                return Err(RelationalDurableOwnedReadRefusal::ForeignProtection);
            }
            if locator.length > limit {
                return Err(RelationalDurableOwnedReadRefusal::Storage(
                    Cause::LimitExceeded {
                        length: locator.length,
                        limit,
                    },
                ));
            }
            let bytes = native
                .members
                .entries()
                .iter()
                .find_map(|member| match member {
                    RelationalCheckpointMember::New {
                        locator: found,
                        bytes,
                    } if *found == locator => Some(bytes),
                    _ => None,
                })
                .ok_or(RelationalDurableOwnedReadRefusal::Storage(
                    Cause::MissingMember,
                ))?;
            self.custody.source.reads.fetch_add(1, Ordering::SeqCst);
            if locator == self.custody.source.history {
                self.custody
                    .source
                    .history_reads
                    .fetch_add(1, Ordering::SeqCst);
            }
            let mut output = ExecutionRequestArrayBuilder::allocate(*request, bytes.len())
                .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
            output
                .extend_from_slice(bytes.bytes())
                .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
            return output
                .seal_immutable_bytes()
                .map_err(RelationalDurableOwnedReadRefusal::Buffer);
        }
        self.custody
            .read_member_owned(request, original, locator, limit)
    }

    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        self.custody
            .retain_members_for_request(request, members, issuer)
    }

    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if let Some(native) = original.downcast_ref::<Arc<SelectedRegionArchive>>() {
            return if Arc::ptr_eq(native, &self.native) {
                Ok(())
            } else {
                Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
            };
        }
        self.custody.validate_member_custody_retirement(original)
    }

    fn advance_member_custody_retirement(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        if original.is::<Arc<SelectedRegionArchive>>() {
            return match self.validate_member_custody_retirement(original) {
                Ok(()) => RelationalDurableMemberRetirementObservation::Complete, // Immutable bytes have no placement registration.
                Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
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

#[derive(Debug)]
pub(super) struct CountedSource {
    pub(super) members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    pub(super) reads: AtomicUsize,
    pub(super) history_reads: AtomicUsize,
    pub(super) history: RelationalDurableMemberLocator,
    pub(super) corrupt: AtomicBool,
}

impl RelationalDurableColdSource for CountedSource {
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        crate::durability::medium::RelationalDurableReadPayload,
        RelationalDurableOwnedReadRefusal,
    > {
        request
            .check_live()
            .map_err(|c| RelationalDurableOwnedReadRefusal::Request(c.into()))?;
        Ok(Box::new(()))
    }
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
        use crate::durability::medium::RelationalDurableMemberRetirementObservation;
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
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

impl CountedSource {
    pub(super) fn member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<&[u8], Cause> {
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
    pub(super) fn funded_member(
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
