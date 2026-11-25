use std::sync::Arc;

use sha2::{Digest, Sha256};
use worth_execution::{ExecutionRequest as Req, ExecutionRequestArrayBuilder as ReqArrayBuilder};

use super::*;
use crate::branch::RelationalBranchTransitionSequence as Seq;
use crate::durability::branch_transition_artifact::RelationalBranchTransitionArtifact as TransArt;
use crate::durability::canonical_commit_artifact::RelationalCanonicalCommitArtifact as CommitArt;
use crate::durability::data::{
    RecoveryOutcome, RecoveryVerificationMode as RecoveryMode,
    RelationalCheckpointArtifactPolicy as CheckpointPolicy,
    RelationalNativeCheckpoint as NativeCheckpoint,
};
use crate::durability::medium::{
    CheckpointSelectionIdentity, CheckpointSelectionObservation as SelectionObs,
    DurablePublicationFate as Fate, OpaqueCapturedCheckpointSubject as CheckpointSubject,
    RelationalDurableColdSource as ColdSource, RelationalDurableMedium as DurableMedium,
    RelationalDurableMemberCustody as MemberCustody, RelationalDurableMemberLocator as MemberLoc,
    RelationalDurableMemberRetirementObservation as RetireObs,
    RelationalDurableMemberRetirementRefusal as RetireRefusal,
    RelationalDurableOwnedReadRefusal as ReadRefusal, RelationalDurableReadBounds as ReadBounds,
    RelationalDurableReadPayload as ReadPayload,
    RelationalDurableTransitionReadFailure as TransReadErr,
};
use crate::durability::source_initial_inputs::{
    RelationalInitialInputsPublicationDetail as InitDetail,
    RelationalSourceInitialInputsArtifact as InitArt,
};
use crate::storage::data::RelationalStorageUnavailableCause as StorageCause;

#[path = "durability/canonical_commit_frame.rs"]
mod canonical_commit_frame;
#[path = "durability/captured_members.rs"]
mod captured_members;
#[path = "durability/owned_medium.rs"]
mod owned_medium;
#[path = "durability/owned_medium_cold_source.rs"]
mod owned_medium_cold_source;
#[path = "durability/owned_medium_tamper.rs"]
mod owned_medium_tamper;
pub(crate) use canonical_commit_frame::reseal_canonical_commit_frame;
pub(crate) use captured_members::{
    captured_members_source, captured_members_source_with_fallback, empty_cold_source,
    encode_checkpoint_with_source, expect_durability_failure, CapturedMembersColdSource,
};
use owned_medium::OwnedMediumAttachment;
pub(crate) use owned_medium::{
    OwnedTestDurableMedium, ScriptedPublication, TestRecoverySourceRefusal,
};

pub(crate) fn capture_checkpoint_with_source(
    execution: &Req<'_, '_>,
    runtime: &RelationalRuntime,
    policy: worth_execution::ExecutionAllocationPolicy<'_, '_>,
) -> Result<
    (NativeCheckpoint, Arc<dyn ColdSource>),
    crate::durability::data::RelationalNativeCheckpointCaptureDenial,
> {
    let prior = runtime.durability.checkpoint_selection().snapshot();
    let mut current_members = Vec::new();
    let prior_source = if let Some(directory) = &prior {
        if let Ok(semantic) = directory.manifest.semantic_members(execution) {
            current_members.extend(semantic.iter().copied());
        }
        for root in directory.manifest.roots.iter() {
            for content in &root.content {
                current_members.push(content.member);
            }
        }
        Some(directory.source.clone())
    } else {
        None
    };
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            policy,
            CheckpointPolicy::ExternalMembers {
                current_members: &current_members,
                source: prior_source.clone(),
            },
        )?;
    let source = captured_members_source_with_fallback(&checkpoint, prior_source);
    Ok((checkpoint, source))
}

/// Fork `branch_name` from main on a serial request of its own, as a test
/// fixture step whose request is irrelevant to the behavior under test.
pub(crate) fn create_branch_from_main(runtime: &RelationalRuntime, branch_name: &str) -> BranchId {
    let host_request = crate::test_execution_policy::serial_request();
    let execution = &Req::serial(&host_request);
    let branch_id = BranchId(branch_name.to_string());
    runtime
        .history_authority()
        .fork_branch_from(execution, branch_id.clone(), &BranchId("main".to_string()))
        .unwrap();
    branch_id
}

