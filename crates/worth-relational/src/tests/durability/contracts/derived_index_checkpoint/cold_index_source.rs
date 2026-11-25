use super::*;
use crate::durability::data::{
    RelationalCheckpointArtifactPolicy as Policy, RelationalCheckpointMember as Member,
};
use crate::durability::medium::{
    RelationalDurableCheckpointSource as CheckpointSource,
    RelationalDurableColdSource as ColdSource, RelationalDurableMemberCustody as MemberCustody,
    RelationalDurableMemberLocator as MemberLocator,
    RelationalDurableOwnedReadRefusal as OwnedRefusal, RelationalDurableReadPayload as ReadPayload,
    RelationalDurableRecoverySource as RecoverySource,
};
use crate::storage::data::RelationalStorageUnavailableCause as StorageCause;

pub(super) fn reopen_cold_index_members(
    execution: &Req<'_, '_>,
    producer: &Rt,
) -> (Rt, std::sync::Arc<IndexColdSource>) {
    let native = producer
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            Policy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let members = native
        .members()
        .iter()
        .map(|m| match m {
            Member::New { locator, bytes } => (*locator, bytes.clone()),
            Member::Carried { .. } => unreachable!(),
        })
        .collect();
    let source = std::sync::Arc::new(IndexColdSource(members, Default::default()));
    let recovery = RecoverySource::new(
        Some(CheckpointSource::new(
            native.covered_position().unwrap(),
            native.covered_branch_transitions().unwrap(),
            native,
        )),
        Vec::new(),
        Vec::new(),
        source.clone(),
    );
    let mut runtime =
        runtime_with_declared_aspect_schema(CascadeDeletePolicy::CascadeDeleteRelations);
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
    (runtime, source)
}

#[derive(Debug)]
pub(super) struct IndexColdSource(
    std::collections::BTreeMap<MemberLocator, worth_execution::ExecutionImmutableBytes>,
    pub(super) std::sync::atomic::AtomicUsize,
);

impl ColdSource for IndexColdSource {
    fn retain_members_for_request(
        &self,
        request: &Req<'_, '_>,
        _: &[MemberLocator],
        issuer: std::sync::Weak<dyn ColdSource>,
    ) -> Result<MemberCustody, OwnedRefusal> {
        MemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &Req<'_, '_>,
    ) -> Result<ReadPayload, OwnedRefusal> {
        request
            .check_live()
            .map_err(|c| OwnedRefusal::Request(c.into()))?;
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &Req<'_, '_>,
        _: &(dyn std::any::Any + Send + Sync),
        locator: MemberLocator,
        _: u64,
    ) -> Result<worth_execution::ExecutionImmutableBytes, OwnedRefusal> {
        request
            .check_live()
            .map_err(|c| OwnedRefusal::Request(c.into()))?;
        let bytes = self
            .0
            .get(&locator)
            .cloned()
            .ok_or(OwnedRefusal::Storage(StorageCause::MissingMember))?;
        if crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact::from_untrusted_bytes(bytes.as_ref()).is_err() {
            self.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Ok(bytes)
    }
    fn retain_members(&self, _: &[MemberLocator]) -> Result<MemberCustody, StorageCause> {
        Ok(MemberCustody::new(()))
    }
    fn read_member(&self, locator: MemberLocator, _: u64) -> Result<Vec<u8>, StorageCause> {
        self.1.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        self.0
            .get(&locator)
            .map(|b| b.to_vec())
            .ok_or(StorageCause::MissingMember)
    }
    fn read_commit(
        &self,
        _: crate::publication::patch::data::PatchStreamPosition,
        _: u64,
    ) -> Result<Vec<u8>, StorageCause> {
        Err(StorageCause::MissingMember)
    }
}
