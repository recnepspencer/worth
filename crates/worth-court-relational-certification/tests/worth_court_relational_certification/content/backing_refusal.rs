//! Real cold checkpoint ingress preserves physical cause and the funded read bound.
use super::{original, recovery::publish, transaction::fixture};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use worth_execution::{CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::{
    content::RelationalContentDenial as Denial, durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug)]
struct Source {
    members: BTreeMap<RelationalDurableMemberLocator, Vec<u8>>,
    commits: BTreeMap<PatchStreamPosition, Vec<u8>>,
    content: RelationalDurableMemberLocator,
    cause: Cause,
    limits: Mutex<Vec<u64>>,
}
impl RelationalDurableColdSource for Source {
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
        if locator == self.content {
            self.limits.lock().unwrap().push(limit);
            return Err(self.cause);
        }
        let bytes = self.members.get(&locator).ok_or(Cause::MissingMember)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.clone())
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        let bytes = self
            .commits
            .get(&position)
            .ok_or(Cause::SourceUnavailable)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.clone())
    }
}

#[test]
fn content_cold_ingress_preserves_physical_refusals_and_uses_exact_funded_limit() {
    let cancel = CancellationSource::new();
    let request = original(&cancel);
    let execution = ExecutionRequest::serial(&request);
    let (runtime, contract) = fixture();
    publish(&runtime, &execution, &contract);
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            &execution,
            execution.allocation_policy(),
            RelationalCheckpointArtifactPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .unwrap();
    let members = checkpoint
        .members()
        .iter()
        .map(|member| match member {
            RelationalCheckpointMember::New { locator, bytes } => (*locator, bytes.to_vec()),
            _ => panic!("first capture owns bytes"),
        })
        .collect::<BTreeMap<_, _>>();
    let content = *members
        .iter()
        .find(|(_, bytes)| bytes.starts_with(b"WRTHCONT"))
        .unwrap()
        .0;
    assert_eq!(content.length, 75);
    let commits = runtime
        .durability()
        .canonical_commit_artifacts(execution.allocation_policy())
        .map(|artifact| {
            let artifact = artifact.unwrap();
            (artifact.position(), artifact.as_bytes().to_vec())
        })
        .collect::<BTreeMap<_, _>>();
    for cause in [
        Cause::MissingMember,
        Cause::SourceUnavailable,
        Cause::Damaged,
        Cause::MemoryExhausted,
        Cause::LimitExceeded {
            length: 1000,
            limit: 75,
        },
    ] {
        let source = Arc::new(Source {
            members: members.clone(),
            commits: commits.clone(),
            content,
            cause,
            limits: Mutex::new(Vec::new()),
        });
        let budget = SerialMemoryBudget::new(Some(512 * 1024 * 1024));
        let receiving = SerialRequest::from_memory(budget.clone(), cancel.token(), None);
        let receiving = ExecutionRequest::serial(&receiving);
        let (mut target, _) = fixture();
        let before = target
            .observe_branch(&target.main_branch_identity())
            .unwrap()
            .1;
        let recovery = RelationalDurableRecoverySource::new(
            Some(RelationalDurableCheckpointSource::new(
                checkpoint.covered_position().unwrap(),
                checkpoint.covered_branch_transitions().unwrap(),
                RelationalNativeCheckpoint::from_untrusted_bytes(checkpoint.bytes().to_vec()),
            )),
            vec![],
            vec![],
            source.clone(),
        );
        let plan = target
            .durability()
            .recovery_plan_from_source(
                &receiving,
                recovery,
                RecoveryVerificationMode::NormalRecoveryVerification,
            )
            .unwrap();
        let refusal = target
            .durability_recovery()
            .recover(&receiving, plan)
            .unwrap_err();
        match cause {
            Cause::MissingMember => assert!(
                matches!(
                    refusal,
                    RelationalRecoveryStop::Content(Denial::MissingContent)
                ),
                "{refusal:?}"
            ),
            cause => assert!(
                matches!(refusal, RelationalRecoveryStop::Content(Denial::BackingUnavailable { cause: actual, .. }) if actual == cause),
                "{refusal:?}"
            ),
        }
        assert_eq!(*source.limits.lock().unwrap(), [75]);
        assert_eq!(budget.charged_memory_bytes(), 0);
        assert_eq!(
            target
                .observe_branch(&target.main_branch_identity())
                .unwrap()
                .1
                .descriptor(),
            before.descriptor()
        );
    }
}
