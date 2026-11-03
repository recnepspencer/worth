//! Real external checkpoint bytes, with storage-port loss of unread members.
use super::*;
use std::collections::BTreeMap;
use worth_relational::facade::publication::PatchStreamPosition;

#[derive(Debug)]
pub(crate) struct MemberSource {
    pub members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub reads: AtomicUsize,
    pub damaged: std::sync::atomic::AtomicBool,
    pub member_reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub failures: Mutex<BTreeMap<RelationalDurableMemberLocator, Cause>>,
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
impl RelationalDurableColdSource for MemberSource {
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
        self.reads.fetch_add(1, Ordering::Relaxed);
        self.member_reads.lock().unwrap().push(locator);
        if let Some(cause) = self.failures.lock().unwrap().get(&locator) {
            return Err(*cause);
        }
        let mut bytes = bounded(
            self.members.get(&locator).ok_or(Cause::MissingMember)?,
            limit,
        )?;
        if self.damaged.load(Ordering::Relaxed) && bytes.starts_with(b"WORTH-REL-REG") {
            bytes[0] ^= 1;
        }
        Ok(bytes)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        bounded(
            self.commits.get(&position).ok_or(Cause::MissingMember)?,
            limit,
        )
    }
}

pub(crate) fn reopen(
    execution: &ExecutionRequest<'_, '_>,
    producer: &RelationalRuntime,
    schema: RelationalSchemaRegistry,
) -> (RelationalRuntime, Arc<MemberSource>) {
    let checkpoint = producer
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
    let source = Arc::new(MemberSource {
        members: checkpoint
            .members()
            .iter()
            .map(|member| match member {
                RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("first checkpoint owns every member")
                }
            })
            .collect(),
        commits: producer
            .durability()
            .canonical_commit_artifacts(execution.allocation_policy())
            .map(|artifact| {
                let artifact = artifact.unwrap();
                (artifact.position(), artifact.as_bytes().to_vec())
            })
            .collect(),
        reads: AtomicUsize::new(0),
        damaged: std::sync::atomic::AtomicBool::new(false),
        member_reads: Mutex::new(Vec::new()),
        failures: Mutex::new(BTreeMap::new()),
    });
    let recovery = RelationalDurableRecoverySource::new(
        Some(RelationalDurableCheckpointSource::new(
            checkpoint.covered_position().unwrap(),
            checkpoint.covered_branch_transitions().unwrap(),
            checkpoint,
        )),
        vec![],
        vec![],
        source.clone(),
    );
    let mut runtime = RelationalRuntimeApi::builder()
        .schema_registry(schema)
        .build()
        .unwrap();
    let plan = runtime
        .durability()
        .recovery_plan_from_source(
            execution,
            recovery,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    runtime
        .durability_recovery()
        .recover(execution, plan)
        .unwrap();
    assert_eq!(
        source.reads.load(Ordering::Relaxed),
        0,
        "record regions remain genuinely cold after recovery"
    );
    (runtime, source)
}
