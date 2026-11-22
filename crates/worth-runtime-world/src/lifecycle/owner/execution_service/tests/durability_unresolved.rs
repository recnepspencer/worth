//! A Relational publication whose durable fate is unknown may already be
//! durable, so World retains it as an owner effect instead of reporting
//! no-effect, and offers only release, inspection and owner closure.
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;

use worth_relational::facade::durability::{
    DurablePublicationFate, RelationalBranchTransitionArtifact, RelationalCanonicalCommitArtifact,
    RelationalDurableMedium,
};
use worth_relational::facade::runtime::RelationalRuntimeApi;

use super::*;

/// A durable medium that commits until it is told, through the switch the
/// test keeps, to leave the next publication's fate unknown, as a crash after
/// the medium's effect would. Branch transitions are answered the same way.
#[derive(Debug, Default)]
struct UnresolvingMedium {
    unresolve_next: Arc<AtomicBool>,
    journal: Arc<JournalContents>,
}

impl UnresolvingMedium {
    fn answer(&self) -> DurablePublicationFate<String> {
        if self.unresolve_next.swap(false, Ordering::AcqRel) {
            return DurablePublicationFate::Indeterminate {
                detail: "the medium lost its answer after the effect".to_owned(),
            };
        }
        DurablePublicationFate::Committed
    }
}

impl RelationalDurableMedium for UnresolvingMedium {
    fn cold_source(
        &self,
    ) -> Arc<dyn worth_relational::facade::durability::RelationalDurableColdSource> {
        self.journal.clone()
    }
    fn read_transition(
        &self,
        sequence: worth_relational::facade::branch::RelationalBranchTransitionSequence,
    ) -> Result<
        RelationalBranchTransitionArtifact,
        worth_relational::facade::durability::RelationalDurableTransitionReadFailure,
    > {
        self.journal.transitions.lock().unwrap().iter()
            .find(|artifact| artifact.sequence() == sequence).cloned()
            .ok_or(worth_relational::facade::durability::RelationalDurableTransitionReadFailure::Missing(sequence))
    }

    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        self.journal.commits.lock().unwrap().push(artifact.clone());
        self.answer()
    }

    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        self.journal
            .transitions
            .lock()
            .unwrap()
            .push(artifact.clone());
        self.answer()
    }
}

#[test]
fn unresolved_relational_durability_retains_a_possible_effect() {
    let medium = UnresolvingMedium::default();
    let unresolve_next = Arc::clone(&medium.unresolve_next);
    let relational = RelationalRuntimeApi::builder()
        .build_durable(Box::new(medium))
        .expect("valid native runtime configuration");
    let mut fixture =
        reference_test_fixture::real_fixture_with_relational_runtime(relational, 12, 12);
    let owner = Arc::new(
        TestOwner::new(
            fixture.owner_inputs(budgets(4), RuntimeWorldClock::from_source(FixedClock)),
        )
        .expect("managed owner construction"),
    );
    let expected = match owner.bootstrap_root(fixture.bootstrap_intent()) {
        RuntimeWorldBootstrapOutcome::Performed(performed) => performed.product_branch().clone(),
        RuntimeWorldBootstrapOutcome::NoEffect(no_effect) => {
            panic!("bootstrap unexpectedly denied: {:?}", no_effect.cause())
        }
    };
    let serial_request = worth_execution::SerialRequest::from_memory(
        worth_execution::SerialMemoryBudget::from_policy(&owner.state.execution.request_policy()),
        worth_execution::CancellationToken::new(),
        None,
    );
    let execution = worth_execution::ExecutionRequest::serial(&serial_request);
    let prepared = prepare_relational(&execution, &fixture, &owner, &expected, "unresolved");
    unresolve_next.store(true, Ordering::Release);

    let retained = match execute_without_signal(execution, &owner, prepared) {
        OwnerExecutionOutcome::ProductUnpublished(retained) => retained,
        other => panic!("an unresolved durable fate is never no-effect: {other:?}"),
    };

    assert_eq!(
        retained.cause(),
        ProductUnpublishedCause::RelationalDurabilityUnresolved
    );
    assert_eq!(retained.owner_effect_count(), 1);
    assert_eq!(
        retained.progress().relational_posture(),
        RelationalAttemptProgressPosture::DurabilityUnresolved
    );
    assert_eq!(
        retained.progress().signal_posture(),
        SignalAttemptProgressPosture::Untouched
    );
    assert!(!retained.progress().relational_requires_settlement());
    assert_eq!(
        retained.next_actions(),
        [
            ProductUnpublishedNextAction::ReleaseObligations,
            ProductUnpublishedNextAction::Inspect,
            ProductUnpublishedNextAction::CloseOwner,
        ],
        "nothing is settled or settleable here; reopening the owner resolves the fate"
    );
    assert_eq!(
        RuntimeWorldObservationService::observe_product_branch(
            owner.as_ref(),
            &expected.branch_identity().clone(),
        )
        .expect("the product branch stays observable")
        .snapshot(),
        expected.snapshot(),
        "the product head does not move for an unresolved owner effect"
    );
}

