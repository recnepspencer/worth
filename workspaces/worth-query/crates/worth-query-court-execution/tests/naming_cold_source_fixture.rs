//! Shared external-member `RelationalDurableColdSource` fixture for Slice 1B
//! cold-directory checkpoint recovery courts.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, Weak};

use worth_execution::{ExecutionImmutableBytes, ExecutionRequest};
use worth_relational::facade::durability::{
    RelationalCheckpointMember, RelationalDurableColdSource, RelationalDurableMemberCustody,
    RelationalDurableMemberLocator, RelationalDurableMemberRetirementObservation,
    RelationalDurableMemberRetirementRefusal, RelationalDurableOwnedReadRefusal,
    RelationalDurableReadPayload, RelationalNativeCheckpoint,
};
use worth_relational::facade::publication::PatchStreamPosition;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;

#[derive(Debug)]
pub struct ExternalColdSource {
    members: BTreeMap<RelationalDurableMemberLocator, ExecutionImmutableBytes>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub member_reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub region_reads: Mutex<usize>,
    pub lineage_reads: Mutex<usize>,
}

impl ExternalColdSource {
    pub fn from_native_capture(
        native: &RelationalNativeCheckpoint,
        producer: &RelationalRuntime,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Arc<Self> {
        let mut members = BTreeMap::new();
        for member in native.members() {
            match member {
                RelationalCheckpointMember::New { locator, bytes } => {
                    members.insert(*locator, bytes.clone());
                }
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("first external checkpoint must emit all members")
                }
            }
        }
        let mut commits = BTreeMap::new();
        for artifact in producer.durability().canonical_commit_artifacts(execution) {
            let artifact = artifact.expect("export canonical commit artifact");
            commits.insert(artifact.position(), artifact.as_bytes().to_vec());
        }
        Arc::new(Self {
            members,
            commits,
            member_reads: Mutex::new(Vec::new()),
            region_reads: Mutex::new(0),
            lineage_reads: Mutex::new(0),
        })
    }

    fn read_checked(
        &self,
        id: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<&ExecutionImmutableBytes, Cause> {
        let bytes = self.members.get(&id).ok_or(Cause::MissingMember)?;
        self.member_reads.lock().unwrap().push(id);
        if bytes.starts_with(b"WORTH-REL-REG") {
            *self.region_reads.lock().unwrap() += 1;
        } else if bytes.starts_with(b"WORTH-REL-MBR1\n") {
            let kind = bytes.get(17).copied();
            if matches!(kind, Some(1 | 3)) {
                *self.lineage_reads.lock().unwrap() += 1;
            }
        }
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes)
    }
}

impl RelationalDurableColdSource for ExternalColdSource {
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
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
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }

    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        Ok(Box::new(()))
    }

    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        id: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        if !original.is::<()>() {
            return Err(RelationalDurableOwnedReadRefusal::ForeignProtection);
        }
        self.read_checked(id, limit)
            .cloned()
            .map_err(RelationalDurableOwnedReadRefusal::Storage)
    }

    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }

    fn read_member(&self, id: RelationalDurableMemberLocator, limit: u64) -> Result<Vec<u8>, Cause> {
        self.read_checked(id, limit).map(|bytes| bytes.to_vec())
    }

    fn read_commit(&self, id: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        let bytes = self.commits.get(&id).ok_or(Cause::MissingMember)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.clone())
    }
}
