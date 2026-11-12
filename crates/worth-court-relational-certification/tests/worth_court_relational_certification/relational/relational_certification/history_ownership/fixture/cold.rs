//! Real external checkpoint bytes, with storage-port loss of unread members.
use super::*;
use std::collections::BTreeMap;
use worth_relational::facade::publication::PatchStreamPosition;
mod owned_reads;
use owned_reads::{failure, snapshot, Snapshot};

#[derive(Debug)]
pub(crate) struct MemberSource {
    pub members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    original_members: worth_testkit_relational::durable_medium::bytes::MemberBytes,
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
impl MemberSource {
    /// Record-region demands only; catalog/symbol/checkpoint metadata remains in total reads.
    pub(in super::super) fn region_read_count(&self) -> usize {
        self.member_reads
            .lock()
            .unwrap()
            .iter()
            .filter(|locator| {
                self.members
                    .get(*locator)
                    .is_some_and(|bytes| bytes.starts_with(b"WORTH-REL-REG"))
            })
            .count()
    }
}

impl RelationalDurableColdSource for MemberSource {
    fn checkpoint_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (RelationalDurableReadBounds, RelationalDurableReadPayload),
        RelationalDurableOwnedReadRefusal,
    > {
        let original = self.snapshot(request)?;
        let [canonical, transitions] = original.bounds();
        Ok((
            RelationalDurableReadBounds {
                selected_checkpoint: true,
                canonical: PatchStreamPosition(canonical),
                transitions: worth_relational::facade::branch::RelationalBranchTransitionSequence(
                    transitions,
                ),
            },
            original,
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        self.snapshot(request)
            .map(|original| original as RelationalDurableReadPayload)
    }
    fn read_initial_inputs_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        limit: u64,
    ) -> Result<Option<worth_execution::ExecutionImmutableBytes>, RelationalDurableOwnedReadRefusal>
    {
        snapshot(original)?
            .initial_inputs(request, limit)
            .map_err(failure)
    }
    fn read_member_prefix_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        member: RelationalDurableMemberLocator,
        length: usize,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        snapshot(original)?
            .member_prefix(request, member.sha256, member.length, length)
            .map_err(failure)
    }
    fn read_commit_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        snapshot(original)?
            .canonical(request, position.0, limit)
            .map_err(failure)
    }
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        self.read_owned_member(request, snapshot(original)?, locator, limit)
    }
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        _: &[RelationalDurableMemberLocator],
        issuer: std::sync::Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || {
            self.snapshot(request).map(|original| *original)
        })
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        if original.is::<Snapshot>() {
            Ok(())
        } else {
            Err(RelationalDurableMemberRetirementRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
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
    let mut original_members =
        worth_testkit_relational::durable_medium::bytes::MemberBytes::default();
    for member in checkpoint.members() {
        match member {
            RelationalCheckpointMember::New { locator, bytes } => {
                original_members.insert(locator.sha256, locator.length, bytes.clone());
            }
            RelationalCheckpointMember::Carried { .. } => {
                panic!("first checkpoint owns every member")
            }
        }
    }
    let source = Arc::new(MemberSource {
        original_members,
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
            .canonical_commit_artifacts(execution)
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
        source.region_read_count(),
        0,
        "record regions remain genuinely cold after recovery"
    );
    (runtime, source)
}