/// The retained bill of a public immutable byte owner holding `bytes`,
/// measured after its original controller scope is gone: payload plus control.
pub(crate) fn retained_byte_owner_bill(bytes: &[u8]) -> u64 {
    let budget = worth_execution::SerialMemoryBudget::new(Some(
        (bytes.len() as u64).checked_add(64 * 1024).unwrap(),
    ));
    let host = crate::test_execution_policy::serial_request_with_memory(budget.clone());
    let request = Req::serial(&host);
    let mut building =
        worth_execution::ExecutionRequestArrayBuilder::<u8>::allocate(request, bytes.len())
            .unwrap();
    building.extend_from_slice(bytes).unwrap();
    let owned = building.seal_immutable_bytes().unwrap();
    assert_eq!(owned.charged_payload_bytes(), Some(bytes.len() as u64));
    let total = budget.charged_memory_bytes();
    assert!(total > bytes.len() as u64);
    drop(owned);
    assert_eq!(budget.charged_memory_bytes(), 0);
    total
}

/// Measure counted work charged to `execution` during `operation`.
pub(crate) fn measure_counted_work<T>(
    execution: &Req<'_, '_>,
    operation: impl FnOnce() -> T,
) -> (T, u64) {
    let (value, report) = execution
        .run(worth_execution::ExecutionWorkCeiling::new(None), |_| {
            operation()
        })
        .expect("unbounded counted work measurement completes");
    (value, report.charged_work())
}

impl DurableMedium for OwnedMediumAttachment {
    fn publish_initial_inputs(
        &self,
        artifact: &InitArt,
        request: &Req<'_, '_>,
    ) -> Fate<InitDetail> {
        if let Err(cause) = request.consult() {
            return Fate::ProvenNoEffect {
                detail: InitDetail::Request(cause.into()),
            };
        }
        let mut state = self.0.lock();
        if state
            .initial_inputs
            .as_ref()
            .is_some_and(|old| old.as_bytes() == artifact.as_bytes())
        {
            return Fate::Committed;
        }
        let revision = state
            .initial_inputs
            .as_ref()
            .map_or(0, |old| old.revision());
        if artifact.revision() != revision + 1 {
            return Fate::ProvenNoEffect {
                detail: InitDetail::Physical("initial input revision conflict".into()),
            };
        }
        state.initial_inputs = Some(artifact.clone());
        Fate::Committed
    }
    fn checkpoint_selection(&self, subject: &CheckpointSubject) -> SelectionObs {
        match self.0.lock().selected.as_ref() {
            Some(selected) if selected.same_capture(subject) => {
                SelectionObs::Selected(CheckpointSelectionIdentity {
                    generation: 1,
                    manifest_sha256: Sha256::digest(subject.bytes()).into(),
                })
            }
            _ => SelectionObs::NotSelected,
        }
    }
    fn cold_source(&self) -> Arc<dyn ColdSource> {
        self.0.clone()
    }
    fn read_transition(&self, sequence: Seq) -> Result<TransArt, TransReadErr> {
        let bytes = self
            .0
            .lock()
            .transitions
            .entries
            .get(&sequence.0)
            .cloned()
            .ok_or(TransReadErr::Missing(sequence))?;
        TransArt::from_untrusted_bytes(bytes).map_err(TransReadErr::Frame)
    }
    fn publish(&self, artifact: &CommitArt) -> Fate<String> {
        self.0.publish_commit(artifact)
    }
    fn publish_transition(&self, artifact: &TransArt) -> Fate<String> {
        self.0.publish_transition(artifact)
    }
}

