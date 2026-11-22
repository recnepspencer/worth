use std::any::Any;
use std::sync::{Arc, Weak};

use sha2::{Digest, Sha256};
use worth_execution::{ExecutionImmutableBytes as ImmBytes, ExecutionRequest as Req};
use worth_testkit_relational::durable_medium::OwnedNativeReadSnapshot;

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

#[path = "durability/owned_medium.rs"]
mod owned_medium;
use owned_medium::{bounded_bytes, downcast_snapshot, map_owned_denial, OwnedMediumAttachment};
pub(crate) use owned_medium::OwnedTestDurableMedium;

pub(crate) fn expect_durability_failure(
    stop: crate::durability::data::RelationalRecoveryStop,
) -> crate::durability::data::DurabilityError {
    match stop {
        crate::durability::data::RelationalRecoveryStop::AdmissionImage(cause) => {
            panic!("expected failed durable evidence, received admission image refusal: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Durability(error) => error,
        crate::durability::data::RelationalRecoveryStop::Content(cause) => {
            panic!("unexpected content refusal: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Allocation(cause) => {
            panic!("expected failed durable evidence, received allocation stop: {cause:?}")
        }
        crate::durability::data::RelationalRecoveryStop::Request(cause) => {
            panic!("expected failed durable evidence, received request stop: {cause:?}")
        }
    }
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
    fn publish_initial_inputs(&self, artifact: &InitArt, request: &Req<'_, '_>) -> Fate<InitDetail> {
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
        let revision = state.initial_inputs.as_ref().map_or(0, |old| old.revision());
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
        if self
            .0
            .lock()
            .commits
            .publish(artifact.position().0, artifact.as_bytes())
        {
            Fate::Committed
        } else {
            Fate::ProvenNoEffect {
                detail: "not next or identical original position".into(),
            }
        }
    }
    fn publish_transition(&self, artifact: &TransArt) -> Fate<String> {
        if self
            .0
            .lock()
            .transitions
            .publish(artifact.sequence().0, artifact.as_bytes())
        {
            Fate::Committed
        } else {
            Fate::ProvenNoEffect {
                detail: "not next or identical original transition".into(),
            }
        }
    }
}

impl ColdSource for OwnedTestDurableMedium {
    fn read_member_prefix_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        member: MemberLoc,
        length: usize,
    ) -> Result<ImmBytes, ReadRefusal> {
        downcast_snapshot(original)?
            .member_prefix(request, member.sha256, member.length, length)
            .map_err(map_owned_denial)
    }
    fn read_initial_inputs_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        limit: u64,
    ) -> Result<Option<ImmBytes>, ReadRefusal> {
        downcast_snapshot(original)?
            .initial_inputs(request, limit)
            .map_err(map_owned_denial)
    }
    fn checkpoint_read_protection(
        &self,
        request: &Req<'_, '_>,
    ) -> Result<(ReadBounds, ReadPayload), ReadRefusal> {
        let contents = self.lock();
        let snapshot = OwnedNativeReadSnapshot::capture(
            request,
            contents.commits.entries.iter().map(|(id, b)| (*id, b.as_ref())),
            contents
                .transitions
                .entries
                .iter()
                .map(|(id, b)| (*id, b.as_ref())),
            &contents.members,
            contents.initial_inputs.as_ref().map(|a| a.as_bytes()),
        )
        .map_err(map_owned_denial)?;
        let [canonical, transitions] = snapshot.bounds();
        Ok((
            ReadBounds {
                selected_checkpoint: contents.checkpoint.is_some(),
                canonical: PatchStreamPosition(canonical),
                transitions: Seq(transitions),
            },
            snapshot,
        ))
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &Req<'_, '_>,
    ) -> Result<ReadPayload, ReadRefusal> {
        let contents = self.lock();
        OwnedNativeReadSnapshot::capture(
            request,
            std::iter::empty(),
            std::iter::empty(),
            &contents.members,
            contents.initial_inputs.as_ref().map(|a| a.as_bytes()),
        )
        .map(|original| original as ReadPayload)
        .map_err(map_owned_denial)
    }
    fn read_commit_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        position: PatchStreamPosition,
        limit: u64,
    ) -> Result<ImmBytes, ReadRefusal> {
        downcast_snapshot(original)?
            .canonical(request, position.0, limit)
            .map_err(map_owned_denial)
    }
    fn read_transition_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        sequence: Seq,
        limit: u64,
    ) -> Result<ImmBytes, ReadRefusal> {
        downcast_snapshot(original)?
            .transition(request, sequence.0, limit)
            .map_err(map_owned_denial)
    }
    fn read_member_owned(
        &self,
        request: &Req<'_, '_>,
        original: &(dyn Any + Send + Sync),
        locator: MemberLoc,
        limit: u64,
    ) -> Result<ImmBytes, ReadRefusal> {
        downcast_snapshot(original)?
            .member(request, locator.sha256, locator.length, limit)
            .map_err(map_owned_denial)
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn Any + Send + Sync),
    ) -> Result<(), RetireRefusal> {
        if original.is::<()>() || original.is::<OwnedNativeReadSnapshot>() {
            Ok(())
        } else {
            Err(RetireRefusal::ForeignCustody)
        }
    }
    fn advance_member_custody_retirement(
        &self,
        _: &Req<'_, '_>,
        original: &mut (dyn Any + Send + Sync),
    ) -> RetireObs {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RetireObs::Complete,
            Err(cause) => RetireObs::Refused(cause),
        }
    }
    fn retain_members_for_request(
        &self,
        request: &Req<'_, '_>,
        _: &[MemberLoc],
        issuer: Weak<dyn ColdSource>,
    ) -> Result<MemberCustody, ReadRefusal> {
        MemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }
    fn retain_members(&self, _: &[MemberLoc]) -> Result<MemberCustody, StorageCause> {
        Ok(MemberCustody::new(()))
    }
    fn read_member(&self, locator: MemberLoc, limit: u64) -> Result<Vec<u8>, StorageCause> {
        bounded_bytes(self.lock().members.get(locator.sha256, locator.length), limit)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, StorageCause> {
        bounded_bytes(
            self.lock().commits.entries.get(&position.0).map(AsRef::as_ref),
            limit,
        )
    }
}

pub(crate) fn durable_runtime_for_medium(medium: &Arc<OwnedTestDurableMedium>) -> RelationalRuntime {
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
    let plan = recovered
        .durability()
        .recovery_plan_from_source(
            execution,
            medium.recovery_source(),
            RecoveryMode::NormalRecoveryVerification,
        )
        .expect("medium-sourced recovery plan");
    recovered
        .durability_recovery()
        .recover(execution, plan)
        .expect("medium-sourced recovery")
}