/// The test medium's actual durable bytes, including a lost-answer publication.
#[derive(Debug, Default)]
struct JournalContents {
    commits: std::sync::Mutex<Vec<RelationalCanonicalCommitArtifact>>,
    transitions: std::sync::Mutex<Vec<RelationalBranchTransitionArtifact>>,
}
impl worth_relational::facade::durability::RelationalDurableColdSource for JournalContents {
    fn retain_members(
        &self,
        members: &[worth_relational::facade::durability::RelationalDurableMemberLocator],
    ) -> Result<
        worth_relational::facade::durability::RelationalDurableMemberCustody,
        worth_relational::facade::storage::RelationalStorageUnavailableCause,
    > {
        if !members.is_empty() {
            return Err(
                worth_relational::facade::storage::RelationalStorageUnavailableCause::MissingMember,
            );
        }
        Ok(worth_relational::facade::durability::RelationalDurableMemberCustody::new(()))
    }
    fn read_member(
        &self,
        _: worth_relational::facade::durability::RelationalDurableMemberLocator,
        _: u64,
    ) -> Result<Vec<u8>, worth_relational::facade::storage::RelationalStorageUnavailableCause> {
        Err(worth_relational::facade::storage::RelationalStorageUnavailableCause::MissingMember)
    }
    fn read_commit(
        &self,
        position: worth_relational::facade::publication::PatchStreamPosition,
        limit: u64,
    ) -> Result<Vec<u8>, worth_relational::facade::storage::RelationalStorageUnavailableCause> {
        use worth_relational::facade::storage::RelationalStorageUnavailableCause as Cause;
        let commits = self.commits.lock().unwrap();
        let bytes = commits
            .iter()
            .find(|artifact| artifact.position() == position)
            .ok_or(Cause::MissingMember)?
            .as_bytes();
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.to_vec())
    }
}

#[derive(Debug, Clone, Copy)]
enum ScriptedHistoryAnswer { Commit, Busy, Unresolved }

#[derive(Debug)]
struct ScriptedHistoryMedium {
    binding: crate::durability::WorldDurableHomeBinding,
    answers: Mutex<std::collections::VecDeque<ScriptedHistoryAnswer>>,
    next_position: Mutex<u64>,
}

impl ScriptedHistoryMedium {
    fn new(answers: Vec<ScriptedHistoryAnswer>) -> Arc<Self> {
        use worth_foundational::facade::{
            DurableWorldOwnerIdentity, HomeIdentity, HomeIncarnation, WorldOwnerSlotIdentity,
        };
        let owner = DurableWorldOwnerIdentity::from_recorded(
            HomeIdentity::from_observed_bytes([0x44; 16]).unwrap(),
            WorldOwnerSlotIdentity::from_recorded(0),
        );
        let binding = crate::durability::WorldDurableHomeBinding::from_admitted_adapter(
            owner,
            HomeIncarnation::from_recorded(1).unwrap(),
            [0x55; 32],
        );
        Arc::new(Self { binding, answers: Mutex::new(answers.into()), next_position: Mutex::new(1) })
    }
}

