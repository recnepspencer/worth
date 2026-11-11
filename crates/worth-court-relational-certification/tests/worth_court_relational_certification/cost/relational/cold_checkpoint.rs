//! Immutable checkpoint placement with observable real member reads.
use std::{
    collections::BTreeMap,
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc, Mutex,
    },
};
use worth_execution::ExecutionRequest;
use worth_relational::facade::{
    durability::*, publication::PatchStreamPosition, runtime::RelationalRuntime,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug)]
pub(super) struct ColdCheckpointSource {
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub(super) bytes: AtomicU64,
    pub(super) reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) unavailable: AtomicBool,
    pub(super) damaged: AtomicBool,
}
impl ColdCheckpointSource {
    pub(super) fn capture(
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
        checkpoint: &RelationalNativeCheckpoint,
    ) -> Arc<Self> {
        Arc::new(Self {
            members: checkpoint
                .members()
                .iter()
                .map(|member| match member {
                    RelationalCheckpointMember::New { locator, bytes } => {
                        (*locator, bytes.to_vec())
                    }
                    RelationalCheckpointMember::Carried { .. } => {
                        panic!("first checkpoint owns its members")
                    }
                })
                .collect(),
            commits: runtime
                .durability()
                .canonical_commit_artifacts(request)
                .map(|artifact| {
                    let a = artifact.unwrap();
                    (a.position(), a.as_bytes().to_vec())
                })
                .collect(),
            bytes: AtomicU64::new(0),
            reads: Mutex::new(Vec::new()),
            unavailable: AtomicBool::new(false),
            damaged: AtomicBool::new(false),
        })
    }
}
impl RelationalDurableColdSource for ColdCheckpointSource {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        if members
            .iter()
            .any(|member| !self.members.contains_key(member))
        {
            return Err(Cause::MissingMember);
        }
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, Cause> {
        self.reads.lock().unwrap().push(locator);
        if self.damaged.load(Ordering::SeqCst) {
            return Err(Cause::Damaged);
        }
        if self.unavailable.load(Ordering::SeqCst) {
            return Err(Cause::SourceUnavailable);
        }
        let bytes = bounded(
            self.members.get(&locator).ok_or(Cause::MissingMember)?,
            limit,
        )?;
        self.bytes.fetch_add(bytes.len() as u64, Ordering::SeqCst);
        Ok(bytes)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        bounded(
            self.commits
                .get(&position)
                .ok_or(Cause::SourceUnavailable)?,
            limit,
        )
    }
}
fn bounded(bytes: &[u8], limit: u64) -> Result<Vec<u8>, Cause> {
    if bytes.len() as u64 > limit {
        return Err(Cause::LimitExceeded {
            length: bytes.len() as u64,
            limit,
        });
    }
    Ok(bytes.to_vec())
}

pub(super) fn reopen(
    request: &ExecutionRequest<'_, '_>,
    checkpoint: &RelationalNativeCheckpoint,
    source: Arc<ColdCheckpointSource>,
    mut runtime: RelationalRuntime,
) -> RelationalRuntime {
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
        )),
        vec![],
        vec![],
        source,
    );
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            request,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(request, plan)
        .unwrap();
    runtime
}
