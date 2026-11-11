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

mod owned_reads;
use owned_reads::{failure, snapshot, Snapshot};

#[derive(Debug)]
pub(super) struct ColdCheckpointSource {
    original_members: worth_testkit_relational::durable_medium::bytes::MemberBytes,
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    pub(super) bytes: AtomicU64,
    pub(super) reads: Mutex<Vec<RelationalDurableMemberLocator>>,
    pub(super) unavailable: AtomicBool,
    pub(super) damaged: AtomicBool,
}
impl ColdCheckpointSource {
    /// Record-region demands only; catalog/symbol/checkpoint metadata remains in total reads.
    pub(super) fn region_read_count(&self) -> usize {
        self.reads
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

    pub(super) fn capture(
        runtime: &RelationalRuntime,
        request: &ExecutionRequest<'_, '_>,
        checkpoint: &RelationalNativeCheckpoint,
    ) -> Arc<Self> {
        let mut original_members =
            worth_testkit_relational::durable_medium::bytes::MemberBytes::default();
        for member in checkpoint.members() {
            match member {
                RelationalCheckpointMember::New { locator, bytes } => {
                    original_members.insert(locator.sha256, locator.length, bytes.clone());
                }
                RelationalCheckpointMember::Carried { .. } => {
                    panic!("first checkpoint owns its members")
                }
            }
        }
        Arc::new(Self {
            original_members,
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