impl crate::durability::RuntimeWorldDurableMedium for ScriptedHistoryMedium {
    fn binding(&self) -> &crate::durability::WorldDurableHomeBinding { &self.binding }
    fn publication_task_posture(&self) -> crate::durability::WorldPublicationTaskPosture {
        crate::durability::WorldPublicationTaskPosture::Terminal
    }
    fn retire_for_home(&self, _: &worth_execution::ExecutionRequest<'_, '_>) -> Result<(), crate::durability::WorldDurableMediumDenial> { Ok(()) }
    fn capture_checkpoint_source(&self, _: &worth_execution::ExecutionRequest<'_, '_>) -> Result<Box<dyn crate::durability::WorldCheckpointReadSource>, crate::durability::WorldDurableMediumDenial> {
        Err(crate::durability::WorldDurableMediumDenial::Unavailable)
    }
    fn advance_publication(
        &self,
        subject: &crate::durability::WorldDurableArtifact,
        _: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> crate::durability::WorldPublicationObservation {
        use crate::durability::{
            WorldDurableMediumDenial as Denial, WorldPublicationCompletion as Completion,
            WorldPublicationObservation as Obs,
        };
        use sha2::{Digest, Sha256};
        match self.answers.lock().unwrap().pop_front().unwrap_or(ScriptedHistoryAnswer::Commit) {
            ScriptedHistoryAnswer::Commit => {
                let mut pos = self.next_position.lock().unwrap();
                let current = std::num::NonZeroU64::new(*pos).unwrap();
                *pos += 1;
                let digest = worth_foundational::facade::CanonicalDigestId::new(Sha256::digest(subject.as_bytes()).into());
                Obs::Finished(Completion::committed(worth_foundational::facade::WorldDurableJournal::Custody, current, digest))
            }
            ScriptedHistoryAnswer::Busy => Obs::Finished(Completion::proven_no_effect(Denial::PublicationBusy)),
            ScriptedHistoryAnswer::Unresolved => Obs::Finished(Completion::indeterminate(Denial::PublicationUnresolved)),
        }
    }
}

fn setup_history_chain(
    extra_commits: usize,
) -> (
    reference_test_fixture::RealReferenceFixture,
    TestOwner,
    Vec<Arc<crate::history::CompositeRuntimeWorldCommit>>,
    crate::history::ExplicitCommitHistoryProtectionObligation,
    crate::history::ExplicitCommitHistoryProtectionObligation,
) {
    let mut fx = reference_test_fixture::real_fixture(32, 32);
    let owner = TestOwner::new(fx.owner_inputs(
        crate::budget::RuntimeWorldBudgets::without_policy_limits(),
        RuntimeWorldClock::from_source(FixedClock),
    ))
    .unwrap();
    let RuntimeWorldBootstrapOutcome::Performed(performed) = owner.bootstrap_root(fx.bootstrap_intent()) else {
        panic!("bootstrap denied");
    };
    let root_obs = performed.product_branch().clone();
    let basis = root_obs.basis().clone();
    let root = owner.state.history.lookup(root_obs.selected_commit()).unwrap();
    drop((performed, root_obs));
    let root_guard = owner.state.history.protect_explicit_commit(root.as_ref()).unwrap();
    let mut chain = vec![root];
    for _ in 0..extra_commits {
        let (commit_id, attempt_id) = {
            let mut ids = owner.state.identities.lock().unwrap();
            (ids.composite_commit().unwrap(), ids.publication_attempt().unwrap())
        };
        let commit = Arc::new(
            crate::history::CompositeRuntimeWorldCommit::from_ordinary_publication(
                commit_id,
                chain.last().unwrap().as_ref(),
                basis.clone(),
                attempt_id,
                &crate::publication::CompositeOwnerExecutionResults::retained(),
                None,
            )
            .unwrap(),
        );
        let pins = owner.state.retention.issue_publication(&basis).unwrap().fork_history(&basis).unwrap();
        chain.push(owner.state.history.append(commit, pins).unwrap());
    }
    let tip_guard = owner.state.history.protect_explicit_commit(chain.last().unwrap().as_ref()).unwrap();
    (fx, owner, chain, root_guard, tip_guard)
}

#[test]
fn second_frame_busy_and_indeterminate_history_retirement_account_prefix_and_fences() {
    use crate::durability::WorldDurableMediumBinding;
    use crate::history::{CompositeHistoryCatalogDenial, HistoryReclamationDenial};
    // Case 1: 4 interior commits (2 chunks of 2); frame 1 commits, frame 2 returns Busy.
    let (_fx1, owner, chain, _rg1, _tg1) = setup_history_chain(5);
    let policy = owner.state.execution.request_policy();
    let serial = worth_execution::SerialRequest::from_policy(&policy, worth_execution::CancellationSource::new().token(), None);
    let exec = worth_execution::ExecutionRequest::serial(&serial);
    let medium = ScriptedHistoryMedium::new(vec![ScriptedHistoryAnswer::Commit, ScriptedHistoryAnswer::Busy]);
    let binding = Arc::new(WorldDurableMediumBinding::new(medium.clone(), &exec).unwrap());
    let pending = owner.state.history.begin_durable_retire_unprotected_history(chain.last().unwrap().identity()).unwrap();
    let err = owner.drive_durable_history_retirement_with_chunk_size(&binding, pending, 2).unwrap_err();
    assert!(matches!(
        err,
        HistoryReclamationDenial::PartialPublication { ref reclaimed, indeterminate: false }
            if reclaimed == &[chain[1].identity().clone(), chain[2].identity().clone()]
    ));
    assert!(owner.state.history.lookup(chain[1].identity()).is_none() && owner.state.history.lookup(chain[2].identity()).is_none());
    drop(owner.state.history.protect_explicit_commit(chain[3].as_ref()).expect("suffix unfenced"));
    medium.answers.lock().unwrap().push_back(ScriptedHistoryAnswer::Commit);
    let retry = owner.state.history.begin_durable_retire_unprotected_history(chain.last().unwrap().identity()).unwrap();
    let (reclaimed, held) = owner.drive_durable_history_retirement_with_chunk_size(&binding, retry, 2).unwrap();
    assert!(held.is_none());
    assert_eq!(reclaimed, vec![chain[3].identity().clone(), chain[4].identity().clone()]);

    // Case 2: 6 interior commits (3 chunks of 2); frame 1 commits, frame 2 is Unresolved.
    let (_fx2, owner2, chain2, _rg2, _tg2) = setup_history_chain(7);
    let medium2 = ScriptedHistoryMedium::new(vec![ScriptedHistoryAnswer::Commit, ScriptedHistoryAnswer::Unresolved]);
    let binding2 = Arc::new(WorldDurableMediumBinding::new(medium2, &exec).unwrap());
    let pending2 = owner2.state.history.begin_durable_retire_unprotected_history(chain2.last().unwrap().identity()).unwrap();
    let err2 = owner2.drive_durable_history_retirement_with_chunk_size(&binding2, pending2, 2).unwrap_err();
    assert!(matches!(
        err2,
        HistoryReclamationDenial::PartialPublication { ref reclaimed, indeterminate: true }
            if reclaimed == &[chain2[1].identity().clone(), chain2[2].identity().clone()]
    ));
    assert!(owner2.state.history.lookup(chain2[1].identity()).is_none());
    assert!(matches!(
        owner2.state.history.protect_explicit_commit(chain2[3].as_ref()),
        Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(_))
    ));
    drop(owner2.state.history.protect_explicit_commit(chain2[5].as_ref()).expect("frame 3 unfenced"));
    let retry2 = owner2.state.history.begin_durable_retire_unprotected_history(chain2.last().unwrap().identity()).unwrap();
    assert_eq!(retry2.remaining(), &[chain2[5].identity().clone(), chain2[6].identity().clone()]);
    let fresh = Arc::new(WorldDurableMediumBinding::new(ScriptedHistoryMedium::new(vec![ScriptedHistoryAnswer::Commit]), &exec).unwrap());
    let (reclaimed2, _) = owner2.drive_durable_history_retirement_with_chunk_size(&fresh, retry2, 2).unwrap();
    assert_eq!(reclaimed2, vec![chain2[5].identity().clone(), chain2[6].identity().clone()]);
    assert!(matches!(
        owner2.state.history.protect_explicit_commit(chain2[3].as_ref()),
        Err(CompositeHistoryCatalogDenial::UnknownProtectionTarget(_))
    ));
}