pub(crate) fn durable_runtime_for_medium(
    medium: &Arc<OwnedTestDurableMedium>,
) -> RelationalRuntime {
    durable_runtime_profile_for_medium(RelationalRuntimeProfile::CertificationCore, medium)
}
pub(crate) fn durable_runtime_profile_for_medium(
    profile: RelationalRuntimeProfile,
    medium: &Arc<OwnedTestDurableMedium>,
) -> RelationalRuntime {
    durable_runtime_schema_profile_for_medium(profile, test_schema_registry(), medium)
}
pub(crate) fn durable_runtime_schema_profile_for_medium(
    profile: RelationalRuntimeProfile,
    schema_registry: RelationalSchemaRegistry,
    medium: &Arc<OwnedTestDurableMedium>,
) -> RelationalRuntime {
    RelationalRuntimeApi::builder()
        .profile(profile)
        .schema_registry(schema_registry)
        .build_durable(medium.attachment())
        .expect("valid durable native runtime configuration")
}
pub(crate) fn capture_and_file_checkpoint(
    execution: &Req<'_, '_>,
    runtime: &RelationalRuntime,
    medium: &Arc<OwnedTestDurableMedium>,
) -> NativeCheckpoint {
    let checkpoint = runtime
        .durability_authority()
        .native_checkpoint_with_policy(
            execution,
            execution.allocation_policy(),
            CheckpointPolicy::ExternalMembers {
                current_members: &[],
                source: None,
            },
        )
        .expect("native checkpoint capture");
    medium.file_checkpoint(checkpoint.clone());
    checkpoint
}
pub(crate) fn recover_runtime_from_medium(
    execution: &Req<'_, '_>,
    recovered: &mut RelationalRuntime,
    medium: &Arc<OwnedTestDurableMedium>,
) -> RecoveryOutcome {
    let plan = medium
        .recovery_plan(
            execution,
            recovered,
            RecoveryMode::NormalRecoveryVerification,
        )
        .expect("medium-sourced recovery plan");
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("medium-sourced recovery")
}
impl OwnedTestDurableMedium {
    pub(crate) fn recovery_plan(
        self: &Arc<Self>,
        execution: &Req<'_, '_>,
        target: &RelationalRuntime,
        mode: RecoveryMode,
    ) -> Result<
        crate::durability::data::RecoveryPlan,
        crate::durability::data::RelationalRecoveryStop,
    > {
        target
            .durability()
            .recovery_plan_from_source(execution, self.recovery_source(), mode)
    }
    /// Build a recovery plan with the medium's filed external-member checkpoint
    /// expanded into in-memory `DurableCheckpoint` fields so contract tests can
    /// mutate specific fields and verify `runtime_rebuild` validation.
    pub(crate) fn expanded_recovery_plan(
        self: &Arc<Self>,
        execution: &Req<'_, '_>,
        target: &RelationalRuntime,
        mode: RecoveryMode,
    ) -> Result<
        crate::durability::data::RecoveryPlan,
        crate::durability::data::RelationalRecoveryStop,
    > {
        let mut plan = self.recovery_plan(execution, target, mode)?;
        let filed = self.lock().checkpoint.clone();
        if let Some(filed) = filed {
            let expanded =
                crate::durability::log::native_checkpoint_codec::decode_expanded_checkpoint(
                    execution,
                    filed.checkpoint().bytes(),
                    self.clone(),
                )?
                .checkpoint;
            plan.checkpoint = Some(expanded);
        }
        Ok(plan)
    }
    pub(crate) fn rewrite_artifact_envelope(
        &self,
        execution: &Req<'_, '_>,
        position: PatchStreamPosition,
        edit: impl FnOnce(&mut crate::history::data::CanonicalCommitEnvelopeBody),
    ) {
        let bytes = self
            .commit_bytes_at(position)
            .unwrap_or_else(|| panic!("medium holds no artifact at {position:?}"));
        let readmitted = CommitArt::readmit(&bytes, execution)
            .expect("the medium's own artifact readmits before it is rewritten");
        let mut envelope = readmitted.envelope().copy_original_for_test(execution);
        edit(envelope.body_mut_for_test());
        let rewritten = CommitArt::from_positioned(
            &crate::history::data::PositionedCanonicalCommit::for_test(
                position,
                Arc::new(envelope),
            ),
            execution.allocation_policy(),
        )
        .expect("a rewritten envelope still encodes");
        self.replace_commit_bytes(position, Box::from(rewritten.as_bytes()));
    }
    pub(crate) fn rewrite_commit_envelope(
        &self,
        execution: &Req<'_, '_>,
        commit_id: crate::history::data::CommitId,
        edit: impl FnOnce(&mut crate::history::data::CanonicalCommitEnvelopeBody),
    ) {
        let position = self
            .positions()
            .into_iter()
            .find(|p| {
                self.artifact_bytes(*p)
                    .and_then(|b| CommitArt::from_untrusted_bytes(b).ok())
                    .is_some_and(|a| a.commit_id() == commit_id)
            })
            .unwrap_or_else(|| panic!("medium holds no artifact for {commit_id:?}"));
        self.rewrite_artifact_envelope(execution, position, edit);
    }
}
