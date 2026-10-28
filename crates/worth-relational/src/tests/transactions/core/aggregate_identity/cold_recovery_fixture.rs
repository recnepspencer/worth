//! Real external native checkpoint input for aggregate cold-read integration.
use super::fixtures::*;
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
    members: BTreeMap<RelationalDurableMemberLocator, Arc<[u8]>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub(super) fail_regions: AtomicBool,
    pub(super) reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) histories: BTreeSet<RelationalDurableMemberLocator>,
}
impl RelationalDurableColdSource for ColdSource {
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
        self.reads.lock().unwrap().push(id);
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
        Ok(bytes.to_vec())
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
    let artifacts = attached_test_medium(producer).offered();
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
    let attachment = TestDurableMedium::new().attachment();
    for artifact in artifacts {
        assert!(matches!(
            attachment.publish(&artifact),
            RelationalDurablePublication::Committed
        ));
    }
    let mut receiving = RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .build_durable(attachment)
        .unwrap();
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
    (receiving, source)
}
