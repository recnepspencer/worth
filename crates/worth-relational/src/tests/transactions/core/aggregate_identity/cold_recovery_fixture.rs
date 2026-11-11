//! Real external native checkpoint input for aggregate cold-read integration.
use crate::durability::{data::*, medium::*};
use crate::publication::patch::data::PatchStreamPosition;
use crate::storage::data::RelationalStorageUnavailableCause as Cause;
use crate::tests::support::*;
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
};

#[derive(Debug)]
pub(super) struct ColdSource {
    members: BTreeMap<RelationalDurableMemberLocator, worth_execution::ExecutionImmutableBytes>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    canonical_members: BTreeSet<RelationalDurableMemberLocator>,
    canonical_reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    journal_reads: std::sync::atomic::AtomicUsize,
    pub(super) fail_regions: AtomicBool,
    pub(super) reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) histories: BTreeSet<RelationalDurableMemberLocator>,
}
impl RelationalDurableColdSource for ColdSource {
    fn retain_members_for_request(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        // This source's immutable map retains the members for its whole lifetime.
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
        _request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        // Every member belongs to this immutable source for its Arc lifetime.
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        id: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        if !original.is::<()>() {
            return Err(RelationalDurableOwnedReadRefusal::ForeignProtection);
        }
        self.member(id, limit)
            .cloned()
            .map_err(RelationalDurableOwnedReadRefusal::Storage)
    }
    fn retain_members(
        &self,
        _: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        id: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.member(id, limit).map(|bytes| bytes.to_vec())
    }
    fn read_commit(&self, id: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.journal_reads.fetch_add(1, Ordering::Relaxed);
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
impl ColdSource {
    fn member(
        &self,
        id: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<&worth_execution::ExecutionImmutableBytes, Cause> {
        if self.canonical_members.contains(&id) {
            self.canonical_reads.lock().unwrap().push(id);
        } else {
            self.reads.lock().unwrap().push(id);
        }
        let bytes = self.members.get(&id).ok_or(Cause::MissingMember)?;
        if self.fail_regions.load(Ordering::Relaxed) && bytes.starts_with(b"WORTH-REL-REG") {
            return Err(Cause::SourceUnavailable);
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
pub(super) fn reopen(
    execution: &worth_execution::ExecutionRequest<'_, '_>,
    producer: &RelationalRuntime,
) -> (RelationalRuntime, Arc<ColdSource>) {
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    #[derive(serde::Deserialize)]
    struct Input {
        manifest: RelationalCheckpointManifest,
    }
    let manifest = rmp_serde::from_slice::<Input>(
        native
            .bytes()
            .strip_prefix(crate::durability::log::native_checkpoint_codec::CHECKPOINT_MAGIC)
            .unwrap(),
    )
    .unwrap()
    .manifest;
    let artifacts = producer
        .history
        .commit_artifacts()
        .iter()
        .map(|entry| {
            entry
                .envelope_residency()
                .require_original_artifact(execution, None)
                .unwrap()
        })
        .collect::<Vec<_>>();
    let source = Arc::new(ColdSource {
        members: native
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.clone()),
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("first capture owns every member")
                }
            })
            .collect(),
        commits: artifacts
            .iter()
            .map(|artifact| (artifact.position(), artifact.as_bytes().to_vec()))
            .collect(),
        canonical_members: artifacts
            .iter()
            .map(|artifact| {
                crate::durability::log::native_checkpoint_codec::member_frame::locator(
                    artifact.as_bytes(),
                )
            })
            .collect(),
        canonical_reads: Default::default(),
        journal_reads: Default::default(),
        fail_regions: AtomicBool::new(false),
        reads: Mutex::new(Vec::new()),
        histories: manifest
            .mirror
            .iter()
            .chain(manifest.roots.iter().flat_map(|root| &root.regions))
            .map(|region| region.history)
            .collect(),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut receiving = super::fixtures::runtime();
    let plan = receiving
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    receiving
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert!(
        source.reads.lock().unwrap().is_empty(),
        "reopen keeps record/history members cold"
    );
    assert_eq!(source.journal_reads.load(Ordering::Relaxed), 0);
    assert!(
        (1..4).contains(&source.canonical_reads.lock().unwrap().len()),
        "only actual distinct head envelopes are read"
    );
    (receiving, source)
}
